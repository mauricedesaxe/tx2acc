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
    amount: u64,
}

impl ProcessedTransaction {
    pub fn new(
        transaction_id: u32,
        client_id: u16,
        amount: u64,
        transaction_type: ProcessedTransactionType,
    ) -> Self {
        Self {
            transaction_id,
            client_id,
            transaction_type,
            amount,
            dispute_status: DisputeStatus::Valid,
        }
    }
}
