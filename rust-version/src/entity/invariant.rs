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
    type Error = error::Game;

    fn try_into(self) -> Result<Identifier, Self::Error> {
        if let Self::Identifier(i) = &self {
            Ok(i.clone())
        } else {
            Err(error::Game::IllegalConversion {
                src: self.into(),
                dest: "Identifier",
            })
        }
    }
}

impl TryInto<Title> for EntityName {
    type Error = error::Game;

    fn try_into(self) -> Result<Title, Self::Error> {
        if let Self::Title(i) = &self {
            Ok(i.clone())
        } else {
            Err(error::Game::IllegalConversion {
                src: self.into(),
                dest: "Title",
            })
        }
    }
}

static IDENTIFIER_RX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z_]+$").unwrap());
static TITLE_RX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Z]{1,1}[A-Za-z]+$").unwrap());

#[derive(Debug, Display, AsRef, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(String);

impl Identifier {
    pub fn parse(s: &str) -> Result<Self, error::Game> {
        s.parse()
    }
}

impl FromStr for Identifier {
    type Err = error::Game;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if IDENTIFIER_RX.is_match(s) {
            Ok(Self(s.to_case(Case::Lower)))
        } else {
            Err(error::Game::CouldNotParse {
                value: s.into(),
                field: "identifier",
            })
        }
    }
}

#[derive(Debug, Display, AsRef, Clone, PartialEq, Eq, Hash)]
pub struct Title(String);

impl Title {
    pub fn parse(s: &str) -> Result<Self, error::Game> {
        s.parse()
    }
}

impl FromStr for Title {
    type Err = error::Game;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if TITLE_RX.is_match(s) {
            Ok(Self(s.to_case(Case::Title)))
        } else {
            Err(error::Game::CouldNotParse {
                value: s.into(),
                field: "title",
            })
        }
    }
}
