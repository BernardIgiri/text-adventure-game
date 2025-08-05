// Allowed in Tests
#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::core::{Identifier, Title};

pub fn t(s: &str) -> Title {
    s.parse().unwrap()
}

pub fn i(s: &str) -> Identifier {
    s.parse().unwrap()
}
