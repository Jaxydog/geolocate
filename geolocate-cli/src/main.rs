//! A CLI for geolocate-core that allows for the resolution of IP addresses to their countries.
#![cfg_attr(not(debug_assertions), deny(clippy::unwrap_used))]
#![cfg_attr(not(debug_assertions), warn(missing_docs))]
#![cfg_attr(debug_assertions, warn(clippy::unwrap_used))]
#![warn(clippy::nursery, clippy::pedantic, clippy::todo)]
#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use geolocate_core::country::{Country, CountryCode};
use geolocate_core::prelude::{Ipv4AddrBlockMap, Ipv6AddrBlockMap};
use map::MaybeCountry;

/// Provides IP address deserializers.
pub mod ip;
/// Provides IP-block-map deserializers.
pub mod map;

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
    /// Resolves a single IP address' country of origin.
    /// By default, this output's the country's English name.
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
        command @ Command::Resolve { .. } => crate::resolve(command, &ipv4_map, &ipv6_map),
    }
}

/// Resolves an IP address.
///
/// # Errors
///
/// This function will return an error if the functions arguments are invalid or contradicting.
pub fn resolve(
    Command::Resolve { address, v4, v6, name, alpha_2, numeric }: Command,
    ipv4: &Ipv4AddrBlockMap<MaybeCountry>,
    ipv6: &Ipv6AddrBlockMap<MaybeCountry>,
) -> Result<()> {
    if !name && !alpha_2 && !numeric {
        return Ok(());
    }

    let Some(country) = (match (address, v4, v6) {
        (IpAddr::V4(address), true, _) | (IpAddr::V4(address), false, false) => ipv4.get_from_address(address),
        (IpAddr::V6(address), _, true) | (IpAddr::V6(address), false, false) => ipv6.get_from_address(address),
        _ => bail!("invalid command arguments"),
    }) else {
        bail!("unmapped address provided");
    };

    match country {
        MaybeCountry::Missing(code) => {
            if name {
                eprintln!("missing country name for alpha-2 code '{code}'");
            }
            if alpha_2 {
                println!("{code}");
            }
            if numeric {
                eprintln!("missing numeric code for alpha-2 code '{code}'");
            }
        }
        MaybeCountry::Present(country) => {
            if name {
                println!("{}", country.name);
            }
            if alpha_2 {
                println!("{}", country.code);
            }
            if numeric {
                println!("{}", country.numeric);
            }
        }
    }

    Ok(())
}
