use crate::Error;
use crate::Transaction;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Client {
    client_id: u16,
    available_amount: Decimal,
    held_amount: Decimal,
    total_amount: Decimal,
    locked: bool,
    transactions: BTreeMap<u32, Transaction>,
}

impl Client {
    pub fn new(id: u16) -> Self {
        Client {
            client_id: id,
            available_amount: dec!(0.0000),
            held_amount: dec!(0.0000),
            total_amount: dec!(0.0000),
            locked: false,
            transactions: BTreeMap::<u32, Transaction>::new(),
        }
    }

    pub fn deposit(
        &mut self,
        transaction_id: u32,
        amount: Decimal,
    ) -> std::result::Result<(), Error> {
        if self.locked {
            return Err(Error::LockedAccount(self.client_id));
        }
        self.transactions
            .insert(transaction_id, Transaction::new(transaction_id, amount));
        self.total_amount += amount;
        self.available_amount += amount;
        Ok(())
    }

    pub fn withdrawal(
        &mut self,
        transaction_id: u32,
        amount: Decimal,
    ) -> std::result::Result<(), Error> {
        if self.locked {
            return Err(Error::LockedAccount(self.client_id));
        }

        if self.available_amount < amount {
            return Err(Error::NotEnoughCredit(self.client_id));
        }
        self.transactions
            .insert(transaction_id, Transaction::new(transaction_id, amount));
        self.total_amount -= amount;
        self.available_amount -= amount;
        Ok(())
    }

    pub fn dispute(&mut self, transaction_id: u32) -> std::result::Result<(), Error> {
        if self.locked {
            return Err(Error::LockedAccount(self.client_id));
        }
        if let Some(transaction) = self.transactions.get_mut(&transaction_id) {
            transaction.set_dispute_status(true);
            self.held_amount += transaction.get_amount();
            self.available_amount -= transaction.get_amount();
        } else {
            return Err(Error::TransactionDoesNotExist(
                self.client_id,
                transaction_id,
            ));
        }
        Ok(())
    }

    pub fn resolve(&mut self, transaction_id: u32) -> std::result::Result<(), Error> {
        if self.locked {
            return Err(Error::LockedAccount(self.client_id));
        }
        if let Some(transaction) = self.transactions.get_mut(&transaction_id) {
            if transaction.get_dispute_status() {
                transaction.set_dispute_status(false);
                self.held_amount -= transaction.get_amount();
                self.available_amount += transaction.get_amount();
            } else {
                return Err(Error::TransactionNotDisputed(
                    transaction_id,
                    self.client_id,
                ));
            }
        } else {
            return Err(Error::TransactionDoesNotExist(
                self.client_id,
                transaction_id,
            ));
        }
        Ok(())
    }

    pub fn chargeback(&mut self, transaction_id: u32) -> std::result::Result<(), Error> {
        if self.locked {
            return Err(Error::LockedAccount(self.client_id));
        }
        if let Some(transaction) = self.transactions.get_mut(&transaction_id) {
            if transaction.get_dispute_status() {
                transaction.set_dispute_status(false);
                self.held_amount -= transaction.get_amount();
                self.total_amount -= transaction.get_amount();
                self.locked = true;
            } else {
                return Err(Error::TransactionNotDisputed(
                    transaction_id,
                    self.client_id,
                ));
            }
        } else {
            return Err(Error::TransactionDoesNotExist(
                self.client_id,
                transaction_id,
            ));
        }
        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use crate::client::Client;
    use crate::Error;
    use rust_decimal::prelude::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_deposit_ok_case() {
        let mut client = Client::new(1);
        assert_eq!(client.client_id, 1);
        assert_eq!(client.total_amount, dec!(0.0000));
        assert_eq!(client.held_amount, dec!(0.0000));
        assert_eq!(client.available_amount, dec!(0.0000));
        client.deposit(1, dec!(2.0000)).unwrap();
        assert_eq!(client.total_amount, dec!(2.0000));
        assert_eq!(client.held_amount, dec!(0.0000));
        assert_eq!(client.available_amount, dec!(2.0000));
    }

    #[test]
    fn test_deposit_account_locked() {
        let mut client = Client::new(1);
        client.locked = true;
        assert_eq!(
            client.deposit(1, dec!(2.0000)).unwrap_err(),
            Error::LockedAccount(1)
        );
    }

    #[test]
    fn test_withdrawal_ok_case() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        client.withdrawal(2, dec!(1.0000)).unwrap();
        assert_eq!(client.total_amount, dec!(2.0000));
        assert_eq!(client.held_amount, dec!(0.0000));
        assert_eq!(client.available_amount, dec!(2.0000));
    }

    #[test]
    fn test_withdrawal_account_locked() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        client.locked = true;
        assert_eq!(
            client.withdrawal(2, dec!(1.0000)).unwrap_err(),
            Error::LockedAccount(1)
        );
        assert_eq!(client.total_amount, dec!(3.0000));
        assert_eq!(client.held_amount, dec!(0.0000));
        assert_eq!(client.available_amount, dec!(3.0000));
    }

    #[test]
    fn test_withdrawal_not_enough_credit() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        assert_eq!(
            client.withdrawal(2, dec!(4.0000)).unwrap_err(),
            Error::NotEnoughCredit(1)
        );
        assert_eq!(client.total_amount, dec!(3.0000));
        assert_eq!(client.held_amount, dec!(0.0000));
        assert_eq!(client.available_amount, dec!(3.0000));
    }

    #[test]
    fn test_dispute_ok_case() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        client.dispute(1).unwrap();
        assert!(client.transactions.get(&1).unwrap().get_dispute_status());
        client.deposit(2, dec!(3.0000)).unwrap();
        assert!(!client.transactions.get(&2).unwrap().get_dispute_status());
        assert_eq!(client.total_amount, dec!(6.0000));
        assert_eq!(client.held_amount, dec!(3.0000));
        assert_eq!(client.available_amount, dec!(3.0000))
    }

    #[test]
    fn test_dispute_account_locked() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        client.dispute(1).unwrap();
        client.locked = true;
        assert_eq!(
            client.deposit(2, dec!(3.0000)).unwrap_err(),
            Error::LockedAccount(1)
        );
        assert_eq!(client.total_amount, dec!(3.0000));
        assert_eq!(client.held_amount, dec!(3.0000));
        assert_eq!(client.available_amount, dec!(0.0000))
    }

    #[test]
    fn test_dispute_not_enough_for_withdraw() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        client.dispute(1).unwrap();
        client.deposit(2, dec!(3.0000)).unwrap();
        assert_eq!(
            client.withdrawal(3, dec!(4.0000)).unwrap_err(),
            Error::NotEnoughCredit(1)
        );
        assert_eq!(client.total_amount, dec!(6.0000));
        assert_eq!(client.held_amount, dec!(3.0000));
        assert_eq!(client.available_amount, dec!(3.0000))
    }

    #[test]
    fn test_dispute_transaction_does_not_exist() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        assert_eq!(
            client.dispute(2).unwrap_err(),
            Error::TransactionDoesNotExist(1, 2)
        );
        assert_eq!(client.total_amount, dec!(3.0000));
        assert_eq!(client.held_amount, dec!(0.0000));
        assert_eq!(client.available_amount, dec!(3.0000))
    }

    #[test]
    fn test_resolve_ok_case() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        client.dispute(1).unwrap();
        assert!(client.transactions.get(&1).unwrap().get_dispute_status());
        client.deposit(2, dec!(3.0000)).unwrap();
        client.resolve(1).unwrap();
        assert_eq!(client.total_amount, dec!(6.0000));
        assert_eq!(client.held_amount, dec!(0.0000));
        assert_eq!(client.available_amount, dec!(6.0000))
    }

    #[test]
    fn test_resolve_account_locked() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        client.dispute(1).unwrap();
        assert!(client.transactions.get(&1).unwrap().get_dispute_status());
        client.deposit(2, dec!(3.0000)).unwrap();
        client.locked = true;
        assert_eq!(client.resolve(1).unwrap_err(), Error::LockedAccount(1));
        assert_eq!(client.total_amount, dec!(6.0000));
        assert_eq!(client.held_amount, dec!(3.0000));
        assert_eq!(client.available_amount, dec!(3.0000))
    }

    #[test]
    fn test_resolve_transaction_does_not_exist() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        client.dispute(1).unwrap();
        assert!(client.transactions.get(&1).unwrap().get_dispute_status());
        client.deposit(2, dec!(3.0000)).unwrap();
        assert_eq!(
            client.resolve(3).unwrap_err(),
            Error::TransactionDoesNotExist(1, 3)
        );
        assert_eq!(client.total_amount, dec!(6.0000));
        assert_eq!(client.held_amount, dec!(3.0000));
        assert_eq!(client.available_amount, dec!(3.0000))
    }

    #[test]
    fn test_resolve_transaction_not_disputed() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        client.dispute(1).unwrap();
        assert!(client.transactions.get(&1).unwrap().get_dispute_status());
        client.deposit(2, dec!(3.0000)).unwrap();
        assert_eq!(
            client.resolve(2).unwrap_err(),
            Error::TransactionNotDisputed(2, 1)
        );
        assert_eq!(client.total_amount, dec!(6.0000));
        assert_eq!(client.held_amount, dec!(3.0000));
        assert_eq!(client.available_amount, dec!(3.0000))
    }

    #[test]
    fn test_chargeback_ok_case() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        client.dispute(1).unwrap();
        assert!(client.transactions.get(&1).unwrap().get_dispute_status());
        client.deposit(2, dec!(3.0000)).unwrap();
        client.chargeback(1).unwrap();
        assert!(client.locked);
        assert_eq!(client.total_amount, dec!(3.0000));
        assert_eq!(client.held_amount, dec!(0.0000));
        assert_eq!(client.available_amount, dec!(3.0000))
    }

    #[test]
    fn test_chargeback_account_locked() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        client.dispute(1).unwrap();
        assert!(client.transactions.get(&1).unwrap().get_dispute_status());
        client.deposit(2, dec!(3.0000)).unwrap();
        client.dispute(2).unwrap();
        client.chargeback(1).unwrap();
        assert_eq!(client.chargeback(2).unwrap_err(), Error::LockedAccount(1));
        assert!(client.locked);
        assert_eq!(client.total_amount, dec!(3.0000));
        assert_eq!(client.held_amount, dec!(3.0000));
        assert_eq!(client.available_amount, dec!(0.0000))
    }

    #[test]
    fn test_chargeback_transaction_does_not_exist() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        client.dispute(1).unwrap();
        assert!(client.transactions.get(&1).unwrap().get_dispute_status());
        client.deposit(2, dec!(3.0000)).unwrap();
        assert_eq!(
            client.chargeback(3).unwrap_err(),
            Error::TransactionDoesNotExist(1, 3)
        );
        assert!(!client.locked);
        assert_eq!(client.total_amount, dec!(6.0000));
        assert_eq!(client.held_amount, dec!(3.0000));
        assert_eq!(client.available_amount, dec!(3.0000))
    }

    #[test]
    fn test_chargeback_transaction_not_disputed() {
        let mut client = Client::new(1);
        client.deposit(1, dec!(3.0000)).unwrap();
        client.dispute(1).unwrap();
        assert!(client.transactions.get(&1).unwrap().get_dispute_status());
        client.deposit(2, dec!(3.0000)).unwrap();
        assert_eq!(
            client.chargeback(2).unwrap_err(),
            Error::TransactionNotDisputed(2, 1)
        );
        assert!(!client.locked);
        assert_eq!(client.total_amount, dec!(6.0000));
        assert_eq!(client.held_amount, dec!(3.0000));
        assert_eq!(client.available_amount, dec!(3.0000))
    }
}
