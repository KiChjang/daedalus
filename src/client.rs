use std::{collections::HashMap, default::Default};

use crate::error::Error;
use crate::transaction::{Transaction, TransactionType};

#[derive(Debug, PartialEq)]
/// The state of a client. Note that we don't store available or held funds --
/// both can be derived from the list of transactions that are in dispute. This
/// ensures that we always have a single source of truth, instead of requiring
/// us to update redundant fields in this struct.
pub struct Client {
    pub(crate) total: f32,
    pub(crate) locked: bool,
    disputed_tx: HashMap<u32, Transaction>,
}

impl Client {
    /// Retrieves the amount of funds held by disputes.
    /// Subtract this amount from the total to get the amount of available funds.
    pub fn get_held(&self) -> f32 {
        self.disputed_tx.values().map(|tx| tx.amount.unwrap()).sum()
    }

    fn unlock(&mut self) -> &mut Self {
        self.locked = false;
        self
    }

    fn deposit(&mut self, amount: f32) -> &mut Self {
        self.total += amount;
        self
    }

    fn withdraw(&mut self, amount: f32) -> Result<&mut Self, Error> {
        if self.locked {
            return Err(Error::AccountLocked);
        }

        let new_avail = self.total - self.get_held() - amount;

        if new_avail < 0.0 {
            return Err(Error::InsufficientBalance);
        }

        self.total -= amount;
        Ok(self)
    }

    fn dispute(&mut self, tx: Transaction) -> Result<&mut Self, Error> {
        // Withdrawal disputes need to be handled differently from deposit disputes,
        // since a reversal of a withdrawal implies _adding_ available funds, not
        // subtracting them. Since we're only disputing now, it would not make sense
        // to increase available funds yet. Instead, we'll keep the available funds
        // unchanged, and increase both the total and held funds.
        if matches!(tx.ty, TransactionType::Withdrawal) {
            // We increase the total here, but the available funds should still remain the same,
            // which would still make the equation (total = available + held) true.
            self.total += tx.amount.ok_or(Error::AmountMissing)?;
        }
        self.disputed_tx.insert(tx.id, tx);
        Ok(self)
    }

    fn resolve(&mut self, tx_id: u32) -> Result<&mut Self, Error> {
        if let Some(tx) = self.disputed_tx.remove(&tx_id) {
            if matches!(tx.ty, TransactionType::Withdrawal) {
                // By removing the disputed withdrawal, we decreased the amount
                // of held funds, so we must also decrease the total here to
                // fully reverse the transaction.
                self.total -= tx.amount.ok_or(Error::AmountMissing)?;
            }
        }

        Ok(self)
    }

    fn chargeback(&mut self, tx_id: u32) -> Result<&mut Self, Error> {
        if let Some(tx) = self.disputed_tx.remove(&tx_id) {
            match tx.ty {
                TransactionType::Deposit => {
                    self.total -= tx.amount.ok_or(Error::AmountMissing)?;
                }
                TransactionType::Withdrawal => {
                    // Removing the withdrawal transaction from the disputed
                    // map should increase the available funds while decreasing
                    // the held funds, keeping the total funds unchanged, so we
                    // do nothing here
                }
                // Impossible to hit since we should have prevented such kinds of transaction types to be added
                TransactionType::Dispute
                | TransactionType::Resolve
                | TransactionType::Chargeback => unreachable!(),
            }
            self.locked = true;
        }

        Ok(self)
    }

    /// Processes the given transaction for the client. The 2nd argument is
    /// used only for disputes, and it represents the transaction that is under
    /// dispute.
    /// Callers of `process_tx` must ensure that the 2nd argument is a deposit
    /// or a withdrawal. Any other kind of transaction may result in a panic
    /// during a chargeback.
    /// Callers must also ensure that any deposit or withdrawal transaction
    /// contain an amount field. Failing this requirement would result in the
    /// transaction being ignored.
    pub fn process_tx(
        &mut self,
        tx: Transaction,
        disputed_tx: Option<Transaction>,
    ) -> Result<&mut Self, Error> {
        match tx.ty {
            TransactionType::Deposit => Ok(self.deposit(tx.amount.ok_or(Error::AmountMissing)?)),
            TransactionType::Withdrawal => self.withdraw(tx.amount.ok_or(Error::AmountMissing)?),
            TransactionType::Dispute => {
                let disputed_tx = match disputed_tx {
                    Some(t) => t,
                    None => return Ok(self),
                };

                debug_assert_eq!(disputed_tx.client_id, tx.client_id);
                debug_assert!(matches!(
                    disputed_tx.ty,
                    TransactionType::Deposit | TransactionType::Withdrawal
                ));

                self.dispute(disputed_tx)
            }
            TransactionType::Resolve => self.resolve(tx.id),
            TransactionType::Chargeback => self.chargeback(tx.id),
        }
    }
}

impl Default for Client {
    fn default() -> Client {
        Client {
            total: 0.0,
            locked: false,
            disputed_tx: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::*;
    use crate::transaction::*;

    #[test]
    fn test_deposit_and_withdrawal() {
        let mut client = Client::default();
        client.deposit(1.0);

        assert!(client.withdraw(0.5).is_ok());
        assert!(client.withdraw(0.5).is_ok());
        assert_eq!(client.withdraw(0.5), Err(Error::InsufficientBalance));
    }

    #[test]
    fn test_should_disallow_withdraw_when_avail_funds_is_insufficient() -> Result<(), Error> {
        let mut client = Client::default();
        let tx = Transaction {
            ty: TransactionType::Deposit,
            client_id: 0,
            id: 1,
            amount: Some(1.0),
        };

        client.process_tx(tx.clone(), None)?.dispute(tx)?;

        assert_eq!(client.withdraw(1.0), Err(Error::InsufficientBalance));
        assert_eq!(client.total, 1.0);
        assert_eq!(client.get_held(), 1.0);

        Ok(())
    }

    #[test]
    fn test_should_allow_withdraw_when_dispute_is_resolved() -> Result<(), Error> {
        let mut client = Client::default();
        let tx = Transaction {
            ty: TransactionType::Deposit,
            client_id: 0,
            id: 1,
            amount: Some(1.0),
        };

        client
            .process_tx(tx.clone(), None)?
            .dispute(tx)?
            .resolve(1)?;

        assert!(client.withdraw(1.0).is_ok());
        assert_eq!(client.total, 0.0);
        assert_eq!(client.get_held(), 0.0);

        Ok(())
    }

    #[test]
    fn test_should_disallow_withdrawal_after_chargeback() -> Result<(), Error> {
        let mut client = Client::default();
        let tx = Transaction {
            ty: TransactionType::Deposit,
            client_id: 0,
            id: 1,
            amount: Some(1.0),
        };

        client
            .process_tx(tx.clone(), None)?
            .deposit(2.0)
            .dispute(tx)?
            .chargeback(1)?;

        assert_eq!(client.withdraw(1.0), Err(Error::AccountLocked));
        assert_eq!(client.withdraw(5.0), Err(Error::AccountLocked));
        assert_eq!(client.total, 2.0);
        assert_eq!(client.get_held(), 0.0);

        Ok(())
    }

    #[test]
    fn test_should_allow_withdrawal_under_dispute_if_avail_funds_exist() -> Result<(), Error> {
        let mut client = Client::default();
        let tx = Transaction {
            ty: TransactionType::Deposit,
            client_id: 0,
            id: 1,
            amount: Some(1.0),
        };

        client
            .process_tx(tx.clone(), None)?
            .deposit(2.0)
            .dispute(tx)?;

        assert_eq!(client.total, 3.0);
        assert_eq!(client.get_held(), 1.0);

        assert!(client.withdraw(2.0).is_ok());
        assert_eq!(client.total, 1.0);
        assert_eq!(client.get_held(), 1.0);

        Ok(())
    }

    #[test]
    fn test_dispute_withdrawal() -> Result<(), Error> {
        let mut client = Client::default();
        let tx = Transaction {
            ty: TransactionType::Withdrawal,
            client_id: 0,
            id: 2,
            amount: Some(1.0),
        };

        client
            .deposit(3.0)
            .process_tx(tx.clone(), None)?
            .dispute(tx)?;

        let tx2 = Transaction {
            ty: TransactionType::Withdrawal,
            client_id: 0,
            id: 3,
            amount: Some(2.0),
        };

        assert_eq!(client.total, 3.0);
        assert_eq!(client.get_held(), 1.0);

        assert!(client.process_tx(tx2.clone(), None).is_ok());

        client.resolve(1)?;

        assert_eq!(client.total, 1.0);
        assert_eq!(client.get_held(), 1.0);

        assert_eq!(client.withdraw(1.0), Err(Error::InsufficientBalance));

        client.resolve(2)?;

        assert_eq!(client.total, 0.0);
        assert_eq!(client.get_held(), 0.0);
        assert_eq!(client.withdraw(2.0), Err(Error::InsufficientBalance));

        client.dispute(tx2)?;

        assert_eq!(client.total, 2.0);
        assert_eq!(client.get_held(), 2.0);
        assert_eq!(client.withdraw(1.0), Err(Error::InsufficientBalance));

        client.chargeback(3)?;

        assert_eq!(client.total, 2.0);
        assert_eq!(client.get_held(), 0.0);
        assert_eq!(client.withdraw(1.0), Err(Error::AccountLocked));

        client.unlock();

        assert_eq!(client.total, 2.0);
        assert_eq!(client.get_held(), 0.0);
        assert!(client.withdraw(2.0).is_ok());

        Ok(())
    }
}
