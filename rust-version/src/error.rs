use std::fmt::{self, Display};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Application {
    #[error("Could not parse value `{value}` as `{field}`!")]
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
    #[error("Failed to load file!")]
    CouldNotLoadFile,
    #[error("Cannot convert `{src}` to `{dest}`")]
    IllegalConversion {
        src: &'static str,
        dest: &'static str,
    },
    #[error("Name/id matches not found for entities: `{0:?}`")]
    EntityReferencesNotFound(Vec<String>),
    #[error("Incomplete data for entity `{0}`")]
    EntityDataIncomplete(&'static str),
    #[error("Missing entity references:\n{0:?}\n")]
    MultipleMissingEntities(Vec<MissingEntityGroup>),
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
