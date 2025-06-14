use std::rc::Rc;

use ini::SectionIter;

use crate::{error, world::Character};

use super::{
    iter::{EntitySection, RequireProperty, SectionRecordIter},
    types::CharacterMap,
};

pub fn parse_characters(ini_iter: SectionIter) -> Result<CharacterMap, error::Application> {
    let mut map = CharacterMap::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Character.into()) {
        let record = record?;
        let start_dialogue = record.properties.require("start_dialogue", &record)?;
        let character = Rc::new(Character::new(
            record.name.parse()?,
            start_dialogue.parse()?,
        ));
        map.insert(character.name().clone(), character);
    }
    Ok(map)
}
