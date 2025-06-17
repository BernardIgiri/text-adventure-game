use std::rc::Rc;

use derive_getters::Getters;

use crate::error;

use super::{
    entity::{CharacterRefs, ResponseRefs, RoomRefs},
    ActionMap, Character, DialogueMap, GameTitle, Response, RoomMap,
};

#[derive(Debug, Getters)]
pub struct World {
    title: GameTitle,
    action: ActionMap,
    room: RoomMap,
    dialog: DialogueMap,
}

impl World {
    pub fn try_new(
        title: GameTitle,
        actions: ActionMap,
        rooms: RoomMap,
        dialogues: DialogueMap,
        characters: Vec<Rc<Character>>,
        responses: Vec<Rc<Response>>,
    ) -> Result<Self, error::Application> {
        let mut missing_dialogue_ids = characters
            .iter()
            .map(|c| CharacterRefs::new(c).start_dialogue().clone())
            .filter(|id| !dialogues.contains_key(id))
            .map(|id| id.to_string())
            .collect::<Vec<_>>();
        missing_dialogue_ids.extend(
            responses
                .iter()
                .filter_map(|r| ResponseRefs::new(r).leads_to().clone())
                .filter(|id| !dialogues.contains_key(id))
                .map(|id| id.to_string()),
        );
        let mut missing_room_ids = Vec::<String>::new();
        let mut missing_action_ids = Vec::<String>::new();
        for r in rooms.values().flat_map(|inner| inner.values()) {
            let r = RoomRefs::new(r);
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
            action: actions,
            room: rooms,
            dialog: dialogues,
        })
    }
}
