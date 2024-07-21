//! A CLI for geolocate-core that allows for the resolution of IP addresses to their countries.
#![cfg_attr(not(debug_assertions), deny(clippy::unwrap_used))]
#![cfg_attr(not(debug_assertions), warn(missing_docs))]
#![cfg_attr(debug_assertions, warn(clippy::unwrap_used))]
#![warn(clippy::nursery, clippy::pedantic, clippy::todo)]
#![allow(clippy::module_name_repetitions)]
#![feature(iter_intersperse)]

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use geolocate_core::prelude::{Country, CountryCode, Ipv4AddrBlockMap, Ipv6AddrBlockMap};
use map::MaybeCountry;

/// Provides IP address deserializers.
pub mod ip;
/// Provides IP-block-map deserializers.
pub mod map;
/// Provides implementations for each command.
pub mod command {
    /// The list command.
    pub mod list;
}

/// A map containing IPv4 address blocks and their associated countries.
pub type Ipv4CountryMap = Ipv4AddrBlockMap<MaybeCountry>;
/// A map containing IPv6 address blocks and their associated countries.
pub type Ipv6CountryMap = Ipv6AddrBlockMap<MaybeCountry>;

/// The application's command-line arguments.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Parser)]
#[command(about, author, version, long_about = None)]
pub struct Arguments {
    /// The file to source country-to-IPv4 address data from.
    #[arg(short = '4', long = "ipv4-source-data", default_value = "/usr/share/tor/geoip")]
    pub ipv4_source: Box<Path>,
    /// The file to source country-to-IPv6 address data from.
    #[arg(short = '6', long = "ipv6-source-data", default_value = "/usr/share/tor/geoip6")]
    pub ipv6_source: Box<Path>,
    /// The file to source country data from.
    #[arg(short = 'c', long = "country-source-data", default_value = "./data/countries.json")]
    pub country_source: Box<Path>,

    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// The application's sub-commands.
#[non_exhaustive]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Subcommand)]
#[command(about, author, long_about = None)]
pub enum Command {
    /// Tallies the number of IP addresses assigned per country.
    Count {
        /// Only display the specified number of countries.
        #[arg(short = 'l', long = "limit")]
        limit: Option<usize>,
        /// Only count IP addresses in the IPv4 format.
        #[arg(short = '4', long = "v4")]
        v4: bool,
        /// Only count IP addresses in the IPv6 format.
        #[arg(short = '6', long = "v6")]
        v6: bool,
    },
    /// Lists all IP address blocks and their assigned country.
    List(crate::command::list::Arguments),
    /// Resolves a single IP address' country of origin.
    Resolve {
        /// The IP address to resolve.
        address: IpAddr,
        /// Flag that the IP address is of the IPv4 format.
        #[arg(short = '4', long = "v4")]
        v4: bool,
        /// Flag that the IP address is of the IPv6 format.
        #[arg(short = '6', long = "v6")]
        v6: bool,
        /// Output the country's name.
        #[arg(short = 'n', long = "name")]
        name: bool,
        /// Only output the country's Alpha-2 code.
        #[arg(short = 'a', long = "alpha-2")]
        alpha_2: bool,
        /// Only output the country's numeric code.
        #[arg(short = 'N', long = "numeric")]
        numeric: bool,
    },
}

/// The application's entrypoint.
///
/// # Errors
///
/// This function will return an error if the program fails to run.
pub fn main() -> Result<()> {
    let arguments = Arguments::parse();

    if !std::fs::exists(&arguments.ipv4_source)? {
        bail!("unable to locate file '{}'", arguments.ipv4_source.to_string_lossy());
    }
    if !std::fs::exists(&arguments.ipv6_source)? {
        bail!("unable to locate file '{}'", arguments.ipv6_source.to_string_lossy());
    }
    if !std::fs::exists(&arguments.country_source)? {
        bail!("unable to locate file '{}'", arguments.country_source.to_string_lossy());
    }

    let file = std::fs::File::open(&arguments.country_source)?;
    let countries: Box<[Country]> = serde_json::from_reader(file)?;
    let countries: HashMap<CountryCode, Country> = countries.iter().map(|c| (c.code, c.clone())).collect();
    let resolve = |code: CountryCode| -> Option<Country> { countries.get(&code).cloned() };

    let ipv4_map = crate::map::parse_ipv4_map_file(&arguments.ipv4_source, None, resolve)?;
    let ipv6_map = crate::map::parse_ipv6_map_file(&arguments.ipv6_source, None, resolve)?;

    match arguments.command {
        ref command @ Command::Count { .. } => crate::count(command, &ipv4_map, &ipv6_map),
        Command::List(arguments) => command::list::run_command(arguments, &ipv4_map, &ipv6_map, countries.values()),
        ref command @ Command::Resolve { .. } => crate::resolve(command, &ipv4_map, &ipv6_map),
    }
}

/// Counts the mapped countries.
///
/// # Errors
///
/// This function will return an error if the given command contains invalid arguments.
pub fn count(
    command: &Command,
    ipv4: &Ipv4AddrBlockMap<MaybeCountry>,
    ipv6: &Ipv6AddrBlockMap<MaybeCountry>,
) -> Result<()> {
    let Command::Count { limit, v4, v6 } = command else {
        bail!("invalid command type");
    };
    if (!v4 && !v6) || (*v4 && *v6) {
        bail!("one of `v4` or `v6` must be enabled");
    }

    if limit.is_some_and(|n| n == 0) {
        return Ok(());
    }

    let mut countries = HashMap::<MaybeCountry, usize>::new();

    if *v4 {
        for country in ipv4.values().cloned() {
            *countries.entry(country).or_default() += 1;
        }
    } else {
        for country in ipv6.values().cloned() {
            *countries.entry(country).or_default() += 1;
        }
    }

    let mut sorted = countries.into_iter().collect::<Box<[_]>>();

    sorted.sort_unstable_by_key(|(_, n)| *n);
    sorted.reverse();

    if let Some(limit) = *limit {
        for (country, count) in sorted.iter().take(limit) {
            println!("{country}: {count}");
        }
    } else {
        for (country, count) in &sorted {
            println!("{country}: {count}");
        }
    }

    Ok(())
}

/// Resolves an IP address.
///
/// # Errors
///
/// This function will return an error if the given command contains invalid arguments.
pub fn resolve(
    command: &Command,
    ipv4: &Ipv4AddrBlockMap<MaybeCountry>,
    ipv6: &Ipv6AddrBlockMap<MaybeCountry>,
) -> Result<()> {
    let Command::Resolve { address, v4, v6, name, alpha_2, numeric } = command else {
        bail!("invalid command type");
    };

    if !name && !alpha_2 && !numeric {
        return Ok(());
    }

    let Some(country) = (match (*address, v4, v6) {
        (IpAddr::V4(address), true, _) | (IpAddr::V4(address), false, false) => ipv4.get_from_address(address),
        (IpAddr::V6(address), _, true) | (IpAddr::V6(address), false, false) => ipv6.get_from_address(address),
        _ => bail!("invalid command arguments"),
    }) else {
        bail!("unmapped address provided");
    };

    match country {
        MaybeCountry::Missing(code) => {
            if *name {
                eprintln!("missing country name for alpha-2 code '{code}'");
            }
            if *alpha_2 {
                println!("{code}");
            }
            if *numeric {
                eprintln!("missing numeric code for alpha-2 code '{code}'");
            }
        }
        MaybeCountry::Present(country) => {
            if *name {
                println!("{}", country.name);
            }
            if *alpha_2 {
                println!("{}", country.code);
            }
            if *numeric {
                println!("{}", country.numeric);
            }
        }
    }

    Ok(())
}
