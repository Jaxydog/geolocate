//! A utility tool for downloading and parsing a database of countries.
#![cfg_attr(not(debug_assertions), deny(clippy::unwrap_used))]
#![cfg_attr(not(debug_assertions), warn(missing_docs))]
#![cfg_attr(debug_assertions, warn(clippy::unwrap_used))]
#![warn(clippy::nursery, clippy::pedantic, clippy::todo)]
#![allow(clippy::module_name_repetitions)]

use std::num::ParseIntError;
use std::path::Path;

use clap::Parser;
use geolocate_core::country::InvalidCodeError;

/// Provides the application's mediawiki API.
pub mod wiki;

/// A result with a default error type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error returned by this application.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error from an invalid country code.
    #[error(transparent)]
    InvalidCode(#[from] InvalidCodeError),
    /// An error from an IO operation.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// An error from parsing an integer.
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    /// An error during serializing or deserializing JSON.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// An error that may be returned from the [`mediawiki`] crate.
    #[error(transparent)]
    MediaWiki(#[from] mediawiki::MediaWikiError),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Parser)]
#[command(about, author, version, long_about = None)]
pub struct Arguments {
    /// The file to output country data into.
    pub path: Box<Path>,
}

/// The application's entrypoint.
///
/// # Errors
///
/// This function will return an error if the program fails to run.
pub fn main() -> Result<()> {
    let Arguments { path } = Arguments::parse();

    let mut countries = crate::wiki::wiki_data()?;

    countries.sort_unstable_by_key(|c| c.numeric);

    let contents = serde_json::to_vec_pretty(&countries)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&path, contents)?;

    println!("Wrote {} entries to '{}'", countries.len(), path.to_string_lossy());

    Ok(())
}
