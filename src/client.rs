use std::{collections::HashMap, default::Default};

use crate::error::Error;
use crate::transaction::{Transaction, TransactionType};

#[derive(Debug, PartialEq)]
pub struct Client {
    pub(crate) total: f32,
    pub(crate) locked: bool,
    disputed_tx: HashMap<u32, Transaction>,
}

impl Client {
    pub fn get_held(&self) -> f32 {
        self.disputed_tx.values().map(|tx| tx.amount).sum()
    }

    fn deposit(&mut self, amount: f32) -> Result<&mut Self, Error> {
        if self.locked {
            return Err(Error::AccountLocked);
        }

        self.total += amount;
        Ok(self)
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

    fn dispute(&mut self, tx: Transaction) -> &mut Self {
        if matches!(tx.ty, TransactionType::Dispute) {
            // Disputing a dispute results in only one underlying dispute, so we can ignore
            return self;
        }

        self.disputed_tx.insert(tx.id, tx);
        self
    }

    fn resolve(&mut self, tx_id: u32) -> &mut Self {
        self.disputed_tx.remove(&tx_id);

        self
    }

    fn chargeback(&mut self, tx_id: u32) -> &mut Self {
        if let Some(tx) = self.disputed_tx.remove(&tx_id) {
            match tx.ty {
                TransactionType::Deposit | TransactionType::Resolve => {
                    self.total -= tx.amount;
                }
                TransactionType::Withdrawal | TransactionType::Chargeback => {
                    self.total += tx.amount;
                }
                // Impossible to hit since we've already checked whether the tx type was dispute
                TransactionType::Dispute => unreachable!(),
            }
            self.locked = true;
        }

        self
    }

    pub fn process_tx(
        &mut self,
        tx: Transaction,
        disputed_tx: Option<Transaction>,
    ) -> Result<&mut Self, Error> {
        match tx.ty {
            TransactionType::Deposit => self.deposit(tx.amount),
            TransactionType::Withdrawal => self.withdraw(tx.amount),
            TransactionType::Dispute => {
                let disputed_tx = match disputed_tx {
                    Some(t) => t,
                    None => return Ok(self),
                };

                debug_assert_eq!(disputed_tx.client_id, tx.client_id);

                Ok(self.dispute(disputed_tx))
            }
            TransactionType::Resolve => Ok(self.resolve(tx.id)),
            TransactionType::Chargeback => Ok(self.chargeback(tx.id)),
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
        
        assert!(client.deposit(1.0).is_ok());
        assert!(client.withdraw(0.5).is_ok());
        assert!(client.withdraw(0.5).is_ok());
        assert_eq!(client.withdraw(0.5), Err(Error::InsufficientBalance));
    }
    
    #[test]
    fn test_should_disallow_withdraw_when_avail_funds_is_insufficient() {
        let mut client = Client::default();
        let tx = Transaction {
            ty: TransactionType::Deposit,
            client_id: 0,
            id: 1,
            amount: 1.0,
        };

        client.process_tx(tx.clone(), None)
            .unwrap()
            .dispute(tx);

        assert_eq!(client.withdraw(1.0), Err(Error::InsufficientBalance));
        assert_eq!(client.total, 1.0);
        assert_eq!(client.get_held(), 1.0);
    }
    
    #[test]
    fn test_should_allow_withdraw_when_dispute_is_resolved() {
        let mut client = Client::default();
        let tx = Transaction {
            ty: TransactionType::Deposit,
            client_id: 0,
            id: 1,
            amount: 1.0,
        };

        client.process_tx(tx.clone(), None)
            .unwrap()
            .dispute(tx)
            .resolve(1);

        assert!(client.withdraw(1.0).is_ok());
        assert_eq!(client.total, 0.0);
        assert_eq!(client.get_held(), 0.0);
    }

    #[test]
    fn test_should_disallow_withdrawal_after_chargeback() {
        let mut client = Client::default();
        let tx = Transaction {
            ty: TransactionType::Deposit,
            client_id: 0,
            id: 1,
            amount: 1.0,
        };

        client.process_tx(tx.clone(), None)
            .unwrap()
            .deposit(2.0)
            .unwrap()
            .dispute(tx)
            .chargeback(1);

        assert_eq!(client.withdraw(1.0), Err(Error::AccountLocked));
        assert_eq!(client.withdraw(5.0), Err(Error::AccountLocked));
        assert_eq!(client.total, 2.0);
        assert_eq!(client.get_held(), 0.0);
    }
}
