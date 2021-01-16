use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub(crate) ty: TransactionType,
    #[serde(rename = "client")]
    pub(crate) client_id: u16,
    #[serde(rename = "tx")]
    pub(crate) id: u32,
    pub(crate) amount: f32,
}

#[derive(Debug, Deserialize)]
pub enum TransactionType {
    #[serde(alias = "deposit")]
    Deposit,
    #[serde(alias = "withdrawal")]
    Withdrawal,
    #[serde(alias = "dispute")]
    Dispute,
    #[serde(alias = "resolve")]
    Resolve,
    #[serde(alias = "chargeback")]
    Chargeback,
}
