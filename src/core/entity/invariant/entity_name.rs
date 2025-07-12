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
    use asserting::prelude::*;

    #[test]
    fn valid_identifier() {
        let list = ["test_aid15-8", "t"].map(Identifier::from_str);
        assert_that!(&list).each_item(|e| e.is_ok());
        assert_that!(list.map(|i| i.unwrap().to_string())).contains_exactly(["test_aid_15_8", "t"]);
    }

    #[test]
    fn valid_title() {
        let list = ["ATitleThat_big", "Al"].map(Title::from_str);
        assert_that!(&list).each_item(|e| e.is_ok());
        assert_that!(list.map(|i| i.unwrap().to_string()))
            .contains_exactly(["A Title That Big", "Al"]);
    }

    #[test]
    fn invalid_identifier() {
        assert_that!(["test_aid15?", "Test_aid15", "T"].map(Identifier::from_str))
            .each_item(|e| e.is_err());
    }

    #[test]
    fn invalid_title() {
        assert_that!(["aTitleThat_big", "a", "1aTitleThat_big"].map(Title::from_str))
            .each_item(|e| e.is_err());
    }
}
