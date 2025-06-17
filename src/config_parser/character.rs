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

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use ini::Ini;

    use super::*;
    use asserting::prelude::*;

    const GOOD_CHARACTER_DATA: &str = r"
                [Character:OldMan]
                start_dialogue=greeting_old_man

                [Character:Merchant]
                start_dialogue=buy_or_leave

                [Character:Guard]
                start_dialogue=halt_intruder
            ";
    const BAD_CHARACTER_DATA: &str = r"
                [Character:OldMan]

                [Character:Merchant]
                start_dialogue=buy_or_leave
            ";

    #[test]
    fn test_parse_characters_success() {
        let ini = Ini::load_from_str(GOOD_CHARACTER_DATA).unwrap();
        let characters = parse_characters(ini.iter()).unwrap();
        assert_eq!(characters.len(), 3);

        let c = characters.get(&"OldMan".parse().unwrap()).unwrap();
        assert_eq!(c.name().to_string().as_str(), "Old Man");

        assert!(characters.contains_key(&"Merchant".parse().unwrap()));
        assert!(characters.contains_key(&"Guard".parse().unwrap()));
    }

    #[test]
    fn test_parse_characters_missing_field() {
        let ini = Ini::load_from_str(BAD_CHARACTER_DATA).unwrap();
        let characters = parse_characters(ini.iter());

        assert!(characters.is_err());
        let err = characters.err().unwrap().to_string();
        assert_that!(err.as_str())
            .contains("Character")
            .contains("start_dialogue")
            .contains("OldMan");
    }
}
