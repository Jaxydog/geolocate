use std::net::IpAddr;

use anyhow::{bail, Result};
use clap::Args;

use crate::map::MaybeCountry;
use crate::{Ipv4CountryMap, Ipv6CountryMap};

/// The arguments for the 'count' command.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Args)]
pub struct Arguments {
    /// The IP address to resolve.
    pub address: IpAddr,
    /// Output the country's name. This is enabled by default if no arguments are provided.
    #[arg(short = 'n', long = "name")]
    pub name: bool,
    /// Output the country's alpha-2 code.
    #[arg(short = 'a', long = "alpha-2")]
    pub code: bool,
    /// Output the country's numeric code.
    #[arg(short = 'N', long = "numeric")]
    pub numeric: bool,
}

/// Runs the 'resolve' command.
///
/// # Errors
///
/// This function will return an error if the command failed to execute.
pub fn run(
    Arguments { address, mut name, code, numeric }: Arguments,
    ipv4_map: &Ipv4CountryMap,
    ipv6_map: &Ipv6CountryMap,
) -> Result<()> {
    if !name && !code && !numeric {
        name = true;
    }

    let Some(country) = (match address {
        IpAddr::V4(ip) => ipv4_map.get_from_address(ip),
        IpAddr::V6(ip) => ipv6_map.get_from_address(ip),
    }) else {
        bail!("the given ip address is unmapped");
    };

    match country {
        MaybeCountry::Present(country) => {
            if name {
                println!("Country: {}", country.name);
            }
            if code {
                println!("Alpha-2: {}", country.code);
            }
            if numeric {
                println!("Numeric: {}", country.numeric);
            }
        }
        MaybeCountry::Missing(country_code) => {
            if name {
                println!("Country: N/A");
            }
            if code {
                println!("Alpha-2: {country_code}");
            }
            if numeric {
                println!("Numeric: N/A");
            }
        }
    }

    Ok(())
}
