use std::fmt::Formatter;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CheckTx,
    SyncTx,
    SendErr,
    TxInternalErr(InternalError),
    Io(std::io::Error),
    Db(redis::RedisError),
    NotSupport(String),
    Unknown(String),
}

#[derive(Debug)]
pub enum InternalError {
    InvalidNonce(String),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CheckTx => write!(f, "tx check failed"),
            Error::SyncTx => write!(f, "tx not accepted by tendermint"),
            Error::SendErr => write!(f, "tx not sent"),
            Error::TxInternalErr(e) => write!(f, "Internal Error:: {:?}", e),
            Error::Io(e) => write!(f, "Io error {:?}", e),
            Error::Db(e) => write!(f, "Database error {:?}", e),
            Error::NotSupport(e) => write!(f, "Not support: {}", e),
            Error::Unknown(e) => write!(f, "a unknown error happened: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Db(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<redis::RedisError> for Error {
    fn from(e: redis::RedisError) -> Self {
        Self::Db(e)
    }
}
