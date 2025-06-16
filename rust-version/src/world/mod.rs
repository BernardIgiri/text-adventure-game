mod entity;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use derive_more::Debug;
use entity::{CharacterRefs, DialogueRefs, ResponseRefs, RoomRefs};

pub use entity::{
    Action, ChangeRoom, Character, Dialogue, GameTitle, GiveItem, Identifier, Item, ReplaceItem,
    Requirement, Response, Room, TakeItem, Title,
};

use crate::error::{self, MissingEntityGroup};

pub type ActionMap = HashMap<Identifier, Rc<Action>>;
pub type DialogueMap = HashMap<Identifier, HashMap<Option<Identifier>, Rc<Dialogue>>>;
pub type RoomMap = HashMap<Title, HashMap<Option<Identifier>, Rc<Room>>>;

#[derive(Debug, PartialEq, Eq)]
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
    pub fn greeting(&self) -> &String {
        self.title.greeting()
    }
    pub fn credits(&self) -> &String {
        self.title.credits()
    }
}

#[derive(Debug)]
pub struct GameState<'a> {
    world: &'a World,
    current_room: Option<Rc<Room>>,
    inventory: HashSet<Rc<Item>>,
    active_room_variants: HashMap<Title, Option<Identifier>>,
}

// Game state already handled in World
#[allow(clippy::expect_used)]
impl<'a> GameState<'a> {
    pub fn new(world: &'a World) -> Self {
        Self {
            world,
            current_room: None,
            inventory: HashSet::new(),
            active_room_variants: HashMap::new(),
        }
    }
    pub fn enter_room(&mut self, room: Rc<Room>) {
        self.current_room = Some(room);
    }

    pub fn trigger_response(&mut self, response: &Response) -> bool {
        response
            .triggers()
            .as_ref()
            .is_none_or(|action| self.do_action(action))
    }

    pub fn do_action(&mut self, action: &Action) -> bool {
        use Action::*;
        match action {
            ChangeRoom(c) => {
                if let Some(r) = c.required() {
                    if !self.inventory.contains(r) {
                        return false;
                    } else {
                        self.inventory.remove(r);
                    }
                }
                self.current_room = Some(c.room().clone());
                true
            }
            GiveItem(g) => {
                if g.items().iter().all(|i| self.inventory.contains(i)) {
                    for i in g.items() {
                        self.inventory.remove(i);
                    }
                    true
                } else {
                    false
                }
            }
            ReplaceItem(r) => {
                if self.inventory.contains(r.original()) {
                    self.inventory.remove(r.original());
                    self.inventory.insert(r.replacement().clone());
                    true
                } else {
                    false
                }
            }
            TakeItem(t) => {
                if let Some(r) = t.required() {
                    if !self.inventory.contains(r) {
                        return false;
                    } else {
                        self.inventory.remove(r);
                    }
                }
                self.inventory.extend(t.items().clone());
                true
            }
        }
    }
    pub fn current_room(&self) -> Rc<Room> {
        self.current_room.as_ref().cloned().unwrap_or_else(|| {
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
    pub fn room_exits(&self, room: &Room) -> Vec<Rc<Room>> {
        RoomRefs::new(room)
            .exits()
            .values()
            .map(|name| {
                let variant = self.active_room_variants.get(name).unwrap_or(&None);
                self.look_up_room(name, variant)
                    .expect("The Room variant data should be validated by this point.")
            })
            .collect()
    }
    pub fn response_reply(&self, response: &Response) -> Option<Rc<Dialogue>> {
        ResponseRefs::new(response)
            .leads_to()
            .as_ref()
            .and_then(|name| self.look_up_dialogue(name))
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
            Requirement::HasItem(needed_item) => self.inventory.contains(needed_item),
            Requirement::DoesNotHave(needed_item) => !self.inventory.contains(needed_item),
            Requirement::RoomVariant(expected_room) => {
                let expected_name = expected_room.name();
                let expected_variant = expected_room.variant();
                // it's easier to read this way
                #[allow(clippy::option_if_let_else)]
                match self.active_room_variants.get(expected_name) {
                    None => expected_variant.is_none(),
                    Some(active_variant) => active_variant == expected_variant,
                }
            }
        }
    }
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::*;

    fn make_room(name: Title, variant: Option<Identifier>) -> Rc<Room> {
        Rc::new(
            Room::builder()
                .name(name)
                .maybe_variant(variant)
                .description("Test".into())
                .items(Vec::new())
                .characters(Vec::new())
                .exits(HashMap::new())
                .actions(Vec::new())
                .build(),
        )
    }

    #[test]
    fn test_room_variant_requirement_met() {
        let room_name = "Barn".parse::<Title>().unwrap();
        let variant_name = "drained".parse::<Identifier>().unwrap();
        let title = GameTitle::new("".into(), "".into(), "".into(), room_name.clone());

        let mut rooms = RoomMap::new();
        let mut inner = HashMap::new();
        inner.insert(None, make_room(room_name.clone(), None));
        inner.insert(
            Some(variant_name.clone()),
            make_room(room_name.clone(), Some(variant_name.clone())),
        );
        rooms.insert(room_name.clone(), inner);
        let world = World::try_new(
            title,
            HashMap::new(),
            rooms,
            HashMap::new(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        let mut state = GameState::new(&world);
        let req = Requirement::RoomVariant(make_room(room_name.clone(), None));
        assert!(state.requirement_met(&req));

        // Case 2: No entry in map, expected = Some(...) => false
        let req =
            Requirement::RoomVariant(make_room(room_name.clone(), Some(variant_name.clone())));
        assert!(!state.requirement_met(&req));

        // Case 3: Entry is None, expected = None => true
        state.active_room_variants.insert(room_name.clone(), None);
        let req = Requirement::RoomVariant(make_room(room_name.clone(), None));
        assert!(state.requirement_met(&req));

        // Case 4: Entry = Some(x), expected = Some(x) => true
        let ident = variant_name.clone();
        state
            .active_room_variants
            .insert(room_name.clone(), Some(ident));
        let req = Requirement::RoomVariant(make_room(room_name.clone(), Some(variant_name)));
        assert!(state.requirement_met(&req));

        // Case 5: Entry = Some(x), expected = Some(y) ≠ x => false
        let req =
            Requirement::RoomVariant(make_room(room_name, Some("different".parse().unwrap())));
        assert!(!state.requirement_met(&req));
    }
}
