use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("the WebSocket client failed to connect the server")]
    WsConnectError(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("the WebSocket client failed to read a message from the server")]
    WsReadError(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("the WebSocket client failed to send a message to the server")]
    WsWriteError(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("the WebSocket client failed to close the connection")]
    WsCloseError(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("the WebSocket connection was closed")]
    WsClosed,
    #[error("it was timeout for the WebSocket client to read a message")]
    WsReadTimeout,
    #[error("it was timeout for the WebSocket client to send a message")]
    WsWriteTimeout,
    #[error("it was timeout for the WebSocket client to close the connection")]
    WsCloseTimeout,
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
    SendMpscError(#[from] futures::channel::mpsc::SendError),
    #[error("getting danmaku was stopped: {0}")]
    StopDanmaku(&'static str),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("index {1} in {0} was out of range")]
    IndexOutOfRange(&'static str, usize),
    #[error("failed to send danmaku")]
    SendDanmakuError,
    #[error("failed to send a message through the oneshot channel")]
    SendOneshotError,
    #[error(transparent)]
    SenderCancelError(#[from] futures::channel::oneshot::Canceled),
    #[error("invalid danmaku token")]
    InvalidToken,
    #[error(transparent)]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error(transparent)]
    TryFromIntError(#[from] std::num::TryFromIntError),

    #[cfg(feature = "api")]
    #[error(transparent)]
    AcFunLiveApiError(#[from] acfunliveapi::Error),
}
