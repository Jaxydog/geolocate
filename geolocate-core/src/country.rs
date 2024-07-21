use std::fmt::{Display, Write};
use std::str::{Chars, FromStr};

use serde::de::{Unexpected, Visitor};
use serde::{Deserialize, Serialize};

/// An ISO-3166 country.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Country {
    /// The country's name.
    pub name: Box<str>,
    /// The country's code.
    pub code: CountryCode,
    /// The country's numeric code.
    pub numeric: u16,
}

impl Country {
    /// Creates a new [`Country`].
    #[inline]
    pub fn new(name: impl AsRef<str>, code: CountryCode, numeric: u16) -> Self {
        Self { name: Box::from(name.as_ref()), code, numeric }
    }
}

impl PartialOrd for Country {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Country {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.numeric.cmp(&other.numeric)
    }
}

/// An error that is returned when trying to parse an invalid country code.
#[repr(transparent)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InvalidCodeError(Box<str>);

impl std::error::Error for InvalidCodeError {}

impl Display for InvalidCodeError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid country code: {}", self.0)
    }
}

/// A country's code.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CountryCode {
    /// An Alpha-2 code.
    Alpha2([char; 2]),
    /// An Alpha-3 code.
    Alpha3([char; 3]),
    /// An Alpha-4 code.
    Alpha4([char; 4]),
    /// An unassigned code.
    Unassigned,
}

impl From<[char; 2]> for CountryCode {
    #[inline]
    fn from(value: [char; 2]) -> Self {
        Self::Alpha2(value)
    }
}

impl From<[char; 3]> for CountryCode {
    #[inline]
    fn from(value: [char; 3]) -> Self {
        Self::Alpha3(value)
    }
}

impl From<[char; 4]> for CountryCode {
    #[inline]
    fn from(value: [char; 4]) -> Self {
        Self::Alpha4(value)
    }
}

impl FromStr for CountryCode {
    type Err = InvalidCodeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        #[inline]
        fn array<const N: usize>(iter: &mut Chars<'_>) -> Option<[char; N]> {
            let array = std::array::from_fn(|_| iter.next().unwrap_or_else(|| unreachable!("a character is missing")));

            array.iter().all(char::is_ascii_uppercase).then_some(array)
        }

        let mut chars = value.chars();

        Ok(match value.chars().count() {
            2 => array(&mut chars).map_or(Self::Unassigned, Self::Alpha2),
            3 => array(&mut chars).map_or(Self::Unassigned, Self::Alpha3),
            4 => array(&mut chars).map_or(Self::Unassigned, Self::Alpha4),
            _ => return Err(InvalidCodeError(value.into())),
        })
    }
}

impl Display for CountryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slice: &[char] = match self {
            Self::Alpha2(array) => array,
            Self::Alpha3(array) => array,
            Self::Alpha4(array) => array,
            Self::Unassigned => return write!(f, "??"),
        };

        slice.iter().try_for_each(|c| f.write_char(*c))
    }
}

impl Serialize for CountryCode {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for CountryCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CodeVisitor;

        impl Visitor<'_> for CodeVisitor {
            type Value = CountryCode;

            #[inline]
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a valid country code")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                CountryCode::from_str(v).map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(CodeVisitor)
    }
}
