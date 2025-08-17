mod client;
mod convert;
mod processed_transaction;
mod raw_transaction;

use client::Client;
use convert::convert_fractional_to_number;
use processed_transaction::{ProcessedTransaction, ProcessedTransactionType};
use raw_transaction::{RawTransaction, RawTransactionType};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let input_file = if args.len() > 1 { &args[1] } else { "unknown" };
    eprintln!("input = {}", input_file);

    if !Path::new(input_file).exists() {
        eprintln!("Error: File '{}' not found", input_file);
        return Ok(());
    }

    let mut transactions: HashMap<u32, ProcessedTransaction> = HashMap::new();
    let mut clients: HashMap<u16, Client> = HashMap::new();

    let file = File::open(input_file)?;
    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(file);

    let mut row = 0;
    for result in csv_reader.deserialize() {
        row += 1;
        let raw_tx: RawTransaction = match result {
            Ok(tx) => tx,
            Err(e) => {
                eprintln!("Error parsing row: {}", e);
                continue;
            }
        };

        eprintln!("CSV Row {}, {:?}", row, raw_tx);

        match raw_tx.transaction_type {
            RawTransactionType::Deposit => {
                handle_deposit(&raw_tx, &mut transactions, &mut clients);
            }
            RawTransactionType::Withdrawal => {
                handle_withdrawal(&raw_tx, &mut transactions, &mut clients);
            }
            RawTransactionType::Dispute
            | RawTransactionType::Resolve
            | RawTransactionType::Chargeback => {
                eprintln!(
                    "Found an effect for transaction with ID {}.",
                    raw_tx.transaction_id
                );
            }
        }
    }

    Ok(())
}

/// Takes in a raw transaction that should be a deposit,
/// a mutable reference to a hashmap of transactions,
/// and a mutable reference to a hashmap of clients.
/// Modifies the hash maps to reflect the deposit.
///
/// Not the most testable or functional function. I don't love it
/// but I'd rather move fast and we can test at the "integration" layer.
fn handle_deposit(
    raw_tx: &RawTransaction,
    transactions: &mut HashMap<u32, ProcessedTransaction>,
    clients: &mut HashMap<u16, Client>,
) {
    if raw_tx.transaction_type != RawTransactionType::Deposit {
        panic!("You should never pass an invalid transaction type to handle_deposit")
    }

    eprintln!("Found a deposit with ID {}.", raw_tx.transaction_id);

    let amount =
        convert_fractional_to_number(raw_tx.amount.expect("Deposit/Withdrawal must have amount"));

    let client = clients
        .entry(raw_tx.client_id)
        .or_insert(Client::new(raw_tx.client_id));

    client.deposit(amount);

    if transactions.contains_key(&raw_tx.transaction_id) {
        // I want to ignore them because overwriting
        // would mean we lose any effects we've previously applied.
        eprintln!(
            "Ignoring duplicate transaction ID {}",
            raw_tx.transaction_id
        );
    } else {
        let transaction = ProcessedTransaction::new(
            raw_tx.transaction_id,
            raw_tx.client_id,
            amount,
            ProcessedTransactionType::Deposit,
        );

        transactions.insert(raw_tx.transaction_id, transaction);
    }
}

/// Takes in a raw transaction that should be a withdrawal,
/// a mutable reference to a hashmap of transactions,
/// and a mutable reference to a hashmap of clients.
/// Modifies the hash maps to reflect the deposit.
///
/// Not the most testable or functional function. I don't love it
/// but I'd rather move fast and we can test at the "integration" layer.
fn handle_withdrawal(
    raw_tx: &RawTransaction,
    transactions: &mut HashMap<u32, ProcessedTransaction>,
    clients: &mut HashMap<u16, Client>,
) {
    if raw_tx.transaction_type != RawTransactionType::Withdrawal {
        panic!("You should never pass an invalid transaction type to handle_withdrawal")
    }

    eprintln!("Found a withdrawal with ID {}.", raw_tx.transaction_id);

    let amount =
        convert_fractional_to_number(raw_tx.amount.expect("Deposit/Withdrawal must have amount"));

    let client = clients
        .entry(raw_tx.client_id)
        .or_insert(Client::new(raw_tx.client_id));

    client.withdraw(amount);

    if transactions.contains_key(&raw_tx.transaction_id) {
        // I want to ignore them because overwriting
        // would mean we lose any effects we've previously applied.
        eprintln!(
            "Ignoring duplicate transaction ID {}",
            raw_tx.transaction_id
        );
    } else {
        let transaction = ProcessedTransaction::new(
            raw_tx.transaction_id,
            raw_tx.client_id,
            amount,
            ProcessedTransactionType::Withdrawal,
        );
        transactions.insert(raw_tx.transaction_id, transaction);
    }
}
