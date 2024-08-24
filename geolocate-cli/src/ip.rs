use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use serde::de::{Unexpected, Visitor};
use serde::Deserializer;

/// Deserializes an IPv4 address.
///
/// # Errors
///
/// This function will return an error if the value cannot be deserialized.
pub fn deserialize_ipv4<'de, D>(deserializer: D) -> Result<Ipv4Addr, D::Error>
where
    D: Deserializer<'de>,
{
    struct Ipv4Visitor;

    impl Visitor<'_> for Ipv4Visitor {
        type Value = Ipv4Addr;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a valid ipv4 address")
        }

        fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Ipv4Addr::from_bits(v))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ipv4Addr::from_str(v).map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
    }

    deserializer.deserialize_u32(Ipv4Visitor)
}

/// Deserializes an IPv6 address.
///
/// # Errors
///
/// This function will return an error if the value cannot be deserialized.
pub fn deserialize_ipv6<'de, D>(deserializer: D) -> Result<Ipv6Addr, D::Error>
where
    D: Deserializer<'de>,
{
    struct Ipv6Visitor;

    impl Visitor<'_> for Ipv6Visitor {
        type Value = Ipv6Addr;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a valid ipv6 address")
        }

        fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Ipv6Addr::from_bits(v))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ipv6Addr::from_str(v).map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
    }

    deserializer.deserialize_str(Ipv6Visitor)
}
