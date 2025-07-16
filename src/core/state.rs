use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use ini::Ini;
use tracing::info;

use crate::{config_parser, error};

use super::{Identifier, Title, World, entity::*};

#[derive(Debug)]
pub struct GameState {
    world: World,
    current_room: Title,
    inventory: HashSet<Rc<Item>>,
    active_room_variants: HashMap<Title, Identifier>,
}

// Game state data already verified in World
#[allow(clippy::expect_used)]
impl GameState {
    pub fn from_ini(ini: Ini) -> Result<Self, error::Application> {
        Ok(Self::new(config_parser::parse(ini)?))
    }
    pub fn new(world: World) -> Self {
        let current_room = world.title().start_room().clone();
        Self {
            world,
            current_room,
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
    pub fn theme(&self) -> Rc<Theme> {
        self.world.theme().clone()
    }
    pub fn language(&self) -> Rc<Language> {
        self.world.language().clone()
    }
    pub fn current_room(&self) -> Room<'_, Self> {
        self.lookup_room(&self.current_room).to_proxy(self)
    }
    pub fn has_inventory(&self) -> bool {
        !self.inventory.is_empty()
    }
    pub fn inventory(&self) -> Vec<String> {
        self.inventory
            .iter()
            .map(|i| i.description().to_string())
            .collect()
    }
    fn requirement_met(&self, requirement: &Requirement) -> bool {
        match requirement {
            Requirement::HasItem(needed_item) => self.inventory.contains(needed_item),
            Requirement::DoesNotHave(needed_item) => !self.inventory.contains(needed_item),
            Requirement::RoomVariant(expected) => {
                expected.variant() == &self.active_room_variants.get(expected.name()).cloned()
            }
        }
    }
    fn action_requirement_met(&self, action: &ActionEntity) -> bool {
        let mut required = Vec::new();
        use ActionEntity::*;
        match action {
            GiveItem(g) => {
                if let Some(r) = g.required() {
                    required.push(r);
                }
            }
            TakeItem(t) => {
                required.extend(t.items());
            }
            ChangeRoom(c) => {
                if let Some(r) = c.required() {
                    required.push(r);
                }
            }
            ReplaceItem(r) => {
                required.push(r.original());
            }
            Teleport(t) => {
                if let Some(r) = t.required() {
                    required.push(r);
                }
            }
            Sequence(s) => {
                if let Some(r) = s.required() {
                    required.push(r);
                }
            }
        }
        required.iter().all(|r| self.inventory.contains(&**r))
    }
    fn complete_action(&mut self, action: &ActionEntity) {
        use ActionEntity::*;
        match action {
            ChangeRoom(c) => {
                if let Some(r) = c.required() {
                    self.inventory.remove(r);
                }
                let room = c.room();
                if let Some(v) = room.variant() {
                    self.active_room_variants
                        .insert(room.name().clone(), v.clone());
                } else {
                    self.active_room_variants.remove(room.name());
                }
            }
            GiveItem(g) => {
                if let Some(r) = g.required() {
                    self.inventory.remove(r);
                }
                self.inventory.extend(g.items().clone());
            }
            TakeItem(t) => {
                for i in t.items() {
                    self.inventory.remove(i);
                }
            }
            ReplaceItem(r) => {
                self.inventory.remove(r.original());
                self.inventory.insert(r.replacement().clone());
            }
            Teleport(t) => {
                if let Some(r) = t.required() {
                    self.inventory.remove(r);
                }
                let room = self.lookup_room(t.room_name());
                self.enter_room(&room);
            }
            Sequence(s) => {
                if let Some(r) = s.required() {
                    self.inventory.remove(r);
                }
                let action_map = self.world.actions().clone();
                for id in s.actions() {
                    let a = action_map
                        .get(id)
                        .expect("All actions should be in the world!");
                    self.complete_action(a);
                }
            }
        }
    }
}

// Game state data already verified in World
#[allow(clippy::expect_used)]
impl Database for GameState {
    fn lookup_action(&self, name: &Identifier) -> Rc<ActionEntity> {
        self.world
            .actions()
            .get(name)
            .expect("All actions should be loaded.")
            .clone()
    }
    fn lookup_room(&self, name: &Title) -> Rc<RoomEntity> {
        let variant = self.active_room_variants.get(name).cloned();
        self.world
            .rooms()
            .get(name)
            .expect("All rooms should be loaded.")
            .get(&variant)
            .expect("All room variants should be loaded.")
            .clone()
    }
    fn lookup_dialogue(&self, id: &Identifier) -> Rc<DialogueEntity> {
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
    fn lookup_responses(&self, unfiltered: &[Rc<ResponseEntity>]) -> Vec<Rc<ResponseEntity>> {
        unfiltered
            .iter()
            .filter(|response| {
                response.requires().is_empty()
                    || response.requires().iter().all(|r| self.requirement_met(r))
            })
            .cloned()
            .collect()
    }
    fn enter_room(&mut self, room: &RoomEntity) {
        self.current_room = room.name().clone();
    }
    fn do_action(&mut self, action: &ActionEntity) -> bool {
        info!("do_action({action:#?})");
        if self.action_requirement_met(action) {
            self.complete_action(action);
            true
        } else {
            false
        }
    }
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use std::rc::Rc;

    use asserting::prelude::*;
    use indexmap::IndexMap;

    use crate::config_parser::test_utils::data::{
        action_map, character_map, dialogue_map, item_map, response_map_with_items, room_map,
    };
    use crate::config_parser::test_utils::{i, t};
    use crate::core::ItemMap;

    use super::*;

    fn make_room(name: Title, variant: Option<Identifier>) -> Rc<RoomEntity> {
        Rc::new(
            RoomEntity::builder()
                .name(name)
                .maybe_variant(variant)
                .description("Test".into())
                .characters(Vec::new())
                .exits(IndexMap::new())
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
        let world = World::try_new()
            .title(title)
            .theme(Theme::default())
            .language(Language::default())
            .actions(actions)
            .rooms(rooms)
            .dialogues(dialogues)
            .characters(characters.values().cloned().collect())
            .responses(responses.values().cloned().collect())
            .call()
            .unwrap();
        (GameState::new(world), items)
    }

    #[test]
    fn dialogue_responses_returns_only_defaults_when_no_requirements_are_met() {
        let (state, ..) = make_game();
        let dialogue = state.lookup_dialogue(&i("hello"));
        let responses = state.lookup_responses(dialogue.responses());
        assert_that!(responses).satisfies_with_message("Excludes unexpected responses", |list| {
            list.iter()
                .all(|r| !r.text().contains("key") && !r.text().contains("ring"))
        });
    }

    #[test]
    fn dialogue_responses_include_multiple_met_requirements() {
        let (mut state, items) = make_game();
        let required = [
            items.get(&i("ring")).unwrap().clone(),
            items.get(&i("key")).unwrap().clone(),
        ];
        state.inventory.extend(required);
        let dialogue = state.lookup_dialogue(&i("hello"));
        let responses = state.lookup_responses(dialogue.responses());
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
    fn dialogue_responses_includes_met_requirements() {
        let (mut state, items) = make_game();
        let required = items.get(&i("ring")).unwrap().clone();
        state.inventory.insert(required);
        let dialogue = state.lookup_dialogue(&i("hello"));
        let responses = state.lookup_responses(dialogue.responses());
        assert_that!(responses.clone())
            .satisfies_with_message("Includes expected responses", |list| {
                list.iter().any(|r| r.text().contains("ring"))
            });
        assert_that!(responses).satisfies_with_message("Excludes unexpected responses", |list| {
            list.iter().all(|r| !r.text().contains("key"))
        });
    }

    #[test]
    fn look_up_dialogue_returns_variant_if_requirement_met() {
        let (mut state, ..) = make_game();
        let dialogue_id = i("hello");
        state
            .active_room_variants
            .insert(t("WoodShed"), i("closed"));
        let dialogue = state.lookup_dialogue(&dialogue_id);
        assert_that!(dialogue.text()).contains("Who goes there");
    }

    #[test]
    fn look_up_dialogue_falls_back_to_default_variant() {
        let (state, ..) = make_game();
        let dialogue = state.lookup_dialogue(&i("hello"));
        assert!(dialogue.text().contains("Hiya stranger!"));
    }

    #[test]
    fn do_action_change_room() {
        let (mut state, ..) = make_game();
        let id = t("WoodShed");
        state.current_room = id.clone();
        {
            assert!(state.current_room().variant().clone().is_none());
        }
        assert_that!(
            state.do_action(&ActionEntity::ChangeRoom(
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
            ))
        )
        .is_true();
        {
            let current_room = state.current_room();
            assert_eq!(current_room.name().clone(), id);
            assert_eq!(current_room.variant().clone(), Some(i("closed")))
        }
    }

    #[test]
    fn do_action_change_room_with_requirements_unmet() {
        let (mut state, items, ..) = make_game();
        let item = items.get(&i("lever")).unwrap().clone();
        let id = t("WoodShed");
        state.current_room = id.clone();
        {
            let current_room = state.current_room();
            assert!(current_room.variant().clone().is_none());
        }
        assert_that!(
            state.do_action(&ActionEntity::ChangeRoom(
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
            ))
        )
        .is_false();
        {
            let current_room = state.current_room();
            assert_eq!(current_room.name().clone(), id);
            assert_eq!(current_room.variant().clone(), None);
        }
    }

    #[test]
    fn do_action_change_room_with_requirements_met() {
        let (mut state, items, ..) = make_game();
        let item = items.get(&i("lever")).unwrap().clone();
        state.inventory.insert(item.clone());
        let id = t("WoodShed");
        state.current_room = id.clone();
        {
            let current_room = state.current_room();
            assert!(current_room.variant().clone().is_none());
        }
        assert_that!(
            state.do_action(&ActionEntity::ChangeRoom(
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
            ))
        )
        .is_true();
        {
            let current_room = state.current_room();
            assert_eq!(current_room.name().clone(), id);
            assert_eq!(current_room.variant().clone(), Some(i("closed")));
        }
    }

    #[test]
    fn do_action_teleport() {
        let (mut state, ..) = make_game();
        let id = t("Field");
        state.current_room = t("WoodShed");
        assert_that!(
            state.do_action(&ActionEntity::Teleport(
                Teleport::builder()
                    .name(i("beam_me_up"))
                    .description(
                        "You suddenly find yourself naked in an open field! The wind is cold..."
                            .into()
                    )
                    .room_name(id.clone())
                    .build()
            ))
        )
        .is_true();
        {
            let current_room = state.current_room();
            assert_eq!(current_room.name().clone(), id);
        }
    }

    #[test]
    fn room_variant_requirement_met() {
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
