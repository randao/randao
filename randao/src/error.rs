use std::fmt::Formatter;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CheckChainErr,
    GetNumCampaignsErr,
    CheckCampaignsInfoErr,
    TxInternalErr(InternalError),
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
            Error::CheckChainErr => write!(f, "chain id check failed"),
            Error::GetNumCampaignsErr => write!(f, "Get numCampaigns faild"),
            Error::CheckCampaignsInfoErr => write!(f, "Check campaigns info faild"),
            Error::TxInternalErr(e) => write!(f, "Internal Error:: {:?}", e),
            Error::Unknown(e) => write!(f, "a unknown error happened: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            _ => None,
        }
    }
}
