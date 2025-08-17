use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RawTransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawTransaction {
    #[serde(rename = "type")]
    pub transaction_type: RawTransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub transaction_id: u32,
    pub amount: Option<f64>,
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
    amount: u64,
}
