#[derive(Debug, Clone)]
pub enum ProcessedTransactionType {
    Deposit,
    Withdrawal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DisputeStatus {
    Valid,
    Disputed,
    Resolved,
    ChargedBack,
}

#[derive(Debug, Clone)]
pub struct ProcessedTransaction {
    pub transaction_type: ProcessedTransactionType,
    pub dispute_status: DisputeStatus,
    pub client_id: u16,
    pub transaction_id: u32,
    pub amount: u64,
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
