use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::Path;

use anyhow::Result;
use clap::Args;
use geolocate_core::ip::{Address, IpAddrBlock};
use geolocate_core::prelude::*;

use crate::filter::Filter;
use crate::map::MaybeCountry;

/// The arguments for the 'count' command.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Args)]
pub struct Arguments {
    /// Only display the country with this name, alpha-2 code, or numeric code.
    pub country: Option<Filter<'static>>,
    /// Only display the specified number of countries. Does nothing when searching for a specific country.
    #[arg(short = 'c', long = "country-limit")]
    pub limit: Option<NonZeroUsize>,
    /// Display IPv4 address blocks.
    #[arg(short = '4', long = "ipv4", required_if_eq("display_ipv6", "false"))]
    pub display_ipv4: bool,
    /// Display IPv6 address blocks.
    #[arg(short = '6', long = "ipv6", required_if_eq("display_ipv4", "false"))]
    pub display_ipv6: bool,
}

/// Runs the 'count' command.
///
/// # Errors
///
/// This function will return an error if the command failed to execute.
pub fn run<'c>(
    Arguments { country, limit, display_ipv4, display_ipv6 }: Arguments,
    ipv4_source: &Path,
    ipv6_source: &Path,
    resolve: impl Fn(CountryCode) -> Option<Country> + Copy,
    country_iter: impl Iterator<Item = &'c Country>,
) -> Result<()> {
    let ipv4_map = crate::map::parse_ipv4_map_file(ipv4_source, None, resolve)?;
    let ipv6_map = crate::map::parse_ipv6_map_file(ipv6_source, None, resolve)?;

    let mut countries: Box<[_]> = if let Some(filter) = country {
        let country = crate::filter::find_country(&filter, country_iter)?;
        let ipv4_blocks = display_ipv4.then(|| self::count_blocks(&filter, ipv4_map.iter()));
        let ipv6_blocks = display_ipv6.then(|| self::count_blocks(&filter, ipv6_map.iter()));

        Box::new([(MaybeCountry::Present(country), ipv4_blocks.unwrap_or_default(), ipv6_blocks.unwrap_or_default())])
    } else {
        let mut countries = HashMap::<MaybeCountry, (usize, usize)>::new();

        if display_ipv4 {
            for (_, country) in ipv4_map.iter() {
                countries.entry(country.clone()).or_default().0 += 1;
            }
        }

        if display_ipv6 {
            for (_, country) in ipv6_map.iter() {
                countries.entry(country.clone()).or_default().1 += 1;
            }
        }

        countries.into_iter().map(|(c, (v4, v6))| (c, v4, v6)).collect()
    };

    countries.sort_unstable_by_key(|(c, ..)| match c {
        MaybeCountry::Present(country) => country.code,
        MaybeCountry::Missing(code) => *code,
    });

    let limit = limit.map_or(countries.len(), NonZeroUsize::get);

    for (country, ipv4_blocks, ipv6_blocks) in countries.iter_mut().take(limit) {
        println!("{country}");

        if display_ipv4 {
            println!("IPv4: {ipv4_blocks}");
        }
        if display_ipv6 {
            println!("IPv6: {ipv6_blocks}");
        }

        println!();
    }

    Ok(())
}

/// Counts the total number of blocks in the given filtered iterator.
pub fn count_blocks<'i, 'f, A, I>(filter: &Filter<'_>, iter: I) -> usize
where
    'i: 'f,
    A: Address + 'i,
    I: Iterator<Item = (&'i IpAddrBlock<A>, &'i MaybeCountry)>,
{
    iter.filter(|(_, c)| filter.test_maybe(c).unwrap_or(false)).count()
}
