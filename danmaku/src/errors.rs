use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to connect WebSocket server: {0}")]
    WebSocketConnectError(String),
    #[error(transparent)]
    InvalidKeyIvLength(#[from] block_modes::InvalidKeyIvLength),
    #[error(transparent)]
    DecryptAesError(#[from] block_modes::BlockModeError),
    #[error("the cipher text was too short and its length was {0}")]
    CipherTextTooShort(usize),
    #[error(transparent)]
    DecodeBase64Error(#[from] base64::DecodeError),
    #[error(transparent)]
    EncodeProtoError(#[from] prost::EncodeError),
    #[error(transparent)]
    DecodeProtoError(#[from] prost::DecodeError),
    #[error("the length of the ProtoBuf data received from the WebSocket server was wrong: {0} part, expected length {1}, actual length {2}")]
    ProtoDataLengthError(&'static str, usize, usize),
    #[error(transparent)]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("index {1} in {0} was out of range")]
    IndexOutOfRange(&'static str, usize),
    #[error("invalid danmaku token")]
    InvalidToken,
    #[error(transparent)]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error(transparent)]
    TryFromIntError(#[from] std::num::TryFromIntError),
    #[error("no session key")]
    NoSessionKey,
    #[error("failed to register in the danmaku server")]
    RegisterError,

    #[cfg(feature = "api")]
    #[error(transparent)]
    AcFunLiveApiError(#[from] acfunliveapi::Error),

    #[cfg(feature = "_serde")]
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    #[cfg(feature = "default_ws_client")]
    #[error(transparent)]
    TungsteniteError(#[from] async_tungstenite::tungstenite::Error),
    #[cfg(feature = "default_ws_client")]
    #[error("it was timeout for the WebSocket client to connect the server")]
    WsConnectTimeout,
}
