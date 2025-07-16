use std::rc::Rc;

use bon::bon;
use derive_getters::Getters;

use crate::error;

use super::{
    ActionEntity, ActionMap, CharacterEntity, DialogueMap, GameTitle, Language, ResponseEntity,
    RoomMap, Theme,
};

#[derive(Debug, Getters)]
pub struct World {
    title: GameTitle,
    theme: Rc<Theme>,
    language: Rc<Language>,
    actions: ActionMap,
    rooms: RoomMap,
    dialogues: DialogueMap,
}

#[bon]
impl World {
    #[builder]
    pub fn try_new(
        title: GameTitle,
        theme: Theme,
        language: Language,
        actions: ActionMap,
        rooms: RoomMap,
        dialogues: DialogueMap,
        characters: Vec<Rc<CharacterEntity>>,
        responses: Vec<Rc<ResponseEntity>>,
    ) -> Result<Self, error::Application> {
        // find missing defaults
        for (id, inner) in rooms.iter() {
            if !inner.contains_key(&None) {
                return Err(error::DefaultEntityNotFound {
                    etype: "Room",
                    id: id.to_string(),
                });
            }
        }
        for (id, inner) in dialogues.iter() {
            if !inner.contains_key(&None) {
                return Err(error::DefaultEntityNotFound {
                    etype: "Dialogue",
                    id: id.to_string(),
                });
            }
        }
        // Find missing ids
        let mut missing_dialogue_ids = characters
            .iter()
            .map(|c| c.start_dialogue().clone())
            .filter(|id| !dialogues.contains_key(id))
            .map(|id| id.to_string())
            .collect::<Vec<_>>();
        missing_dialogue_ids.extend(
            responses
                .iter()
                .filter_map(|r| r.leads_to().clone())
                .filter(|id| !dialogues.contains_key(id))
                .map(|id| id.to_string()),
        );
        let mut missing_room_ids = Vec::<String>::new();
        let mut missing_action_ids = Vec::<String>::new();
        for r in rooms.values().flat_map(|inner| inner.values()) {
            missing_action_ids.extend(
                r.actions()
                    .iter()
                    .filter(|id| !actions.contains_key(id))
                    .map(ToString::to_string),
            );
            missing_room_ids.extend(
                r.exits()
                    .values()
                    .filter(|id| !rooms.contains_key(id))
                    .map(ToString::to_string),
            );
        }
        if !rooms.contains_key(title.start_room()) {
            missing_room_ids.push(title.start_room().to_string());
        }
        for a in actions.values() {
            if let ActionEntity::Sequence(s) = &**a {
                let m = s
                    .actions()
                    .iter()
                    .filter(|id| !actions.contains_key(id))
                    .map(|id| id.to_string());
                missing_action_ids.extend(m);
                if let Some(child_id) = s.actions().iter().find(|id| {
                    matches!(
                        actions.get(id).map(|a| &**a),
                        Some(ActionEntity::Sequence(_))
                    )
                }) {
                    return Err(error::CircularReferenceFound {
                        entity: "Action::Sequence",
                        parent_id: s.name().to_string(),
                        child_id: child_id.to_string(),
                    });
                }
            }
        }
        let mut missing = Vec::new();
        if !missing_dialogue_ids.is_empty() {
            missing.push(error::MissingEntityGroup {
                etype: "Dialogue",
                ids: missing_dialogue_ids,
            });
        }
        if !missing_action_ids.is_empty() {
            missing.push(error::MissingEntityGroup {
                etype: "Action",
                ids: missing_action_ids,
            });
        }
        if !missing_room_ids.is_empty() {
            missing.push(error::MissingEntityGroup {
                etype: "Room",
                ids: missing_room_ids,
            });
        }
        if !missing.is_empty() {
            return Err(error::Application::MultipleMissingEntities(missing));
        }
        Ok(Self {
            title,
            theme: Rc::new(theme),
            language: Rc::new(language),
            actions,
            rooms,
            dialogues,
        })
    }
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use asserting::assert_that;
    use asserting::prelude::AssertStringPattern;

    use crate::config_parser::test_utils::data::{
        action_map, character_map, dialogue_map, item_map, response_map, room_map,
    };
    use crate::config_parser::test_utils::{i, t};
    use crate::core::{CharacterMap, ResponseMap, Sequence};

    use super::*;

    struct WorldData {
        title: GameTitle,
        characters: CharacterMap,
        rooms: RoomMap,
        actions: ActionMap,
        responses: ResponseMap,
        dialogues: DialogueMap,
    }

    impl WorldData {
        pub fn new() -> Self {
            let title = GameTitle::new("".into(), "".into(), "".into(), t("WoodShed"));
            let characters = character_map();
            let items = item_map();
            let rooms = room_map(true);
            let actions = action_map(&rooms, &items);
            let responses = response_map(&actions);
            let dialogues = dialogue_map(&responses, &items, &rooms);
            Self {
                title,
                characters,
                rooms,
                actions,
                responses,
                dialogues,
            }
        }

        pub fn world_from_data(self) -> Result<World, error::Application> {
            World::try_new()
                .title(self.title)
                .theme(Theme::default())
                .language(Language::default())
                .actions(self.actions)
                .rooms(self.rooms)
                .dialogues(self.dialogues)
                .characters(self.characters.values().cloned().collect())
                .responses(self.responses.values().cloned().collect())
                .call()
        }
    }

    #[test]
    fn try_new_succeeds_when_all_entities_present() {
        let data = WorldData::new();
        let world = data.world_from_data();
        assert!(world.is_ok());
    }

    #[test]
    fn try_new_fails_when_a_referenced_action_is_missing() {
        let mut data = WorldData::new();
        data.actions.remove(&i("pull_lever"));
        let world = data.world_from_data();
        assert!(world.is_err());
        assert_that!(world.unwrap_err().to_string())
            .contains("Action")
            .contains("pull_lever");
    }

    #[test]
    fn try_new_fails_when_a_referenced_action_in_sequence_is_missing() {
        let mut data = WorldData::new();
        data.actions.remove(&i("open_door"));
        let world = data.world_from_data();
        assert!(world.is_err());
        assert_that!(world.unwrap_err().to_string())
            .contains("Action")
            .contains("open_door");
    }

    #[test]
    fn try_new_fails_when_an_action_sequence_has_a_circular_reference() {
        let mut data = WorldData::new();
        data.actions.insert(
            i("bad_action"),
            Rc::new(ActionEntity::Sequence(
                Sequence::builder()
                    .name(i("bad_action"))
                    .description("".into())
                    .actions(vec![i("multiple")])
                    .build(),
            )),
        );
        let world = data.world_from_data();
        assert!(world.is_err());
        assert_that!(world.unwrap_err().to_string())
            .contains("Action")
            .contains("circular")
            .contains("bad_action")
            .contains("multiple");
    }

    #[test]
    fn try_new_fails_when_a_referenced_room_is_missing() {
        let mut data = WorldData::new();
        data.rooms.remove(&t("Field"));
        let world = data.world_from_data();
        assert!(world.is_err());
        assert_that!(world.unwrap_err().to_string())
            .contains("Room")
            .contains("Field");
    }

    #[test]
    fn try_new_fails_when_a_referenced_dialogue_is_missing() {
        let mut data = WorldData::new();
        data.dialogues.remove(&i("chirp"));
        let world = data.world_from_data();
        assert!(world.is_err());
        assert_that!(world.unwrap_err().to_string())
            .contains("Dialogue")
            .contains("chirp");
    }

    #[test]
    fn try_new_fails_when_default_room_variant_missing() {
        let mut data = WorldData::new();
        data.rooms.get_mut(&t("WoodShed")).unwrap().remove(&None);
        let world = data.world_from_data();
        assert!(world.is_err());
        assert_that!(world.unwrap_err().to_string())
            .contains("Room")
            .contains("Wood Shed");
    }

    #[test]
    fn try_new_fails_when_default_dialogue_variant_missing() {
        let mut data = WorldData::new();
        data.dialogues.get_mut(&i("hello")).unwrap().remove(&None);
        let world = data.world_from_data();
        assert!(world.is_err());
        assert_that!(world.unwrap_err().to_string())
            .contains("Dialogue")
            .contains("hello");
    }
}
