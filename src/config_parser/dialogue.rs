use std::rc::Rc;

use ini::SectionIter;

use crate::{
    core::{Dialogue, Response},
    error,
};

use super::{
    iter::{EntitySection, SectionRecordIter},
    requirement::parse_requirements,
    types::{DialogueMap, ItemMap, ResponseMap, RoomMap},
};

pub fn parse_dialogues(
    ini_iter: SectionIter,
    response_map: &ResponseMap,
    item_map: &ItemMap,
    room_map: &RoomMap,
) -> Result<DialogueMap, error::Application> {
    let mut map = DialogueMap::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Dialogue) {
        let record = record?.into_record(&["text"], &["response", "requires"])?;
        let text = record.require("text")?;
        let responses = record
            .get_list("response")
            .map(|s| {
                Ok(response_map
                    .get(&s.parse().map_err(|source| error::ConversionFailed {
                        etype: "Dialogue",
                        property: "response",
                        source,
                    })?)
                    .ok_or_else(|| error::EntityNotFound {
                        etype: "Response",
                        id: s.into(),
                    })?
                    .clone())
            })
            .collect::<Result<Vec<Rc<Response>>, error::Application>>()?;
        let requires = parse_requirements(&record, item_map, room_map)?;
        let dialogue = Rc::new(
            Dialogue::builder()
                .text(text.into())
                .responses(responses)
                .requires(requires)
                .build(),
        );
        map.entry(record.parse_name()?)
            .or_default()
            .insert(record.variant().clone(), dialogue);
    }
    Ok(map)
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {

    use ini::Ini;

    use crate::{
        config_parser::test_utils::{
            data::{action_map, item_map, response_map, room_map},
            i, t,
        },
        core::Requirement,
    };

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

    const BAD_DATA_BAD_VARIANT: &str = r"
        [Dialogue:farmer_greeting]
        text=Howdy there stranger!
        response=hello,goodbye
        requires=room_variant:WoodShed|blessed
    ";

    const BAD_DATA_BAD_REQUIREMENT: &str = r"
        [Dialogue:farmer_greeting]
        text=Howdy there stranger!
        response=hello,goodbye
        requires=has_item:unobtainium_ore
    ";

    #[test]
    fn parse_dialogue_sucessfully() {
        let ini = Ini::load_from_str(GOOD_DATA).unwrap();
        let items = item_map();
        let rooms = room_map(false);
        let actions = action_map(&rooms, &items);
        let responses = response_map(&actions);
        let dialogues = parse_dialogues(ini.iter(), &responses, &items, &rooms).unwrap();
        assert_that!(&dialogues)
            .has_length(3)
            .contains_key(i("farmer_greeting"))
            .contains_key(i("cow_emote"));
        let happy = dialogues
            .get(&i("farmer_greeting"))
            .unwrap()
            .get(&None)
            .unwrap();
        let scared = dialogues
            .get(&i("farmer_greeting"))
            .unwrap()
            .get(&Some(i("scared")))
            .unwrap();
        let moo = dialogues.get(&i("cow_emote")).unwrap().get(&None).unwrap();
        let robery = dialogues.get(&i("robery")).unwrap().get(&None).unwrap();
        assert_eq!(moo.text(), &"Moo!".to_string());

        assert_eq!(robery.text(), &"Empty your pockets kid!".to_string());
        assert_that!(robery.requires()).has_length(3);

        assert_eq!(scared.text(), &"Hey, is somebody there?".to_string());
        assert_that!(scared.requires()).has_length(1);
        assert_that!(scared.requires().first().unwrap()).satisfies_with_message(
            "expected RoomVariant",
            |r| {
                matches!(**r, Requirement::RoomVariant(ref rv)
                if rv.name() == &t("WoodShed") && rv.variant().clone().unwrap() == i("closed"))
            },
        );

        assert_that!(happy.requires()).has_length(1);
        assert_eq!(happy.text(), &"Howdy there stranger!".to_string());
        assert_that!(happy.requires().first().unwrap()).satisfies_with_message(
            "expected RoomVariant",
            |r| {
                matches!(**r, Requirement::RoomVariant(ref rv)
                if rv.name() == &t("WoodShed") && rv.variant().is_none())
            },
        );
    }

    #[test]
    fn parse_dialogue_missing_text() {
        let ini = Ini::load_from_str(BAD_DATA_MISSING_TEXT).unwrap();
        let items = item_map();
        let rooms = room_map(false);
        let actions = action_map(&rooms, &items);
        let responses = response_map(&actions);
        let dialogues = parse_dialogues(ini.iter(), &responses, &items, &rooms);
        assert_that!(dialogues)
            .is_err()
            .extracting(|e| e.err().unwrap().to_string())
            .contains("Missing")
            .contains("farmer_greeting")
            .contains("text")
            .contains("Dialogue");
    }

    #[test]
    fn parse_dialogue_bad_variant() {
        let ini = Ini::load_from_str(BAD_DATA_BAD_VARIANT).unwrap();
        let items = item_map();
        let rooms = room_map(false);
        let actions = action_map(&rooms, &items);
        let responses = response_map(&actions);
        let dialogues = parse_dialogues(ini.iter(), &responses, &items, &rooms);
        assert_that!(dialogues)
            .is_err()
            .extracting(|e| e.err().unwrap().to_string())
            .contains("not find")
            .contains("blessed")
            .contains("WoodShed")
            .contains("Room");
    }

    #[test]
    fn parse_dialogue_bad_requirement() {
        let ini = Ini::load_from_str(BAD_DATA_BAD_REQUIREMENT).unwrap();
        let items = item_map();
        let rooms = room_map(false);
        let actions = action_map(&rooms, &items);
        let responses = response_map(&actions);
        let dialogues = parse_dialogues(ini.iter(), &responses, &items, &rooms);
        assert_that!(dialogues)
            .is_err()
            .extracting(|e| e.err().unwrap().to_string())
            .contains("not find")
            .contains("unobtainium_ore")
            .contains("Item");
    }
}
