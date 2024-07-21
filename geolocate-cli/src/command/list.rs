use std::collections::HashMap;
use std::fmt::Display;
use std::num::NonZeroUsize;

use anyhow::{anyhow, Result};
use clap::Args;
use geolocate_core::ip::{Address, IpAddrBlock};
use geolocate_core::prelude::*;

use crate::map::MaybeCountry;
use crate::{Ipv4CountryMap, Ipv6CountryMap};

/// The arguments for the 'list' command.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Args)]
pub struct Arguments {
    /// Only display the country with this name, alpha-2 code, or numeric code.
    pub country: Option<Box<str>>,
    /// Only display the specified number of countries.
    #[arg(short = 'c', long = "country-limit")]
    pub country_limit: Option<NonZeroUsize>,
    /// Only display the specified number of addresses.
    #[arg(short = 'a', long = "address-limit")]
    pub address_limit: Option<NonZeroUsize>,
    /// Display IPv4 address blocks.
    #[arg(short = '4', long = "ipv4", required_if_eq("display_ipv6", "false"))]
    pub display_ipv4: bool,
    /// Display IPv6 address blocks.
    #[arg(short = '6', long = "ipv6", required_if_eq("display_ipv4", "false"))]
    pub display_ipv6: bool,
}

/// A country filter for the list command.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Filter<'c> {
    /// Filters for a specific country.
    Country(&'c Country),
    /// Filters for a country with the given name.
    Name(Box<str>),
    /// Filters for a country with the given alpha-2 code.
    Code(CountryCode),
    /// Filters for a country with the given numeric code.
    Numeric(u16),
}

impl Filter<'_> {
    /// Checks whether the given country matches this filter.
    #[must_use]
    pub fn test(&self, country: &Country) -> bool {
        match self {
            Self::Country(c) => country == *c,
            Self::Name(name) => &country.name == name,
            Self::Code(code) => &country.code == code,
            Self::Numeric(numeric) => &country.numeric == numeric,
        }
    }

    /// Checks whether the given country matches this filter, returning [`None`] if it is not possible to test.
    #[must_use]
    pub fn test_maybe(&self, country: &MaybeCountry) -> Option<bool> {
        match (country, self) {
            (MaybeCountry::Present(country), _) => Some(self.test(country)),
            (MaybeCountry::Missing(code_a), Self::Code(code_b)) => Some(code_a == code_b),
            _ => None,
        }
    }
}

impl From<&str> for Filter<'_> {
    fn from(value: &str) -> Self {
        if let Ok(numeric) = value.parse() {
            return Filter::Numeric(numeric);
        }
        if let Ok(code) = value.parse() {
            return Filter::Code(code);
        }

        Filter::Name(value.into())
    }
}

/// Runs the 'list' command.
///
/// # Errors
///
/// This function will return an error if the command failed to execute.
pub fn run_command<'c>(
    Arguments { country, country_limit, address_limit, display_ipv4, display_ipv6 }: Arguments,
    ipv4_map: &Ipv4CountryMap,
    ipv6_map: &Ipv6CountryMap,
    country_iter: impl Iterator<Item = &'c Country>,
) -> Result<()> {
    let mut countries: Box<[_]> = if let Some(filter) = country.as_deref().map(Filter::from) {
        let country = self::find_country(&filter, country_iter)?;
        let ipv4_blocks = display_ipv4.then(|| self::collect_blocks(Some(&filter), ipv4_map.entries()));
        let ipv6_blocks = display_ipv6.then(|| self::collect_blocks(Some(&filter), ipv6_map.entries()));

        Box::new([(MaybeCountry::Present(country), ipv4_blocks.unwrap_or_default(), ipv6_blocks.unwrap_or_default())])
    } else {
        let mut countries: HashMap<_, (Vec<_>, Vec<_>)> = HashMap::new();

        if display_ipv4 {
            for (address_block, country) in ipv4_map.entries() {
                countries.entry(country.clone()).or_default().0.push(*address_block);
            }
        }

        if display_ipv6 {
            for (address_block, country) in ipv6_map.entries() {
                countries.entry(country.clone()).or_default().1.push(*address_block);
            }
        }

        countries.into_iter().map(|(c, (v4, v6))| (c, v4.into_boxed_slice(), v6.into_boxed_slice())).collect()
    };

    countries.sort_unstable_by_key(|(c, ..)| match c {
        MaybeCountry::Present(country) => country.code,
        MaybeCountry::Missing(code) => *code,
    });

    let country_limit = country_limit.map_or(countries.len(), NonZeroUsize::get);

    for (country, ipv4_blocks, ipv6_blocks) in countries.iter_mut().take(country_limit) {
        if ipv4_blocks.is_empty() && ipv6_blocks.is_empty() {
            continue;
        }

        println!("{country}");

        ipv6_blocks.sort_unstable();

        if display_ipv4 {
            ipv4_blocks.sort_unstable();

            let limit = address_limit.map_or(ipv4_blocks.len(), NonZeroUsize::get);

            println!("\nIPv4:\n    {}", self::blocks_display(limit, ipv4_blocks.iter()));
        }

        if display_ipv6 {
            ipv6_blocks.sort_unstable();

            let limit = address_limit.map_or(ipv6_blocks.len(), NonZeroUsize::get);

            println!("\nIPv6:\n    {}", self::blocks_display(limit, ipv6_blocks.iter()));
        }

        println!();
    }

    Ok(())
}

/// Attempts to find a country using the given filter.
///
/// # Errors
///
/// This function will return an error if the country could not be found.
fn find_country<'c>(filter: &Filter, mut iter: impl Iterator<Item = &'c Country>) -> Result<Country> {
    let country = iter.find(|c| filter.test(c)).ok_or_else(|| match filter {
        Filter::Country(country) => anyhow!("unable to find country '{}'", country.name),
        Filter::Name(name) => anyhow!("unable to find country '{name}'"),
        Filter::Code(code) => anyhow!("unable to find country '{code}'"),
        Filter::Numeric(numeric) => anyhow!("unable to find country #{numeric}"),
    });

    country.cloned()
}

/// Collects IP address blocks from the given iterator into a list.
fn collect_blocks<'i, 'f, A, I>(filter: Option<&Filter<'f>>, iter: I) -> Box<[IpAddrBlock<A>]>
where
    'i: 'f,
    A: Address + 'i,
    I: Iterator<Item = (&'i IpAddrBlock<A>, &'i MaybeCountry)>,
{
    let iter: Box<dyn Iterator<Item = _>> = if let Some(filter) = filter {
        Box::from(iter.filter(|(_, c)| filter.test_maybe(c).unwrap_or(false)))
    } else {
        Box::from(iter)
    };

    iter.map(|(b, _)| *b).collect()
}

/// Returns a display implementation for the given address block list.
fn blocks_display<'b, A, I>(limit: usize, blocks: I) -> impl Display
where
    A: Address + Display + 'b,
    I: Iterator<Item = &'b IpAddrBlock<A>>,
{
    blocks
        .take(limit)
        .map(|b| format!("{} .. {}", b.start(), b.end()))
        .intersperse("\n    ".to_string())
        .collect::<Box<str>>()
}
