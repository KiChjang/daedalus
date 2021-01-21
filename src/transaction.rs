use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Struct representing a transaction. Primarily used during deserialization.
///
/// All deposit and withdrawal transactions MUST have an amount field.
/// The amount field for dispute, resolve and chargeback transactions are
/// OPTIONAL, but if present, the value will be ignored.
pub struct Transaction {
    #[serde(rename = "type")]
    pub(crate) ty: TransactionType,
    #[serde(rename = "client")]
    pub(crate) client_id: u16,
    #[serde(rename = "tx")]
    pub(crate) id: u32,
    pub(crate) amount: Option<f32>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use csv::ReaderBuilder;

    #[test]
    fn test_deserialize_deposit() {
        let data = "\
type,client,tx,amount\n
deposit,1,1,2.0";

        let mut rdr = ReaderBuilder::new()
            .flexible(true)
            .from_reader(data.as_bytes());

        for res in rdr.deserialize::<Transaction>() {
            let tx = res.unwrap();
            assert_eq!(tx.ty, TransactionType::Deposit);
            assert_eq!(tx.client_id, 1);
            assert_eq!(tx.id, 1);
            assert_eq!(tx.amount, Some(2.0));
        }
    }

    #[test]
    fn test_deserialize_withdrawal() {
        let data = "\
type,client,tx,amount\n
withdrawal,1,1,2.0";

        let mut rdr = ReaderBuilder::new()
            .flexible(true)
            .from_reader(data.as_bytes());

        for res in rdr.deserialize::<Transaction>() {
            let tx = res.unwrap();
            assert_eq!(tx.ty, TransactionType::Withdrawal);
            assert_eq!(tx.client_id, 1);
            assert_eq!(tx.id, 1);
            assert_eq!(tx.amount, Some(2.0));
        }
    }

    #[test]
    fn test_deserialize_dispute() {
        let data = "\
type,client,tx,amount\n
dispute,1,1";

        let mut rdr = ReaderBuilder::new()
            .flexible(true)
            .from_reader(data.as_bytes());

        for res in rdr.deserialize::<Transaction>() {
            let tx = res.unwrap();
            assert_eq!(tx.ty, TransactionType::Dispute);
            assert_eq!(tx.client_id, 1);
            assert_eq!(tx.id, 1);
            assert_eq!(tx.amount, None);
        }
    }

    #[test]
    fn test_deserialize_resolve() {
        let data = "\
type,client,tx,amount\n
resolve,1,1";

        let mut rdr = ReaderBuilder::new()
            .flexible(true)
            .from_reader(data.as_bytes());

        for res in rdr.deserialize::<Transaction>() {
            let tx = res.unwrap();
            assert_eq!(tx.ty, TransactionType::Resolve);
            assert_eq!(tx.client_id, 1);
            assert_eq!(tx.id, 1);
            assert_eq!(tx.amount, None);
        }
    }

    #[test]
    fn test_deserialize_chargeback() {
        let data = "\
type,client,tx,amount\n
chargeback,1,1";

        let mut rdr = ReaderBuilder::new()
            .flexible(true)
            .from_reader(data.as_bytes());

        for res in rdr.deserialize::<Transaction>() {
            let tx = res.unwrap();
            assert_eq!(tx.ty, TransactionType::Chargeback);
            assert_eq!(tx.client_id, 1);
            assert_eq!(tx.id, 1);
            assert_eq!(tx.amount, None);
        }
    }
}
