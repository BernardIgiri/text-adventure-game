use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use cursive::reexports::log::info;
use ini::Ini;

use crate::{config_parser, error};

use super::{entity::*, Identifier, Title, World};

#[derive(Debug)]
pub struct GameState {
    world: World,
    current_room: Option<Title>,
    inventory: HashSet<Rc<Item>>,
    active_room_variants: HashMap<Title, Identifier>,
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
        self.current_room = Some(room.name().clone());
    }
    pub fn trigger_response(&mut self, response: &Response) -> Option<Rc<Action>> {
        info!("trigger_response({:#?})", response);
        response
            .triggers()
            .as_ref()
            .and_then(|action| {
                if self.do_action(action) {
                    Some(action)
                } else {
                    None
                }
            })
            .cloned()
    }
    pub fn do_action(&mut self, action: &Action) -> bool {
        info!("do_action({:#?})", action);
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
                let room = c.room();
                if let Some(v) = room.variant() {
                    self.active_room_variants
                        .insert(room.name().clone(), v.clone());
                } else {
                    self.active_room_variants.remove(room.name());
                }
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
            Teleport(t) => {
                if let Some(r) = t.required() {
                    if !self.inventory.contains(r) {
                        return false;
                    } else {
                        self.inventory.remove(r);
                    }
                }
                self.enter_room(
                    self.look_up_room(t.room_name())
                        .expect("All Rooms should exists in the world!"),
                );
                true
            }
        }
    }
    pub fn current_room(&self) -> Rc<Room> {
        self.look_up_room(
            &self
                .current_room
                .clone()
                .unwrap_or_else(|| self.world.title().start_room().clone()),
        )
        .expect("All rooms should exist in the world!")
    }
    pub fn character_start_dialogue(&self, character: &Character) -> Rc<Dialogue> {
        self.look_up_dialogue(CharacterRefs::new(character).start_dialogue())
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
            .filter_map(|name| self.world.actions().get(name))
            .cloned()
            .collect()
    }
    pub fn room_exits(&self, room: &Room) -> Vec<Rc<Room>> {
        RoomRefs::new(room)
            .exits()
            .values()
            .map(|name| {
                self.look_up_room(name)
                    .expect("All Room exits should be in the world!")
            })
            .collect()
    }
    pub fn response_reply(&self, response: &Response) -> Option<Rc<Dialogue>> {
        ResponseRefs::new(response)
            .leads_to()
            .as_ref()
            .map(|name| self.look_up_dialogue(name))
    }
    fn look_up_room(&self, name: &Title) -> Option<Rc<Room>> {
        let variant = self.active_room_variants.get(name).cloned();
        self.world.rooms().get(name)?.get(&variant).cloned()
    }
    fn look_up_dialogue(&self, id: &Identifier) -> Rc<Dialogue> {
        let variants = self
            .world
            .dialogues()
            .get(id)
            .expect("All dialogue ids should be in the world!");
        variants
            .values()
            .filter_map(|dialogue| {
                let count = dialogue
                    .requires()
                    .iter()
                    .filter(|req| self.requirement_met(req))
                    .count();
                if dialogue.requires().len() != count || count == 0 {
                    None
                } else {
                    Some((count, dialogue))
                }
            })
            .max_by_key(|(k, _)| *k)
            .map(|(_, v)| v)
            .unwrap_or_else(|| {
                variants
                    .get(&None)
                    .expect("All dialogues should have a default variant")
            })
            .clone()
    }
    fn requirement_met(&self, requirement: &Requirement) -> bool {
        match requirement {
            Requirement::HasItem(needed_item) => self.inventory.contains(needed_item),
            Requirement::DoesNotHave(needed_item) => !self.inventory.contains(needed_item),
            Requirement::RoomVariant(expected_room) => {
                let expected_name = expected_room.name();
                expected_room.variant() == &self.active_room_variants.get(expected_name).cloned()
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

    use asserting::prelude::*;

    use crate::config_parser::test_utils::data::{
        action_map, character_map, dialogue_map, item_map, response_map_with_items, room_map,
    };
    use crate::config_parser::test_utils::{i, t};
    use crate::core::ItemMap;

    use super::*;

    fn make_room(name: Title, variant: Option<Identifier>) -> Rc<Room> {
        Rc::new(
            Room::builder()
                .name(name)
                .maybe_variant(variant)
                .description("Test".into())
                .characters(Vec::new())
                .exits(HashMap::new())
                .actions(Vec::new())
                .build(),
        )
    }

    fn make_game() -> (GameState, ItemMap) {
        let items = item_map();
        let title = GameTitle::new("".into(), "".into(), "".into(), t("WoodShed"));
        let characters = character_map();
        let rooms = room_map(true);
        let actions = action_map(&rooms, &items);
        let responses = response_map_with_items(&actions, &items);
        let dialogues = dialogue_map(&responses, &items, &rooms);
        let world = World::try_new(
            title,
            actions,
            rooms,
            dialogues,
            characters.values().cloned().collect(),
            responses.values().cloned().collect(),
        )
        .unwrap();
        (GameState::new(world), items)
    }

    #[test]
    fn test_dialogue_responses_returns_only_defaults_when_no_requirements_are_met() {
        let (state, ..) = make_game();
        let dialogue = state.look_up_dialogue(&i("hello"));
        let responses = state.dialogue_responses(&dialogue);
        assert_that!(responses).satisfies_with_message("Excludes unexpected responses", |list| {
            list.iter()
                .all(|r| !r.text().contains("key") && !r.text().contains("ring"))
        });
    }

    #[test]
    fn test_dialogue_responses_include_multiple_met_requirements() {
        let (mut state, items) = make_game();
        let required = [
            items.get(&i("ring")).unwrap().clone(),
            items.get(&i("key")).unwrap().clone(),
        ];
        state.inventory.extend(required);
        let dialogue = state.look_up_dialogue(&i("hello"));
        let responses = state.dialogue_responses(&dialogue);
        assert_that!(responses.clone())
            .satisfies_with_message("Includes first expected response", |list| {
                list.iter().any(|r| r.text().contains("ring"))
            });
        assert_that!(responses)
            .satisfies_with_message("Includes second expected response", |list| {
                list.iter().any(|r| r.text().contains("key"))
            });
    }

    #[test]
    fn test_dialogue_responses_includes_met_requirements() {
        let (mut state, items) = make_game();
        let required = items.get(&i("ring")).unwrap().clone();
        state.inventory.insert(required);
        let dialogue = state.look_up_dialogue(&i("hello"));
        let responses = state.dialogue_responses(&dialogue);
        assert_that!(responses.clone())
            .satisfies_with_message("Includes expected responses", |list| {
                list.iter().any(|r| r.text().contains("ring"))
            });
        assert_that!(responses).satisfies_with_message("Excludes unexpected responses", |list| {
            list.iter().all(|r| !r.text().contains("key"))
        });
    }

    #[test]
    fn test_look_up_dialogue_returns_variant_if_requirement_met() {
        let (mut state, ..) = make_game();
        let dialogue_id = i("hello");
        state
            .active_room_variants
            .insert(t("WoodShed"), i("closed"));
        let dialogue = state.look_up_dialogue(&dialogue_id);
        assert_that!(dialogue.text()).contains("Who goes there");
    }

    #[test]
    fn test_look_up_dialogue_falls_back_to_default_variant() {
        let (state, ..) = make_game();
        let dialogue = state.look_up_dialogue(&i("hello"));
        assert!(dialogue.text().contains("Hiya stranger!"));
    }

    #[test]
    fn test_do_action_change_room() {
        let (mut state, ..) = make_game();
        let id = t("WoodShed");
        state.current_room = Some(id.clone());
        assert!(state.current_room().variant().clone().is_none());
        assert_that!(state.do_action(&Action::ChangeRoom(
            ChangeRoom::builder()
                .name(i("close_door"))
                .description("".into())
                .room(
                    state
                        .world
                        .rooms()
                        .get(&id)
                        .unwrap()
                        .get(&Some(i("closed")))
                        .unwrap()
                        .clone()
                )
                .build()
        )))
        .is_true();
        assert_eq!(
            state.look_up_room(&id).unwrap().variant().clone(),
            Some(i("closed"))
        );
        assert_eq!(state.current_room().name().clone(), id);
        assert_eq!(state.current_room().variant().clone(), Some(i("closed")))
    }

    #[test]
    fn test_do_action_change_room_with_requirements_unmet() {
        let (mut state, items, ..) = make_game();
        let item = items.get(&i("lever")).unwrap().clone();
        let id = t("WoodShed");
        state.current_room = Some(id.clone());
        assert!(state.current_room().variant().clone().is_none());
        assert_that!(state.do_action(&Action::ChangeRoom(
            ChangeRoom::builder()
                .name(i("close_door"))
                .description("".into())
                .room(
                    state
                        .world
                        .rooms()
                        .get(&id)
                        .unwrap()
                        .get(&Some(i("closed")))
                        .unwrap()
                        .clone()
                )
                .required(item)
                .build()
        )))
        .is_false();
        assert_eq!(state.look_up_room(&id).unwrap().variant().clone(), None);
        assert_eq!(state.current_room().name().clone(), id);
        assert_eq!(state.current_room().variant().clone(), None)
    }

    #[test]
    fn test_do_action_change_room_with_requirements_nmet() {
        let (mut state, items, ..) = make_game();
        let item = items.get(&i("lever")).unwrap().clone();
        state.inventory.insert(item.clone());
        let id = t("WoodShed");
        state.current_room = Some(id.clone());
        assert!(state.current_room().variant().clone().is_none());
        assert_that!(state.do_action(&Action::ChangeRoom(
            ChangeRoom::builder()
                .name(i("close_door"))
                .description("".into())
                .room(
                    state
                        .world
                        .rooms()
                        .get(&id)
                        .unwrap()
                        .get(&Some(i("closed")))
                        .unwrap()
                        .clone()
                )
                .required(item)
                .build()
        )))
        .is_true();
        assert_eq!(state.current_room().name().clone(), id);
        assert_eq!(state.current_room().variant().clone(), Some(i("closed")))
    }

    #[test]
    fn test_do_action_teleport() {
        let (mut state, ..) = make_game();
        let id = t("Field");
        state.current_room = Some(t("WoodShed"));
        assert_that!(state.do_action(&Action::Teleport(
            Teleport::builder()
                .name(i("beam_me_up"))
                .description(
                    "You suddenly find yourself naked in an open field! The wind is cold...".into()
                )
                .room_name(id.clone())
                .build()
        )))
        .is_true();
        assert_eq!(state.current_room().name().clone(), id);
    }

    #[test]
    fn test_room_variant_requirement_met() {
        let (mut state, ..) = make_game();
        let room_name = t("WoodShed");
        let variant_name = i("closed");
        let req = Requirement::RoomVariant(make_room(room_name.clone(), None));
        assert!(state.requirement_met(&req));

        // Case 2: No entry in map, expected = Some(...) => false
        let req =
            Requirement::RoomVariant(make_room(room_name.clone(), Some(variant_name.clone())));
        assert!(!state.requirement_met(&req));

        // Case 3: Entry is None, expected = None => true
        state.active_room_variants.remove(&room_name);
        let req = Requirement::RoomVariant(make_room(room_name.clone(), None));
        assert!(state.requirement_met(&req));

        // Case 4: Entry = Some(x), expected = Some(x) => true
        let ident = variant_name.clone();
        state.active_room_variants.insert(room_name.clone(), ident);
        let req = Requirement::RoomVariant(make_room(room_name.clone(), Some(variant_name)));
        assert!(state.requirement_met(&req));

        // Case 5: Entry = Some(x), expected = Some(y) ≠ x => false
        let req = Requirement::RoomVariant(make_room(room_name, Some(i("different"))));
        assert!(!state.requirement_met(&req));
    }
}
