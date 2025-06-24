use convert_case::{Case, Casing};
use derive_more::{AsRef, Debug, Display};
use regex::Regex;
use std::{str::FromStr, sync::LazyLock};

use super::IllegalConversion;

#[allow(clippy::expect_used)]
static IDENTIFIER_RX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z_0-9\-]+$").expect("Valid Rx"));
#[allow(clippy::expect_used)]
static TITLE_RX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z]{1,1}[A-Za-z_]+$").expect("ValidRx"));

#[derive(Debug, Display, AsRef, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(String);

impl Identifier {
    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for Identifier {
    type Err = IllegalConversion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if IDENTIFIER_RX.is_match(s) {
            Ok(Self(s.to_case(Case::Snake)))
        } else {
            Err(IllegalConversion {
                value: s.into(),
                dtype: "Identifier",
            })
        }
    }
}

#[derive(Debug, Display, AsRef, Clone, PartialEq, Eq, Hash)]
pub struct Title(String);

impl Title {
    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for Title {
    type Err = IllegalConversion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if TITLE_RX.is_match(s) {
            Ok(Self(s.to_case(Case::Title)))
        } else {
            Err(IllegalConversion {
                value: s.into(),
                dtype: "Title",
            })
        }
    }
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid_identifier() {
        let id: Identifier = "test_aid15-8".parse().unwrap();
        assert_eq!(id.to_string(), "test_aid_15_8".to_string());
        let id: Identifier = "t".parse().unwrap();
        assert_eq!(id.to_string(), "t".to_string());
    }

    #[test]
    fn valid_title() {
        let title: Title = "ATitleThat_big".parse().unwrap();
        assert_eq!(title.to_string(), "A Title That Big".to_string());
        let title: Title = "Al".parse().unwrap();
        assert_eq!(title.to_string(), "Al".to_string());
    }

    #[test]
    fn invalid_identifier() {
        let id = "test_aid15?".parse::<Identifier>();
        assert!(id.is_err(), "{:?}", id);
        let id = "Test_aid15".parse::<Identifier>();
        assert!(id.is_err(), "{:?}", id);
        let id = "T".parse::<Identifier>();
        assert!(id.is_err(), "{:?}", id);
    }

    #[test]
    fn invalid_title() {
        let title = "aTitleThat_big".parse::<Title>();
        assert!(title.is_err(), "{:?}", title);
        let title = "a".parse::<Title>();
        assert!(title.is_err(), "{:?}", title);
        let title = "1aTitleThat_big".parse::<Title>();
        assert!(title.is_err(), "{:?}", title);
    }
}
