use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone)]
enum RawTransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone)]
struct RawTransaction {
    transaction_type: RawTransactionType,
    client_id: u16,
    transaction_id: u32,
    amount: Option<u64>,
}

#[derive(Debug, Clone)]
enum ProcessedTransactionType {
    Deposit,
    Withdrawal,
}

#[derive(Debug, Clone)]
enum DisputeStatus {
    Valid,
    Disputed,
    Resolved,
    ChargedBack,
}

#[derive(Debug, Clone)]
struct ProcessedTransaction {
    transaction_type: ProcessedTransactionType,
    dispute_status: DisputeStatus,
    client_id: u16,
    transaction_id: u32,
    amount: Option<u64>,
}

#[derive(Debug, Clone)]
struct Client {
    client_id: u16,
    available: u64,
    held: u64,
    total: u64,
    locked: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_file = if args.len() > 1 { &args[1] } else { "unknown" };
    eprintln!("input = {}", input_file);

    let mut transactions: HashMap<u32, ProcessedTransaction> = HashMap::new();
    let mut clients: HashMap<u16, Client> = HashMap::new();
}
