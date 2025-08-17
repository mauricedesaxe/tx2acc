mod client;
mod convert;
mod types;

use client::Client;
use convert::convert_fractional_to_number;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::Path;
use types::{ProcessedTransaction, RawTransaction};

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
            types::RawTransactionType::Deposit => {
                eprintln!("Found a deposit with ID {}.", raw_tx.transaction_id);

                let amount = convert_fractional_to_number(
                    raw_tx.amount.expect("Deposit/Withdrawal must have amount"),
                );

                let client = clients
                    .entry(raw_tx.client_id)
                    .or_insert(Client::new(raw_tx.client_id));

                client.deposit(amount);
            }
            types::RawTransactionType::Withdrawal => {
                eprintln!("Found a withdrawal with ID {}.", raw_tx.transaction_id);

                let amount = convert_fractional_to_number(
                    raw_tx.amount.expect("Deposit/Withdrawal must have amount"),
                );

                let client = clients
                    .entry(raw_tx.client_id)
                    .or_insert(Client::new(raw_tx.client_id));

                client.withdraw(amount);
            }
            types::RawTransactionType::Dispute
            | types::RawTransactionType::Resolve
            | types::RawTransactionType::Chargeback => {
                eprintln!(
                    "Found an effect for transaction with ID {}.",
                    raw_tx.transaction_id
                );
            }
        }
    }

    Ok(())
}
