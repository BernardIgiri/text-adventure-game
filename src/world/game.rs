use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use ini::Ini;

use crate::{config_parser, error};

use super::{entity::*, Identifier, Title, World};

#[derive(Debug)]
pub struct GameState {
    world: World,
    current_room: Option<Rc<Room>>,
    inventory: HashSet<Rc<Item>>,
    active_room_variants: HashMap<Title, Option<Identifier>>,
}

// Game state already handled in World
#[allow(clippy::expect_used)]
impl GameState {
    pub fn from_ini(ini: Ini) -> Result<Self, error::Application> {
        Ok(Self::new(config_parser::parse(ini)?))
    }
    pub fn new(world: World) -> Self {
        Self {
            world,
            current_room: None,
            inventory: HashSet::new(),
            active_room_variants: HashMap::new(),
        }
    }
    pub fn title(&self) -> &String {
        self.world.title().title()
    }
    pub fn greeting(&self) -> &String {
        self.world.title().greeting()
    }
    pub fn credits(&self) -> &String {
        self.world.title().credits()
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
            self.look_up_room(self.world.title().start_room(), &None)
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
            .filter_map(|name| self.world.action().get(name))
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
        self.world.room().get(name)?.get(variant).cloned()
    }
    fn look_up_dialogue(&self, id: &Identifier) -> Option<Rc<Dialogue>> {
        self.world
            .dialog()
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

    use crate::world::RoomMap;

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
        let mut state = GameState::new(world);
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
