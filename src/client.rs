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
