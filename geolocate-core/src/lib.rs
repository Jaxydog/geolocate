#![cfg_attr(not(debug_assertions), deny(clippy::expect_used, clippy::panic, clippy::unwrap_used))]
#![cfg_attr(not(debug_assertions), warn(missing_docs))]
#![cfg_attr(debug_assertions, warn(clippy::expect_used, clippy::panic, clippy::unwrap_used))]
#![warn(clippy::nursery, clippy::pedantic, clippy::todo)]
#![allow(clippy::module_name_repetitions)]

/// Defines countries and their API.
pub mod country;

/// The library's default import prelude.
pub mod prelude {
    pub use crate::country::{Country, CountryCode};
}
