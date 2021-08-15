use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to get device ID cookie")]
    GetDidFailed,
    #[error("invalid uid: {0}")]
    InvalidUid(i64),
    #[error("visitor or user login was needed")]
    VisitorOrUserNotLogin,
    #[error("index {1} in {0} was out of range")]
    IndexOutOfRange(&'static str, usize),
    #[error("live ID was empty")]
    EmptyLiveId,
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
    #[error("user login was needed")]
    NotUser,
    #[error("the liver's uid was not set")]
    NotSetLiverUid,

    #[cfg(feature = "default_http_client")]
    #[error(transparent)]
    BuildClientFailed(#[from] reqwest::Error),
}
