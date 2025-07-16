use std::rc::Rc;

use ini::SectionIter;

use crate::{
    core::{DialogueEntity, ResponseEntity},
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
            .collect::<Result<Vec<Rc<ResponseEntity>>, error::Application>>()?;
        let requires = parse_requirements(&record, item_map, room_map)?;
        let dialogue = Rc::new(
            DialogueEntity::builder()
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

    use std::ops::Deref;

    use ini::Ini;

    use crate::{
        config_parser::test_utils::{
            data::{TakeClone, TakeCloneVariant, action_map, item_map, response_map, room_map},
            i,
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

        let result = dialogues.take("cow_emote", None);
        let expected = DialogueEntity::builder()
            .text("Moo!".into())
            .responses(vec![])
            .requires(vec![])
            .build();
        assert_eq!(result.deref(), &expected);

        let result = dialogues.take("robery", None);
        let expected = DialogueEntity::builder()
            .text("Empty your pockets kid!".into())
            .responses(vec![])
            .requires(vec![
                Requirement::HasItem(items.take_clone("ring")),
                Requirement::HasItem(items.take_clone("key")),
                Requirement::RoomVariant(rooms.take_clone("WoodShed", Some("closed"))),
            ])
            .build();
        assert_eq!(result.deref(), &expected);

        let result = dialogues.take("farmer_greeting", Some("scared"));
        let expected = DialogueEntity::builder()
            .text("Hey, is somebody there?".into())
            .responses(vec![])
            .requires(vec![Requirement::RoomVariant(
                rooms.take_clone("WoodShed", Some("closed")),
            )])
            .build();
        assert_eq!(result.deref(), &expected);

        let result = dialogues.take("farmer_greeting", None);
        let expected = DialogueEntity::builder()
            .text("Howdy there stranger!".into())
            .responses(vec![
                responses.take_clone("hello"),
                responses.take_clone("goodbye"),
            ])
            .requires(vec![Requirement::RoomVariant(
                rooms.take_clone("WoodShed", None),
            )])
            .build();
        assert_eq!(result.deref(), &expected);
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
