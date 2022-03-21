use csv::{self, Trim};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::env;
use std::process::exit;
use std::result::Result;
use thiserror::Error;

use rust_decimal::prelude::*;

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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        exit(1);
    }
    let tokens_res = read_from_file(&args[1]);
}
