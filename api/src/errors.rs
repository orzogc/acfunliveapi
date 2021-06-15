use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to get device ID cookie")]
    GetDidFailed,
    #[error("invalid uid: {0}")]
    InvalidUid(i64),
    #[error("login was needed")]
    NoLogin,
    #[error("index {1} in {0} was out of range")]
    IndexOutOfRange(&'static str, usize),
    #[error("live ID was empty")]
    EmptyLiveId,
    #[error(transparent)]
    BuildClientFailed(#[from] reqwest::Error),
    #[error(transparent)]
    ParseUrlError(#[from] pretend::resolver::ParseError),
    #[error(transparent)]
    PretendError(#[from] pretend::Error),
    #[error(transparent)]
    ParseCookieError(#[from] cookie::ParseError),
    #[error(transparent)]
    HeaderToStrError(#[from] pretend::http::header::ToStrError),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}
