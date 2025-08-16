use std::env;

#[derive(Debug, Clone)]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone)]
struct Transaction {
    transaction_type: TransactionType,
    client_id: u16,
    transaction_id: u32,
    amount: Option<u64>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_file = if args.len() > 1 { &args[1] } else { "unknown" };
    eprintln!("input = {}", input_file);
}
