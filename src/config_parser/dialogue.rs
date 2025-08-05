use ini::SectionIter;

use crate::{core::DialogueRaw, error};

use super::{
    iter::{EntitySection, SectionRecordIter},
    requirement::parse_requirements,
};

pub fn parse_dialogues(ini_iter: SectionIter) -> Result<Vec<DialogueRaw>, error::Application> {
    let mut list = Vec::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Dialogue) {
        let record = record?.into_record(&["text"], &["response", "requires"])?;
        let text = record.require("text")?.to_string();
        let responses = record
            .get_list_parsed("response")
            .collect::<Result<Vec<_>, error::Application>>()?;
        let requires = parse_requirements(&record)?;
        let name = record.parse_name()?;
        let variant = record.variant().clone();
        list.push(DialogueRaw {
            name,
            variant,
            text,
            responses,
            requires,
        });
    }
    Ok(list)
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {

    use ini::Ini;

    use crate::config_parser::test_utils::i;

    use super::*;
    use asserting::prelude::*;

    const GOOD_DATA: &str = r"
        [Dialogue:farmer_greeting]
        text=Howdy there stranger!
        response=hello,goodbye
        requires=room_variant:WoodShed

        [Dialogue:farmer_greeting|scared]
        text=Hey, is somebody there?
        requires=room_variant:WoodShed|closed

        [Dialogue:cow_emote]
        text=Moo!

        [Dialogue:robery]
        text=Empty your pockets kid!
        requires=has_item:ring,has_item:key,room_variant:WoodShed|closed
    ";

    const BAD_DATA_MISSING_TEXT: &str = r"
        [Dialogue:farmer_greeting]
        response=hello,goodbye
        requires=room_variant:WoodShed
    ";

    const BAD_DATA_BAD_REQUIREMENT: &str = r"
        [Dialogue:farmer_greeting]
        text=Howdy there stranger!
        response=hello,goodbye
        requires=has_itemunobtainium_ore
    ";

    #[test]
    fn parse_dialogue_sucessfully() {
        let ini = Ini::load_from_str(GOOD_DATA).unwrap();
        let dialogues = parse_dialogues(ini.iter()).unwrap();
        assert_that!(&dialogues)
            .has_length(4)
            .satisfies_with_message("has expected ids", |d| {
                d.iter().any(|d| d.name == i("farmer_greeting"))
                    && d.iter().any(|d| d.name == i("cow_emote"))
                    && d.iter().any(|d| d.name == i("robery"))
            })
            .satisfies_with_message("has expected variants", |d| {
                d.iter()
                    .any(|d| d.name == i("farmer_greeting") && d.variant.is_none())
                    && d.iter()
                        .any(|d| d.name == i("farmer_greeting") && d.variant == Some(i("scared")))
            })
            .satisfies_with_message("contains expected text", |d| {
                d.iter().any(|d| d.text == "Howdy there stranger!")
            })
            .satisfies_with_message("has requirements", |d| {
                d.iter().any(|d| !d.requires.is_empty())
            });
    }

    #[test]
    fn parse_dialogue_missing_text() {
        let ini = Ini::load_from_str(BAD_DATA_MISSING_TEXT).unwrap();
        let dialogues = parse_dialogues(ini.iter());
        assert_that!(dialogues)
            .is_err()
            .extracting(|e| e.err().unwrap().to_string())
            .contains("Missing")
            .contains("farmer_greeting")
            .contains("text")
            .contains("Dialogue");
    }

    #[test]
    fn parse_dialogue_bad_requirement() {
        let ini = Ini::load_from_str(BAD_DATA_BAD_REQUIREMENT).unwrap();
        let dialogues = parse_dialogues(ini.iter());
        assert_that!(dialogues)
            .is_err()
            .extracting(|e| e.err().unwrap().to_string())
            .contains("unobtainium_ore")
            .contains("requirement");
    }
}
