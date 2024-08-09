//! The core library for SzurubooruClient
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

/// Core client module
pub mod client;
pub use client::SzurubooruClient;
pub use client::SzurubooruRequest;

pub mod errors;
pub use errors::SzurubooruResult;
pub mod models;

#[cfg(test)]
mod tests;
pub mod tokens;
