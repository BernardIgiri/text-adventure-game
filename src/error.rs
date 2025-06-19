use std::fmt::{self, Display};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Application {
    #[error("Cannot use value `{value}` in field `{field}`!")]
    InvalidPropertyValue { value: String, field: &'static str },
    #[error("Property `{property}` not found for `{entity}` with id `{id}`!")]
    PropertyNotFound {
        entity: &'static str,
        property: &'static str,
        id: String,
    },
    #[error("Expected `{0}` data not found!")]
    EntitySectionNotFound(&'static str),
    #[error("Could not find entity `{etype}` with id `{id}`!")]
    EntityNotFound { etype: &'static str, id: String },
    #[error("Failed to load file due to: {0}")]
    CouldNotLoadFile(String),
    #[error("Cannot convert `{value}` to `{dtype}`")]
    IllegalConversion { value: String, dtype: &'static str },
    #[error("Name/id matches not found for entities: `{0:?}`")]
    EntityReferencesNotFound(Vec<String>),
    #[error("Incomplete data for entity `{0}`")]
    EntityDataIncomplete(&'static str),
    #[error("Missing entity references:\n{0:?}\n")]
    MultipleMissingEntities(Vec<MissingEntityGroup>),
    #[error("Cannot find default `{etype}`  entity for id `{id}`")]
    DefaultEntityNotFound { etype: &'static str, id: String },
    #[error(
        "Expected properties `{expected_props}` but found `{found_props}` for entity `{entity}` with id `{id}`!"
    )]
    PropertyNamesDontMatch {
        expected_props: String,
        found_props: String,
        entity: &'static str,
        id: String,
    },
    #[error("Unknown section `{0}` found!")]
    UnknownSectionFound(String),
    #[error("Malformed multiline string found on lines {0}! Make sure you have the correct number of \"'s")]
    MalformedMultilineString(String),
}

#[derive(Debug)]
pub struct MissingEntityGroup {
    pub etype: &'static str,
    pub ids: Vec<String>,
}

impl Display for MissingEntityGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: {}", self.etype, self.ids.join(", "))
    }
}

pub use Application::*;
