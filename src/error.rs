use thiserror::Error;

type S = Box<str>;

#[derive(Error, Debug)]
pub enum Application {
    #[error("Cannot use value `{value}` in field `{field}` for entity `{etype}`!")]
    InvalidPropertyValue { etype: S, value: S, field: S },
    #[error("Property `{property}` not found for `{etype}` with id `{id}`!")]
    PropertyNotFound { etype: S, property: S, id: S },
    #[error("Expected `{0}` data not found!")]
    EntitySectionNotFound(S),
    #[error("Could not find entity `{etype}` with id `{id}`!")]
    EntityNotFound { etype: S, id: S },
    #[error("Could not find entity `{etype}` with id `{id}` and variant `{variant}`!")]
    EntityVariantNotFound { etype: S, id: S, variant: S },
    #[error("Failed to load file due to: {0}")]
    CouldNotLoadFile(S),
    #[error("Conversion failed for `{etype}` at property `{property}` with `{source}`")]
    ConversionFailed {
        etype: S,
        property: S,
        #[source]
        source: IllegalConversion,
    },
    #[error("Incomplete data for entity `{0}`. A field could be missing or mispelled.")]
    EntityDataIncomplete(S),
    #[error("Cannot find default `{etype}`  entity for id `{id}`")]
    DefaultEntityNotFound { etype: S, id: S },
    #[error("Missing properties {missing:?} for entity `{etype}` with id `{id}`!")]
    MissingProperties { missing: Vec<S>, etype: S, id: S },
    #[error("Found unexpected properties {unexpected:?} in entity `{etype}` with id `{id}`!")]
    UnexpectedProperties { unexpected: Vec<S>, etype: S, id: S },
    #[error("Unknown section `{0}` found!")]
    UnknownSectionFound(S),
    #[error(
        "Malformed multiline string found on lines {0}! Make sure you have the correct number of \"'s"
    )]
    MalformedMultilineString(S),
    #[error(
        "Found potential circular reference in `{etype}` with id `{parent_id}` from link to child `{child_id}`"
    )]
    CircularReferenceFound { etype: S, parent_id: S, child_id: S },
}

pub use Application::*;

use crate::core::IllegalConversion;
