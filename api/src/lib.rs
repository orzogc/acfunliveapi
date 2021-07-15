#![forbid(unsafe_code)]

pub mod client;
pub mod response;

#[cfg(feature = "default_http_client")]
pub mod http;

mod acfun;
mod errors;
mod kuaishou;
mod rest;

pub use errors::*;
pub use rest::*;

pub use pretend;
