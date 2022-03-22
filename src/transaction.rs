use rust_decimal::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Transaction {
    _transaction_id: u32,
    amount: Decimal,
    disputed: bool,
}

impl Transaction {
    pub fn new(id: u32, amount: Decimal) -> Self {
        Transaction {
            _transaction_id: id,
            amount,
            disputed: false,
        }
    }

    pub fn get_amount(self) -> Decimal {
        self.amount
    }

    pub fn get_dispute_status(self) -> bool {
        self.disputed
    }

    pub fn set_dispute_status(&mut self, dispute: bool) {
        self.disputed = dispute;
    }
}
