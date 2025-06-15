use convert_case::{Case, Casing};
use derive_more::{AsRef, Display};
use regex::Regex;
use std::{str::FromStr, sync::LazyLock};
use strum::IntoStaticStr;

use crate::error;

#[derive(IntoStaticStr, Display, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityName {
    Identifier(Identifier),
    Title(Title),
}

impl TryInto<Identifier> for EntityName {
    type Error = error::Application;

    fn try_into(self) -> Result<Identifier, Self::Error> {
        if let Self::Identifier(i) = &self {
            Ok(i.clone())
        } else {
            Err(error::IllegalConversion {
                value: self.to_string(),
                dtype: "Identifier(id)",
            })
        }
    }
}

impl TryInto<Title> for EntityName {
    type Error = error::Application;

    fn try_into(self) -> Result<Title, Self::Error> {
        if let Self::Title(i) = &self {
            Ok(i.clone())
        } else {
            Err(error::IllegalConversion {
                value: self.to_string(),
                dtype: "Title(id)",
            })
        }
    }
}

// Valid RX will not panic
#[allow(clippy::unwrap_used)]
static IDENTIFIER_RX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z_0-9\-]+$").unwrap());
#[allow(clippy::unwrap_used)]
static TITLE_RX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Z]{1,1}[A-Za-z_]+$").unwrap());

#[derive(Debug, Display, AsRef, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(String);

impl Identifier {
    pub fn parse(s: &str) -> Result<Self, error::Application> {
        s.parse()
    }
}

impl FromStr for Identifier {
    type Err = error::Application;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if IDENTIFIER_RX.is_match(s) {
            Ok(Self(s.to_case(Case::Snake)))
        } else {
            Err(error::IllegalConversion {
                value: s.into(),
                dtype: "Identifier",
            })
        }
    }
}

#[derive(Debug, Display, AsRef, Clone, PartialEq, Eq, Hash)]
pub struct Title(String);

impl Title {
    pub fn parse(s: &str) -> Result<Self, error::Application> {
        s.parse()
    }
}

impl FromStr for Title {
    type Err = error::Application;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if TITLE_RX.is_match(s) {
            Ok(Self(s.to_case(Case::Title)))
        } else {
            Err(error::IllegalConversion {
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
