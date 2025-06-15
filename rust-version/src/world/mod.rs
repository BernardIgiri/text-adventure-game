mod entity;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use derive_more::Debug;
use derive_new::new;
use entity::{CharacterRefs, DialogueRefs, ResponseRefs, RoomRefs};

pub use entity::{
    Action, ChangeRoom, Character, Dialogue, GameTitle, GiveItem, Identifier, Item, ReplaceItem,
    Requirement, Response, Room, TakeItem, Title,
};

use crate::error::{self, MissingEntityGroup};

pub type ActionMap = HashMap<Identifier, Rc<Action>>;
pub type DialogueMap = HashMap<Identifier, HashMap<Option<Identifier>, Rc<Dialogue>>>;
pub type RoomMap = HashMap<Title, HashMap<Option<Identifier>, Rc<Room>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
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
            title,
            action: actions,
            room: rooms,
            dialog: dialogues,
        })
    }
    pub fn title(&self) -> &String {
        self.title.title()
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
impl GameState {
    pub fn enter_room(&mut self, room: Rc<Room>) {
        self.current_room = Some(room);
    }

    pub fn do_action(&mut self, action: Rc<Action>) -> Option<()> {
        use Action::*;

        match action.as_ref() {
            ChangeRoom(c) => {
                if let Some(r) = c.required() {
                    if !self.inventory.contains(r) {
                        return None;
                    } else {
                        self.inventory.remove(r);
                    }
                }
                self.current_room = Some(c.room().clone());
                Some(())
            }
            GiveItem(g) => {
                if g.items().iter().all(|i| self.inventory.contains(i)) {
                    for i in g.items() {
                        self.inventory.remove(i);
                    }
                    Some(())
                } else {
                    None
                }
            }
            ReplaceItem(r) => {
                if self.inventory.contains(r.original()) {
                    self.inventory.remove(r.original());
                    self.inventory.insert(r.replacement().clone());
                    Some(())
                } else {
                    None
                }
            }
            TakeItem(t) => {
                if let Some(r) = t.required() {
                    if !self.inventory.contains(r) {
                        return None;
                    } else {
                        self.inventory.remove(r);
                    }
                }
                self.inventory.extend(t.items().clone());
                Some(())
            }
        }
    }
}
// TODO: Implement this
#[allow(dead_code)]
#[derive(new, Debug)]
pub struct GameQuery<'a> {
    world: &'a World,
    state: &'a GameState,
}

// TODO: Implement this
// World references should already be validated by this point.
#[allow(clippy::expect_used)]
#[allow(dead_code)]
impl<'a> GameQuery<'a> {
    pub fn is_survivable(&self) -> bool {
        self.current_room().is_trap()
    }
    pub fn current_room(&self) -> Rc<Room> {
        self.state
            .current_room
            .as_ref()
            .cloned()
            .unwrap_or_else(|| {
                self.look_up_room(self.world.title.start_room(), &None)
                    .expect("The start_room should be validated by this point.")
            })
    }
    pub fn character_dialogue(&self, character: &Character) -> Rc<Dialogue> {
        self.look_up_dialogue(CharacterRefs::new(character).start_dialogue())
            .expect("Character dialogue names should be validated by this point.")
    }
    pub fn dialogue_responses(&self, dialogue: &Dialogue) -> Vec<Rc<Response>> {
        DialogueRefs::new(dialogue)
            .responses()
            .iter()
            .filter(|response| {
                response.requires().is_empty()
                    || response.requires().iter().all(|r| self.requirement_met(r))
            })
            .cloned()
            .collect()
    }
    pub fn room_actions(&self, room: &Room) -> Vec<Rc<Action>> {
        RoomRefs::new(room)
            .actions()
            .iter()
            .filter_map(|name| self.world.action.get(name))
            .cloned()
            .collect()
    }
    pub fn room_exits(&self, room: &Room, direction: Identifier) -> Rc<Room> {
        RoomRefs::new(room)
            .exits()
            .get(&direction)
            .and_then(|name| {
                self.state
                    .active_room_variants
                    .get(name)
                    .map(|variant| (name, variant))
            })
            .and_then(|(name, variant)| self.look_up_room(name, variant))
            .expect("The Room id data should be validated by this point.")
    }
    pub fn response_reply(&self, response: &Response) -> Rc<Dialogue> {
        ResponseRefs::new(response)
            .leads_to()
            .as_ref()
            .and_then(|name| self.look_up_dialogue(name))
            .expect("Response name should be validated by this point.")
    }
    fn look_up_room(&self, name: &Title, variant: &Option<Identifier>) -> Option<Rc<Room>> {
        self.world.room.get(name)?.get(variant).cloned()
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
