use std::str::FromStr;

use geolocate_core::country::{Country, CountryCode};
use mediawiki::ApiSync;
use serde::Deserialize;

use crate::Result;

/// A response to a wiki query.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize)]
pub struct Response {
    /// The response variable headers.
    pub head: ResponseHead,
    /// The results of the response.
    pub results: ResponseResults,
}

/// A response's variable headers.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize)]
pub struct ResponseHead {
    /// The list of variables.
    pub vars: Box<[Box<str>]>,
}

/// A response's result list.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize)]
pub struct ResponseResults {
    /// The result's entry bindings.
    pub bindings: Box<[ResponseBinding]>,
}

/// An entry within a response list.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize)]
pub struct ResponseBinding {
    /// The country's name.
    #[serde(rename = "nameLabel")]
    pub name: ResponseBindingEntry,
    /// The country's alpha-2 code.
    pub code: ResponseBindingEntry,
    /// The country's numeric identifier.
    #[serde(default)]
    pub numeric: Option<ResponseBindingEntry>,
}

/// A value within a response binding.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize)]
pub struct ResponseBindingEntry {
    /// The response's value type.
    #[serde(rename = "type")]
    pub kind: Box<str>,
    /// The response's value.
    pub value: Box<str>,
}

/// Queries Wikidata, returning a list of known ISO-3166 countries.
///
/// # Errors
///
/// This function will return an error if .
pub fn wiki_data() -> Result<Box<[Country]>> {
    let client = ApiSync::new("https://www.wikidata.org/w/api.php")?;
    let output = client.sparql_query(&wiki_query(0))?;
    let response = serde_json::from_value::<Response>(output)?;

    let mut countries = Vec::with_capacity(response.results.bindings.len());

    for ResponseBinding { name, code, numeric } in response.results.bindings {
        let code = CountryCode::from_str(&code.value)?;
        let country = if let Some(numeric) = numeric {
            Country::new(name.value, code, numeric.value.parse()?)
        } else {
            Country::new(name.value, code, u16::MAX)
        };

        countries.push(country);
    }

    Ok(countries.into_boxed_slice())
}

/// Creates a new query with the given entry limit.
#[must_use]
pub fn wiki_query(limit: usize) -> String {
    const QUERY: &str = r#"
SELECT
    ?nameLabel
    ?code
    ?numeric
WHERE
{
    ?name wdt:P31 wd:Q6256;
        wdt:P297 ?code;
        wdt:P299 ?numeric.
    SERVICE wikibase:label
    {
        bd:serviceParam wikibase:language "en".
    }
}"#;

    let query = QUERY.trim().replace("    ", "").replace('\n', " ");

    if limit > 0 { format!("{query}\nLIMIT {limit}") } else { query }
}
