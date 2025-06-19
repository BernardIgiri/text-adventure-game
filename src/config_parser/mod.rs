mod action;
mod character;
mod dialogue;
mod item;
mod iter;
mod preprocessor;
mod requirement;
mod response;
mod room;
mod title;
mod types;

#[cfg(test)]
pub mod test_utils;

use std::collections::HashSet;

use action::parse_actions;
use character::parse_characters;
use dialogue::parse_dialogues;
use ini::Ini;
use item::parse_items;
use iter::EntitySection;
use response::parse_responses;
use room::parse_rooms;
use strum::IntoEnumIterator;
use title::parse_title;

use crate::{core::World, error};

pub use preprocessor::*;

pub fn parse(ini: Ini) -> Result<World, error::Application> {
    validate_section_types(&ini)?;
    let title = parse_title(&ini)?;
    let characters = parse_characters(ini.iter())?;
    let items = parse_items(ini.iter())?;
    let rooms = parse_rooms(ini.iter(), &characters)?;
    let actions = parse_actions(ini.iter(), &rooms, &items)?;
    let responses = parse_responses(ini.iter(), &actions, &items, &rooms)?;
    let dialogues = parse_dialogues(ini.iter(), &responses, &items, &rooms)?;
    World::try_new(
        title,
        actions,
        rooms,
        dialogues,
        characters.values().cloned().collect(),
        responses.values().cloned().collect(),
    )
}

fn validate_section_types(ini: &Ini) -> Result<(), error::Application> {
    let allowed_set: HashSet<&'static str> = EntitySection::iter().map(|s| s.into()).collect();
    for section in ini.sections().flatten() {
        let section = section.split(':').next().unwrap_or("");
        if !allowed_set.contains(section) {
            return Err(error::UnknownSectionFound(section.to_string()));
        }
    }
    Ok(())
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use asserting::prelude::*;
    use ini::Ini;

    use super::validate_section_types;

    #[test]
    fn validate_section_types_accepts_known_section_type() {
        let mut ini = Ini::new();
        ini.set_to(Some("Action:name"), "test".into(), "value".into());
        let r = validate_section_types(&ini);
        assert_that!(r).is_ok();
    }

    #[test]
    fn validate_section_types_rejects_unknown_section_type() {
        let mut ini = Ini::new();
        ini.set_to(Some("BadSection:name"), "test".into(), "value".into());
        let r = validate_section_types(&ini);
        assert_that!(r)
            .is_err()
            .extracting(|r| r.unwrap_err().to_string())
            .contains("BadSection");
    }
}
