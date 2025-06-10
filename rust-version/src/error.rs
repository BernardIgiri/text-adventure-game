use thiserror::Error;

#[derive(Error, Debug)]
pub enum Game {
    #[error("Could not parse value `{value}` as `{field}`!")]
    InvalidPropValue { value: String, field: &'static str },
    #[error("Missing expected value for `{0}`!")]
    PropertyNotFound(&'static str),
    #[error("No `{0}` data found!")]
    EntitySectionNotFound(&'static str),
    #[error("Missing entity of type `{etype}` with id `{id}`!")]
    MissingEntity { etype: &'static str, id: String },
    #[error("Failed to load file!")]
    CouldNotLoadFile,
    #[error("Cannot convert `{src}` to `{dest}`")]
    IllegalConversion {
        src: &'static str,
        dest: &'static str,
    },
    #[error("Could not link entities due to missing/incorrect names/ids: `{0:?}`")]
    UnlinkedEntities(Vec<String>),
    #[error("Incomplete entity `{0}`")]
    IncompleteEntity(&'static str),
}
