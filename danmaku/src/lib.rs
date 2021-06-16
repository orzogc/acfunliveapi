#![forbid(unsafe_code)]

pub mod client;
pub mod danmaku;
pub mod websocket;

mod errors;
mod global;
mod proto;

pub use errors::*;
pub use proto::acproto;

#[cfg(feature = "api")]
pub use acfunliveapi;
