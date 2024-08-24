use std::fmt::Display;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use geolocate_core::country::{Country, CountryCode};
use geolocate_core::ip::{Address, IpAddrBlock, IpAddrBlockMap};
use geolocate_core::prelude::{Ipv4AddrBlock, Ipv4AddrBlockMap, Ipv6AddrBlock, Ipv6AddrBlockMap};
use serde::Deserialize;

/// A country that could potentially be unresolved.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum MaybeCountry {
    /// The country is present.
    Present(Country),
    /// The country is missing.
    Missing(CountryCode),
}

impl Display for MaybeCountry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Present(country) => write!(f, "{}", country.name),
            Self::Missing(code) => write!(f, "{code}"),
        }
    }
}

/// The format to use when deserializing an IPv4 map file's entry.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize)]
pub struct Ipv4Schema {
    /// The starting address.
    #[serde(deserialize_with = "crate::ip::deserialize_ipv4")]
    pub start: Ipv4Addr,
    /// The ending address.
    #[serde(deserialize_with = "crate::ip::deserialize_ipv4")]
    pub end: Ipv4Addr,
    /// A country's Alpha-2 code.
    pub country: Box<str>,
}

/// The format to use when deserializing an IPv6 map file's entry.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize)]
pub struct Ipv6Schema {
    /// The starting address.
    #[serde(deserialize_with = "crate::ip::deserialize_ipv6")]
    pub start: Ipv6Addr,
    /// The ending address.
    #[serde(deserialize_with = "crate::ip::deserialize_ipv6")]
    pub end: Ipv6Addr,
    /// A country's Alpha-2 code.
    pub country: Box<str>,
}

/// Attempts to parse an IPv4 map file.
///
/// # Errors
///
/// This function will return an error if the file could not be parsed.
#[inline]
pub fn parse_ipv4_map_file<P, F>(path: P, capacity: Option<usize>, resolve: F) -> Result<Ipv4AddrBlockMap<MaybeCountry>>
where
    P: AsRef<Path>,
    F: Fn(CountryCode) -> Option<Country>,
{
    self::parse_ip_map(path, capacity, resolve, |Ipv4Schema { start, end, country }| {
        let block = Ipv4AddrBlock::try_new(start, end)?;
        let code = CountryCode::from_str(&country)?;

        Ok((block, code))
    })
}

/// Attempts to parse an IPv6 map file.
///
/// # Errors
///
/// This function will return an error if the file could not be parsed.
#[inline]
pub fn parse_ipv6_map_file<P, F>(path: P, capacity: Option<usize>, resolve: F) -> Result<Ipv6AddrBlockMap<MaybeCountry>>
where
    P: AsRef<Path>,
    F: Fn(CountryCode) -> Option<Country>,
{
    self::parse_ip_map(path, capacity, resolve, |Ipv6Schema { start, end, country }| {
        let block = Ipv6AddrBlock::try_new(start, end)?;
        let code = CountryCode::from_str(&country)?;

        Ok((block, code))
    })
}

/// Attempts to parse an IP map file.
///
/// # Errors
///
/// This function will return an error if the file could not be parsed.
pub fn parse_ip_map<A, P, R, F, T>(
    path: P,
    capacity: Option<usize>,
    resolve: R,
    compute: F,
) -> Result<IpAddrBlockMap<A, MaybeCountry>>
where
    A: Address + for<'de> Deserialize<'de>,
    P: AsRef<Path>,
    R: Fn(CountryCode) -> Option<Country>,
    F: Fn(T) -> Result<(IpAddrBlock<A>, CountryCode)>,
    T: for<'de> Deserialize<'de>,
{
    const DEFAULT_CAPACITY: usize = 256;

    let file = std::fs::File::open(path)?;
    let reader = csv::ReaderBuilder::new().has_headers(false).comment(Some(b'#')).from_reader(file);
    let mut map = IpAddrBlockMap::with_capacity(capacity.unwrap_or(DEFAULT_CAPACITY));

    for entry in reader.into_deserialize() {
        let (block, code) = compute(entry?)?;
        let country = resolve(code).map_or(MaybeCountry::Missing(code), MaybeCountry::Present);

        map.insert_unstable(block, country);
    }

    map.normalize();

    Ok(map)
}
