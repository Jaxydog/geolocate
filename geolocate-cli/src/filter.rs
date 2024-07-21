use anyhow::{anyhow, Result};
use geolocate_core::country::{Country, CountryCode};

use crate::map::MaybeCountry;

/// A country filter for usage in commands.
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

/// Attempts to find a country using the given filter.
///
/// # Errors
///
/// This function will return an error if the country could not be found.
pub fn find_country<'c>(filter: &Filter, mut iter: impl Iterator<Item = &'c Country>) -> Result<Country> {
    let country = iter.find(|c| filter.test(c)).ok_or_else(|| match filter {
        Filter::Country(country) => anyhow!("unable to find country '{}'", country.name),
        Filter::Name(name) => anyhow!("unable to find country '{name}'"),
        Filter::Code(code) => anyhow!("unable to find country '{code}'"),
        Filter::Numeric(numeric) => anyhow!("unable to find country #{numeric}"),
    });

    country.cloned()
}
