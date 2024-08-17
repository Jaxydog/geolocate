//! A CLI for geolocate-core that allows for the resolution of IP addresses to their countries.
#![cfg_attr(not(debug_assertions), deny(clippy::unwrap_used))]
#![cfg_attr(not(debug_assertions), warn(missing_docs))]
#![cfg_attr(debug_assertions, warn(clippy::unwrap_used))]
#![warn(clippy::nursery, clippy::pedantic, clippy::todo)]
#![allow(clippy::module_name_repetitions)]
#![feature(iter_intersperse)]

use std::collections::HashMap;
use std::path::Path;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use geolocate_core::prelude::{Country, CountryCode, Ipv4AddrBlockMap, Ipv6AddrBlockMap};
use map::MaybeCountry;

/// Provides country filtering for commands.
pub mod filter;
/// Provides IP address deserializers.
pub mod ip;
/// Provides IP-block-map deserializers.
pub mod map;

/// Provides implementations for each command.
pub mod command {
    /// The count command.
    pub mod count;
    /// The list command.
    pub mod list;
    /// The resolve command.
    pub mod resolve;
}

/// A map containing IPv4 address blocks and their associated countries.
pub type Ipv4CountryMap = Ipv4AddrBlockMap<MaybeCountry>;
/// A map containing IPv6 address blocks and their associated countries.
pub type Ipv6CountryMap = Ipv6AddrBlockMap<MaybeCountry>;

/// The application's command-line arguments.
#[non_exhaustive]
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
    Count(crate::command::count::Arguments),
    /// Lists all IP address blocks and their assigned country.
    List(crate::command::list::Arguments),
    /// Resolves a single IP address' country of origin.
    Resolve(crate::command::resolve::Arguments),
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

    match arguments.command {
        Command::Count(command_arguments) => crate::command::count::run(
            command_arguments,
            &arguments.ipv4_source,
            &arguments.ipv6_source,
            resolve,
            countries.values(),
        ),
        Command::List(command_arguments) => crate::command::list::run(
            command_arguments,
            &arguments.ipv4_source,
            &arguments.ipv6_source,
            resolve,
            countries.values(),
        ),
        Command::Resolve(command_arguments) => {
            crate::command::resolve::run(command_arguments, &arguments.ipv4_source, &arguments.ipv6_source, resolve)
        }
    }
}
