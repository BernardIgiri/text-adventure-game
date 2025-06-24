mod color;
mod entity_name;

use derive_more::Display;
use thiserror::Error;

pub use color::*;
pub use entity_name::*;

#[derive(Error, Debug, Display)]
#[display("Cannot convert `{value}` to type {dtype}")]
pub struct IllegalConversion {
    value: String,
    dtype: &'static str,
}
