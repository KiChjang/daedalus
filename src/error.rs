use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    InsufficientBalance,
    AccountLocked,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::InsufficientBalance => "Insufficient balance for withdrawal",
                Error::AccountLocked => "Account is frozen",
            }
        )
    }
}
