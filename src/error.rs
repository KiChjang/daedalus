use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
/// Possible errors that may crop up during processing transactions.
pub enum Error {
    InsufficientBalance,
    AccountLocked,
    AmountMissing,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::InsufficientBalance => "Insufficient balance for withdrawal",
                Error::AccountLocked => "Account is frozen",
                Error::AmountMissing => "No amount specified",
            }
        )
    }
}
