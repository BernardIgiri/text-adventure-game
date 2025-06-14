mod entity;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use derive_more::Debug;
use derive_new::new;
use entity::{CharacterRefs, ResponseRefs, RoomRefs};

pub use entity::{
    Action, ChangeRoom, Character, Dialogue, GiveItem, Identifier, Item, ReplaceItem, Requirement,
    Response, Room, TakeItem, Title,
};

use crate::error::{self, MissingEntityGroup};

pub type ActionMap = HashMap<Identifier, Rc<Action>>;
pub type DialogueMap = HashMap<Identifier, HashMap<Option<Identifier>, Rc<Dialogue>>>;
pub type RoomMap = HashMap<Title, HashMap<Option<Identifier>, Rc<Room>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
    action: ActionMap,
    room: RoomMap,
    dialog: DialogueMap,
}

impl World {
    pub fn try_new(
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
        let mut missing = Vec::new();
        if !missing_dialogue_ids.is_empty() {
            missing.push(MissingEntityGroup {
                etype: "Dialogue",
                ids: missing_dialogue_ids,
            });
        }
        if !missing_action_ids.is_empty() {
            missing.push(MissingEntityGroup {
                etype: "Action",
                ids: missing_action_ids,
            });
        }
        if !missing_room_ids.is_empty() {
            missing.push(MissingEntityGroup {
                etype: "Room",
                ids: missing_room_ids,
            });
        }
        if !missing.is_empty() {
            return Err(error::Application::MultipleMissingEntities(missing));
        }
        Ok(Self {
            action: actions,
            room: rooms,
            dialog: dialogues,
        })
    }
}

// TODO: Implement this
#[allow(dead_code)]
#[derive(Default, Debug, Clone)]
pub struct GameState {
    current_room: Option<Rc<Room>>,
    inventory: HashSet<Rc<Item>>,
    active_room_variants: HashMap<Title, Option<Identifier>>,
}

// TODO: Implement this
#[allow(dead_code)]
#[derive(new, Debug)]
pub struct GameQuery<'a> {
    world: &'a World,
    state: &'a mut GameState,
}

// TODO: Implement this
#[allow(dead_code)]
impl<'a> GameQuery<'a> {
    pub fn get_character_dialogue(&self, character: &Character) -> Option<Rc<Dialogue>> {
        self.look_up_dialogue(CharacterRefs::new(character).start_dialogue())
    }
    pub fn get_room_actions(&self, room: &Room) -> Vec<Rc<Action>> {
        RoomRefs::new(room)
            .actions()
            .iter()
            .filter_map(|id| self.world.action.get(id))
            .cloned()
            .collect()
    }
    pub fn get_room_exit(&self, room: &Room, direction: Identifier) -> Option<Rc<Room>> {
        RoomRefs::new(room)
            .exits()
            .get(&direction)
            .and_then(|id| self.state.active_room_variants.get(id).map(|v| (id, v)))
            .and_then(|(id, variant)| self.world.room.get(id)?.get(variant).cloned())
    }
    pub fn get_response_reply(&self, response: &Response) -> Option<Rc<Dialogue>> {
        ResponseRefs::new(response)
            .leads_to()
            .as_ref()
            .map(|id| self.look_up_dialogue(id))?
    }
    fn look_up_dialogue(&self, id: &Identifier) -> Option<Rc<Dialogue>> {
        self.world
            .dialog
            .get(id)?
            .values()
            .find(|dialogue| {
                dialogue
                    .requires()
                    .iter()
                    .all(|req| self.requirement_met(req))
            })
            .cloned()
    }
    fn requirement_met(&self, requirement: &Requirement) -> bool {
        match requirement {
            Requirement::HasItem(needed_item) => self
                .state
                .inventory
                .iter()
                .any(|item| Rc::ptr_eq(item, needed_item)),
            Requirement::RoomVariant(expected_room) => {
                let title = expected_room.name();
                let expected_variant = expected_room.variant();
                self.state
                    .active_room_variants
                    .get(title)
                    .map(|active_variant| active_variant == expected_variant)
                    .unwrap_or(false)
            }
        }
    }
}
