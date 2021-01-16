use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Struct representing a transaction. Primarily used during deserialization.
pub struct Transaction {
    #[serde(rename = "type")]
    pub(crate) ty: TransactionType,
    #[serde(rename = "client")]
    pub(crate) client_id: u16,
    #[serde(rename = "tx")]
    pub(crate) id: u32,
    pub(crate) amount: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Enum representing the transaction type for a transaction.
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
