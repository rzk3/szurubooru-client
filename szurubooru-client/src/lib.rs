//! SzurubooruClient is a wrapper around the excellently-documented Szurubooru API,
//! including type-safe (if not API-safe) Query and Sort tokens.
//!
//! # Creating a new client
//!
//! ## Basic authentication
//! Please keep in mind that this is not the preferred method of authentication. Tokens
//! are far superior.
//!
//! ```rust,no_run
//! use szurubooru_client::SzurubooruClient;
//! let client = SzurubooruClient::new_with_basic_auth("http://localhost:5001", "myuser",
//!     "mypassword", true).unwrap();
//! ```
//!
//! ## Token authentication
//! The far superior and more secure means of authentication
//!
//! ```rust,no_run
//! use szurubooru_client::SzurubooruClient;
//! let client = SzurubooruClient::new_with_token("http://localhost:5001", "myuser", "sz-123456", true).unwrap();
//! ```
//!
//! For all other methods for making the requests, see the documentation.

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
