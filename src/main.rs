mod client;
mod convert;
mod processed_transaction;
mod raw_transaction;

use client::Client;
use convert::convert_fractional_to_number;
use processed_transaction::{DisputeStatus, ProcessedTransaction, ProcessedTransactionType};
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

        handle_transaction(&raw_tx, &mut transactions, &mut clients);
    }

    Ok(())
}

/// Takes in a raw transaction that should be a deposit,
/// a mutable reference to a hashmap of transactions,
/// and a mutable reference to a hashmap of clients.
/// Modifies the hash maps to reflect the transaction/effect.
fn handle_transaction(
    raw_tx: &RawTransaction,
    transactions: &mut HashMap<u32, ProcessedTransaction>,
    clients: &mut HashMap<u16, Client>,
) {
    match raw_tx.transaction_type {
        RawTransactionType::Deposit => {
            handle_deposit(raw_tx, transactions, clients);
        }
        RawTransactionType::Withdrawal => {
            handle_withdrawal(raw_tx, transactions, clients);
        }
        RawTransactionType::Dispute => {
            handle_dispute(raw_tx, transactions, clients);
        }
        RawTransactionType::Resolve => {
            handle_resolve(raw_tx, transactions, clients);
        }
        RawTransactionType::Chargeback => {
            handle_chargeback(raw_tx, transactions, clients);
        }
    }
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

fn handle_dispute(
    raw_tx: &RawTransaction,
    transactions: &mut HashMap<u32, ProcessedTransaction>,
    clients: &mut HashMap<u16, Client>,
) {
    if raw_tx.transaction_type != RawTransactionType::Dispute {
        panic!("You should never pass an invalid transaction type to handle_dispute")
    }

    eprintln!(
        "Found a dispute for transaction with ID {}.",
        raw_tx.transaction_id
    );

    if !clients.contains_key(&raw_tx.client_id) {
        // This is an easy skip, if the client doesn't exist it means a transaction
        // doesn't exist so the effect cannot be applied.
        // This is safe because the transactions are fed to the system chronologically
        // so a client should exist if they had a transaction before.
        eprintln!(
            "Client with ID {} not found while handling effect for tx {}.",
            raw_tx.client_id, raw_tx.transaction_id
        );
        return;
    }

    if !transactions.contains_key(&raw_tx.transaction_id) {
        // This is also an easy skip, if the transaction doesn't exist it means a transaction
        // doesn't exist so the effect cannot be applied.
        // This is safe because the transactions are fed to the system chronologically
        // so the transaction should exist if an effect came in from the CSV.
        eprintln!(
            "Transaction with ID {} not found while handling effect for tx {}.",
            raw_tx.transaction_id, raw_tx.transaction_id
        );
        return;
    }

    // I know unwrap is discouraged cause it can panic, but we
    // just checked that the transaction exists
    let tx = transactions.get_mut(&raw_tx.transaction_id).unwrap();

    if tx.dispute_status != DisputeStatus::Valid {
        eprintln!(
            "Failed to dispute transaction with ID {} because it is not valid.",
            raw_tx.transaction_id
        );
        return;
    }

    tx.dispute_status = DisputeStatus::Disputed;

    // I know unwrap is discouraged cause it can panic, but we
    // just checked that the client exists
    let client = clients.get_mut(&tx.client_id).unwrap();
    client.available -= tx.amount;
    client.held += tx.amount;
}

fn handle_resolve(
    raw_tx: &RawTransaction,
    transactions: &mut HashMap<u32, ProcessedTransaction>,
    clients: &mut HashMap<u16, Client>,
) {
    if raw_tx.transaction_type != RawTransactionType::Resolve {
        panic!("You should never pass an invalid transaction type to handle_resolve")
    }

    eprintln!(
        "Found a resolve for transaction with ID {}.",
        raw_tx.transaction_id
    );

    if !clients.contains_key(&raw_tx.client_id) {
        // This is an easy skip, if the client doesn't exist it means a transaction
        // doesn't exist so the effect cannot be applied.
        // This is safe because the transactions are fed to the system chronologically
        // so a client should exist if they had a transaction before.
        eprintln!(
            "Client with ID {} not found while handling effect for tx {}.",
            raw_tx.client_id, raw_tx.transaction_id
        );
        return;
    }

    if !transactions.contains_key(&raw_tx.transaction_id) {
        // This is also an easy skip, if the transaction doesn't exist it means a transaction
        // doesn't exist so the effect cannot be applied.
        // This is safe because the transactions are fed to the system chronologically
        // so the transaction should exist if an effect came in from the CSV.
        eprintln!(
            "Transaction with ID {} not found while handling effect for tx {}.",
            raw_tx.transaction_id, raw_tx.transaction_id
        );
        return;
    }

    // I know unwrap is discouraged cause it can panic, but we
    // just checked that the transaction exists
    let tx = transactions.get_mut(&raw_tx.transaction_id).unwrap();

    if tx.dispute_status != DisputeStatus::Disputed {
        eprintln!(
            "Failed to resolve transaction with ID {} because it is not disputed.",
            raw_tx.transaction_id
        );
        return;
    }

    tx.dispute_status = DisputeStatus::Resolved;

    // I know unwrap is discouraged cause it can panic, but we
    // just checked that the client exists
    let client = clients.get_mut(&tx.client_id).unwrap();
    client.available += tx.amount;
    client.held -= tx.amount;
}

fn handle_chargeback(
    raw_tx: &RawTransaction,
    transactions: &mut HashMap<u32, ProcessedTransaction>,
    clients: &mut HashMap<u16, Client>,
) {
    if raw_tx.transaction_type != RawTransactionType::Chargeback {
        panic!("You should never pass an invalid transaction type to handle_chargeback")
    }

    eprintln!(
        "Found a chargeback for transaction with ID {}.",
        raw_tx.transaction_id
    );

    if !clients.contains_key(&raw_tx.client_id) {
        // This is an easy skip, if the client doesn't exist it means a transaction
        // doesn't exist so the effect cannot be applied.
        // This is safe because the transactions are fed to the system chronologically
        // so a client should exist if they had a transaction before.
        eprintln!(
            "Client with ID {} not found while handling effect for tx {}.",
            raw_tx.client_id, raw_tx.transaction_id
        );
        return;
    }

    if !transactions.contains_key(&raw_tx.transaction_id) {
        // This is also an easy skip, if the transaction doesn't exist it means a transaction
        // doesn't exist so the effect cannot be applied.
        // This is safe because the transactions are fed to the system chronologically
        // so the transaction should exist if an effect came in from the CSV.
        eprintln!(
            "Transaction with ID {} not found while handling effect for tx {}.",
            raw_tx.transaction_id, raw_tx.transaction_id
        );
        return;
    }

    // I know unwrap is discouraged cause it can panic, but we
    // just checked that the transaction exists
    let tx = transactions.get_mut(&raw_tx.transaction_id).unwrap();

    if tx.dispute_status != DisputeStatus::Disputed {
        eprintln!(
            "Failed to chargeback transaction with ID {} because it is not disputed.",
            raw_tx.transaction_id
        );
        return;
    }

    tx.dispute_status = DisputeStatus::ChargedBack;

    // I know unwrap is discouraged cause it can panic, but we
    // just checked that the client exists
    let client = clients.get_mut(&tx.client_id).unwrap();
    client.held -= tx.amount;
    client.total -= tx.amount;
    client.locked = true;
}
