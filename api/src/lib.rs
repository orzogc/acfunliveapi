#![forbid(unsafe_code)]

pub mod client;
pub mod response;

mod acfun;
mod errors;
mod kuaishou;
mod rest;

pub use errors::*;
pub use rest::*;

pub use pretend;
