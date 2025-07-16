use std::rc::Rc;

use ini::SectionIter;

use crate::{
    core::{CharacterEntity, Title},
    error,
};

use super::{
    iter::{EntitySection, SectionRecordIter},
    types::CharacterMap,
};

pub fn parse_characters(ini_iter: SectionIter) -> Result<CharacterMap, error::Application> {
    let mut map = CharacterMap::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Character) {
        let record = record?.into_record(&["start_dialogue"], &[])?;
        let start_dialogue = record.require_parsed("start_dialogue")?;
        let name = record.parse_name::<Title>()?;
        let character = Rc::new(
            CharacterEntity::builder()
                .name(name.clone())
                .start_dialogue(start_dialogue)
                .build(),
        );
        map.insert(name, character);
    }
    Ok(map)
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use std::ops::Deref;

    use ini::Ini;

    use crate::config_parser::test_utils::{data::TakeClone, i, t};

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
    fn parse_characters_success() {
        let ini = Ini::load_from_str(GOOD_CHARACTER_DATA).unwrap();
        let characters = parse_characters(ini.iter()).unwrap();
        assert_eq!(characters.len(), 3);

        let result = characters.take("OldMan");
        let expected = CharacterEntity::builder()
            .name(t("OldMan"))
            .start_dialogue(i("greeting_old_man"))
            .build();
        assert_eq!(result.deref(), &expected);

        assert!(characters.contains_key(&"Merchant".parse().unwrap()));
        assert!(characters.contains_key(&"Guard".parse().unwrap()));
    }

    #[test]
    fn parse_characters_missing_field() {
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
