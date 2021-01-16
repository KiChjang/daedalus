use std::{collections::HashMap, default::Default};

use crate::error::Error;
use crate::transaction::{Transaction, TransactionType};

#[derive(Debug)]
pub struct Client {
    pub(crate) total: f32,
    pub(crate) locked: bool,
    pub(crate) nonce: u32,
    disputed_tx: HashMap<u32, Transaction>,
}

impl Client {
    pub fn get_held(&self) -> f32 {
        self.disputed_tx.values().map(|tx| tx.amount).sum()
    }

    pub fn deposit(&mut self, amount: f32) -> &mut Self {
        self.total += amount;
        self
    }

    pub fn withdraw(&mut self, amount: f32) -> Result<&mut Self, Error> {
        let new_bal = self.total - amount;

        if new_bal < 0.0 {
            return Err(Error::InsufficientBalance);
        }

        self.total = new_bal;
        Ok(self)
    }

    pub fn dispute(&mut self, tx: Transaction) -> Result<&mut Self, Error> {
        if matches!(tx.ty, TransactionType::Dispute) {
            // Disputing a dispute results in only one underlying dispute, so we can ignore
            return Ok(self);
        }

        self.disputed_tx.insert(tx.id, tx);
        Ok(self)
    }

    pub fn resolve(&mut self, tx_id: u32) -> &mut Self {
        self.disputed_tx.remove(&tx_id);

        self
    }

    pub fn chargeback(&mut self, tx_id: u32) -> &mut Self {
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
        self.nonce += 1;

        match tx.ty {
            TransactionType::Deposit => Ok(self.deposit(tx.amount)),
            TransactionType::Withdrawal => self.withdraw(tx.amount),
            TransactionType::Dispute => {
                let disputed_tx = match disputed_tx {
                    Some(t) => t,
                    None => return Ok(self),
                };

                self.dispute(disputed_tx)
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
            nonce: 0,
            disputed_tx: HashMap::new(),
        }
    }
}
