#![cfg_attr(not(debug_assertions), deny(clippy::expect_used, clippy::panic, clippy::unwrap_used))]
#![cfg_attr(not(debug_assertions), warn(missing_docs))]
#![cfg_attr(debug_assertions, warn(clippy::expect_used, clippy::panic, clippy::unwrap_used))]
#![warn(clippy::nursery, clippy::pedantic, clippy::todo)]
#![allow(clippy::module_name_repetitions)]

/// Defines countries and their API.
pub mod country;
/// Defines the IPv4 and IPv6 block API.
#[allow(private_bounds)]
pub mod ip;

/// The library's default import prelude.
pub mod prelude {
    pub use crate::country::{Country, CountryCode};
    pub use crate::ip::v4::Ipv4AddrBlock;
    pub use crate::ip::v6::Ipv6AddrBlock;
}
