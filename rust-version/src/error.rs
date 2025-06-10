use thiserror::Error;

#[derive(Error, Debug)]
pub enum Game {
    #[error("Could not parse value `{value}` as `{field}`!")]
    CouldNotParse { value: String, field: &'static str },
    #[error("Missing expected value for `{0}`!")]
    MissingExpectedValue(&'static str),
    #[error("No `{0}` data found!")]
    NoDataForEntityType(&'static str),
    #[error("Missing entity of type `{etype}` with id `{id}`!")]
    MissingEntity { etype: &'static str, id: String },
    #[error("Failed to load file!")]
    CouldNotLoadFile,
    #[error("Cannot convert `{src}` to `{dest}`")]
    IllegalConversion {
        src: &'static str,
        dest: &'static str,
    },
    #[error("Incomplete entities found: `{0:?}`")]
    IncompleteEntities(Vec<String>),
}
