#[derive(Debug, Clone)]
pub enum RawTransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone)]
pub struct RawTransaction {
    transaction_type: RawTransactionType,
    client_id: u16,
    transaction_id: u32,
    amount: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum ProcessedTransactionType {
    Deposit,
    Withdrawal,
}

#[derive(Debug, Clone)]
pub enum DisputeStatus {
    Valid,
    Disputed,
    Resolved,
    ChargedBack,
}

#[derive(Debug, Clone)]
pub struct ProcessedTransaction {
    transaction_type: ProcessedTransactionType,
    dispute_status: DisputeStatus,
    client_id: u16,
    transaction_id: u32,
    amount: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Client {
    client_id: u16,
    available: u64,
    held: u64,
    total: u64,
    locked: bool,
}
