use crate::client::Client;
use crate::convert::convert_fractional_to_number;
use crate::processed_transaction::{DisputeStatus, ProcessedTransaction, ProcessedTransactionType};
use crate::raw_transaction::{RawTransaction, RawTransactionType};
use std::collections::HashMap;

/// Takes in a raw transaction that should be a deposit,
/// a mutable reference to a hashmap of transactions,
/// and a mutable reference to a hashmap of clients.
/// Modifies the hash maps to reflect the transaction/effect.
pub fn handle_transaction(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Client;
    use crate::processed_transaction::ProcessedTransaction;
    use crate::raw_transaction::{RawTransaction, RawTransactionType};
    use std::collections::HashMap;

    #[test]
    fn test_handle_transaction_with_simple_data() {
        let mut transactions: HashMap<u32, ProcessedTransaction> = HashMap::new();
        let mut clients: HashMap<u16, Client> = HashMap::new();

        // These are the same as in `data/tx/sample_1.csv`
        let sample_transactions = vec![
            RawTransaction {
                transaction_type: RawTransactionType::Deposit,
                client_id: 1,
                transaction_id: 1,
                amount: Some(1.0),
            },
            RawTransaction {
                transaction_type: RawTransactionType::Deposit,
                client_id: 2,
                transaction_id: 2,
                amount: Some(5.0),
            },
            RawTransaction {
                transaction_type: RawTransactionType::Deposit,
                client_id: 1,
                transaction_id: 3,
                amount: Some(2.0),
            },
            RawTransaction {
                transaction_type: RawTransactionType::Withdrawal,
                client_id: 1,
                transaction_id: 4,
                amount: Some(1.5),
            },
            RawTransaction {
                transaction_type: RawTransactionType::Withdrawal,
                client_id: 2,
                transaction_id: 5,
                amount: Some(3.0),
            },
        ];

        for raw_tx in &sample_transactions {
            handle_transaction(raw_tx, &mut transactions, &mut clients);
        }

        assert_eq!(clients.len(), 2);
        assert!(clients.contains_key(&1));
        assert!(clients.contains_key(&2));

        let client1 = clients.get(&1).unwrap();
        assert_eq!(client1.available, 15000); // 1.5 * 10000
        assert_eq!(client1.held, 0);
        assert_eq!(client1.total, 15000); // 1.5 * 10000
        assert_eq!(client1.locked, false);

        let client2 = clients.get(&2).unwrap();
        assert_eq!(client2.available, 20000); // 2.0 * 10000
        assert_eq!(client2.held, 0);
        assert_eq!(client2.total, 20000); // 2.0 * 10000
        assert_eq!(client2.locked, false);

        assert_eq!(transactions.len(), 5);
        assert!(transactions.contains_key(&1));
        assert!(transactions.contains_key(&2));
        assert!(transactions.contains_key(&3));
        assert!(transactions.contains_key(&4));
        assert!(transactions.contains_key(&5));
    }

    #[test]
    fn test_handle_transaction_complex_data() {
        let mut transactions: HashMap<u32, ProcessedTransaction> = HashMap::new();
        let mut clients: HashMap<u16, Client> = HashMap::new();

        let complex_transactions = vec![
            RawTransaction {
                transaction_type: RawTransactionType::Deposit,
                client_id: 1,
                transaction_id: 1,
                amount: Some(1000.0),
            },
            RawTransaction {
                transaction_type: RawTransactionType::Deposit,
                client_id: 2,
                transaction_id: 4,
                amount: Some(800.0),
            },
            RawTransaction {
                transaction_type: RawTransactionType::Deposit,
                client_id: 3,
                transaction_id: 7,
                amount: Some(600.0),
            },
            RawTransaction {
                transaction_type: RawTransactionType::Deposit,
                client_id: 1,
                transaction_id: 2,
                amount: Some(500.0),
            },
            RawTransaction {
                transaction_type: RawTransactionType::Deposit,
                client_id: 2,
                transaction_id: 5,
                amount: Some(400.0),
            },
            RawTransaction {
                transaction_type: RawTransactionType::Deposit,
                client_id: 3,
                transaction_id: 8,
                amount: Some(300.0),
            },
            RawTransaction {
                transaction_type: RawTransactionType::Withdrawal,
                client_id: 1,
                transaction_id: 3,
                amount: Some(200.0),
            },
            RawTransaction {
                transaction_type: RawTransactionType::Withdrawal,
                client_id: 2,
                transaction_id: 6,
                amount: Some(100.0),
            },
            RawTransaction {
                transaction_type: RawTransactionType::Withdrawal,
                client_id: 3,
                transaction_id: 9,
                amount: Some(150.0),
            },
            RawTransaction {
                transaction_type: RawTransactionType::Dispute,
                client_id: 1,
                transaction_id: 1,
                amount: None,
            },
            RawTransaction {
                transaction_type: RawTransactionType::Dispute,
                client_id: 2,
                transaction_id: 4,
                amount: None,
            },
            RawTransaction {
                transaction_type: RawTransactionType::Dispute,
                client_id: 3,
                transaction_id: 7,
                amount: None,
            },
            RawTransaction {
                transaction_type: RawTransactionType::Dispute,
                client_id: 1,
                transaction_id: 2,
                amount: None,
            },
            RawTransaction {
                transaction_type: RawTransactionType::Dispute,
                client_id: 3,
                transaction_id: 8,
                amount: None,
            },
            RawTransaction {
                transaction_type: RawTransactionType::Resolve,
                client_id: 1,
                transaction_id: 2,
                amount: None,
            },
            RawTransaction {
                transaction_type: RawTransactionType::Resolve,
                client_id: 3,
                transaction_id: 7,
                amount: None,
            },
            RawTransaction {
                transaction_type: RawTransactionType::Chargeback,
                client_id: 2,
                transaction_id: 4,
                amount: None,
            },
        ];

        for raw_tx in &complex_transactions {
            handle_transaction(raw_tx, &mut transactions, &mut clients);
        }

        assert_eq!(clients.len(), 3);
        assert!(clients.contains_key(&1));
        assert!(clients.contains_key(&2));
        assert!(clients.contains_key(&3));

        // Client 1 has one unresolved dispute (tx1 = 1000 held), tx2 was resolved
        // We expect: available=300, held=1000, total=1300, locked=false
        let client1 = clients.get(&1).unwrap();
        assert_eq!(client1.available, 3000000); // 300.0 * 10000
        assert_eq!(client1.held, 10000000); // 1000.0 * 10000
        assert_eq!(client1.total, 13000000); // 1300.0 * 10000
        assert_eq!(client1.locked, false);

        // Client 2 has chargeback on tx4 so account should be locked
        // We expect available=300, held=0, total=300, locked=true
        let client2 = clients.get(&2).unwrap();
        assert_eq!(client2.available, 3000000); // 300.0 * 10000
        assert_eq!(client2.held, 0);
        assert_eq!(client2.total, 3000000); // 300.0 * 10000
        assert_eq!(client2.locked, true);

        // Client 3 has one unresolved dispute (tx8 = 300 held), tx7 was resolved
        // Expected: available=450, held=300, total=750, locked=false
        let client3 = clients.get(&3).unwrap();
        assert_eq!(client3.available, 4500000); // 450.0 * 10000
        assert_eq!(client3.held, 3000000); // 300.0 * 10000
        assert_eq!(client3.total, 7500000); // 750.0 * 10000
        assert_eq!(client3.locked, false);

        assert_eq!(transactions.len(), 9);
    }
}
