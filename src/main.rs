use csv::{self, Trim};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::env;
use std::process::exit;
use std::result::Result;
use thiserror::Error;

use rust_decimal::prelude::*;

mod transaction;
use transaction::Transaction;

mod client;
use client::Client;

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct Token {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    #[serde(rename = "client")]
    client_id: u16,
    #[serde(rename = "tx")]
    transaction_id: u32,
    amount: Option<Decimal>,
}

/// Reads data from a file into a reader and deserializes each record
///
/// # Error
///
/// If an error occurs, the error is returned to `main`.
fn read_from_file(path: &str) -> Result<Vec<Token>, csv::Error> {
    // Creates a new csv `Reader` from a file
    let mut reader = csv::ReaderBuilder::new().trim(Trim::All).from_path(path)?;

    // Retrieve and print header record
    let _headers = reader.headers()?;

    Ok(reader.deserialize().flatten().collect())
}

#[derive(Debug, PartialEq, Error)]
pub enum Error {
    /// There client that requested an operation has the account locked.
    #[error("The client number: {0} has the account locked. No operations are allowed.")]
    LockedAccount(u16),
    #[error("The client number: {0} does not have associated the transaction with number: {1}")]
    TransactionDoesNotExist(u16, u32),
    #[error("The client number: {0} does not have enough credit for the requested withdrawal.")]
    NotEnoughCredit(u16),
    #[error("The transaction number: {0} for client number: {1} is not disputed.")]
    TransactionNotDisputed(u32, u16),
}

fn process_requests(
    tokens: Vec<Token>,
    mut clients: BTreeMap<u16, Client>,
) -> BTreeMap<u16, Client> {
    for token in tokens {
        match token.transaction_type {
            TransactionType::Deposit => {
                if let Some(amount) = token.amount {
                    let client = clients
                        .entry(token.client_id)
                        .or_insert_with(|| Client::new(token.client_id));
                    if client.deposit(token.transaction_id, amount).is_err() {
                        // do some error handling here
                    }
                }
            }
            TransactionType::Withdrawal => {
                if let Some(amount) = token.amount {
                    let client = clients
                        .entry(token.client_id)
                        .or_insert_with(|| Client::new(token.client_id));
                    if client.withdrawal(token.transaction_id, amount).is_err() {
                        // do some error handling here
                    }
                }
            }
            TransactionType::Dispute => {
                let client = clients
                    .entry(token.client_id)
                    .or_insert_with(|| Client::new(token.client_id));
                if client.dispute(token.transaction_id).is_err() {
                    // do some error handling here
                }
            }
            TransactionType::Resolve => {
                let client = clients
                    .entry(token.client_id)
                    .or_insert_with(|| Client::new(token.client_id));
                if client.resolve(token.transaction_id).is_err() {
                    // do some error handling here
                }
            }
            TransactionType::Chargeback => {
                let client = clients
                    .entry(token.client_id)
                    .or_insert_with(|| Client::new(token.client_id));
                if client.chargeback(token.transaction_id).is_err() {
                    // do some error handling here
                }
            }
        }
    }
    clients
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        exit(1);
    }
    let tokens_res = read_from_file(&args[1]);
    if let Ok(tokens) = tokens_res {
        let mut clients = BTreeMap::<u16, Client>::new();
        clients = process_requests(tokens, clients);
    } else {
        exit(1);
    }
}
