use std::{
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use ini::Ini;
use tracing::info;

use crate::{config_parser, error};

use super::{Lookup, Update, World, entity::*};

#[derive(Debug)]
pub struct GameState {
    world: World,
    current_room: RoomId,
    inventory: BTreeSet<ItemId>,
    active_room_variants: HashMap<RoomId, RoomVariantId>,
}

impl GameState {
    pub fn from_ini(ini: Ini) -> Result<Self, error::Application> {
        Ok(Self::new(config_parser::parse(ini)?))
    }
    pub fn new(world: World) -> Self {
        let current_room = *world.title().start_room();
        Self {
            world,
            current_room,
            inventory: BTreeSet::new(),
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
        self.world.theme()
    }
    pub fn language(&self) -> Rc<Language> {
        self.world.language()
    }
    pub fn current_room(&self) -> Room<'_, Self> {
        self.current_room.into_proxy(self)
    }
    pub fn has_inventory(&self) -> bool {
        !self.inventory.is_empty()
    }
    pub fn inventory(&self) -> Vec<String> {
        self.inventory
            .iter()
            .map(|i| self.world.item(*i).description.to_string())
            .collect()
    }
    fn requirement_met(&self, requirement: &Requirement) -> bool {
        match requirement {
            Requirement::HasItem(needed_item) => self.inventory.contains(needed_item),
            Requirement::DoesNotHave(needed_item) => !self.inventory.contains(needed_item),
            Requirement::RoomVariant(room, variant) => {
                variant == &self.active_room_variants.get(room).cloned()
            }
        }
    }
    fn action_requirement_met(&self, action: &ActionEntity) -> bool {
        let mut required = Vec::new();
        use ActionEntity::*;
        match action {
            GiveItem(g) => {
                if let Some(r) = g.required {
                    required.push(r);
                }
            }
            TakeItem(t) => {
                required.extend(&t.items);
            }
            ChangeRoom(c) => {
                if let Some(r) = c.required {
                    required.push(r);
                }
            }
            ReplaceItem(r) => {
                required.push(r.original);
            }
            Teleport(t) => {
                if let Some(r) = t.required {
                    required.push(r);
                }
            }
            Sequence(s) => {
                if let Some(r) = s.required {
                    required.push(r);
                }
            }
        }
        required.iter().all(|r| self.inventory.contains(r))
    }
    fn complete_action(&mut self, action: &ActionEntity) {
        use ActionEntity::*;
        match action {
            ChangeRoom(c) => {
                if let Some(r) = c.required {
                    self.inventory.remove(&r);
                }
                if let Some(v) = c.variant {
                    self.active_room_variants.insert(c.room, v);
                } else {
                    self.active_room_variants.remove(&c.room);
                }
            }
            GiveItem(g) => {
                if let Some(r) = g.required {
                    self.inventory.remove(&r);
                }
                self.inventory.extend(g.items.clone());
            }
            TakeItem(t) => {
                for i in &t.items {
                    self.inventory.remove(i);
                }
            }
            ReplaceItem(r) => {
                self.inventory.remove(&r.original);
                self.inventory.insert(r.replacement);
            }
            Teleport(t) => {
                if let Some(r) = t.required {
                    self.inventory.remove(&r);
                }
                self.enter_room(t.room);
            }
            Sequence(s) => {
                if let Some(r) = s.required {
                    self.inventory.remove(&r);
                }
                for id in &s.actions {
                    let a = self.world.action(*id).clone();
                    self.complete_action(&a);
                }
            }
        }
    }
}
impl Lookup for GameState {
    fn lookup_action(&self, id: ActionId) -> &ActionEntity {
        self.world.action(id)
    }
    fn lookup_room(&self, id: RoomId) -> &RoomVariantEntity {
        let variant_id = self.active_room_variants.get(&id).copied();
        self.world.room(id, variant_id)
    }
    fn lookup_dialogue(&self, id: DialogueId) -> &DialogueVariantEntity {
        let variants = &self.world.dialogue(id);
        variants
            .iter()
            .filter_map(|dialogue| {
                let count = dialogue
                    .requires
                    .iter()
                    .filter(|req| self.requirement_met(req))
                    .count();

                if dialogue.requires.len() != count || count == 0 {
                    None
                } else {
                    Some((count, dialogue))
                }
            })
            .max_by_key(|(k, _)| *k)
            .map(|(_, v)| v)
            .unwrap_or_else(|| {
                #[allow(clippy::expect_used)]
                variants
                    .first()
                    .expect("All dialogues should have a default variant")
            })
    }
    fn filter_responses(&self, unfiltered: &[ResponseId]) -> Vec<ResponseId> {
        unfiltered
            .iter()
            .filter_map(|id| {
                let response = &self.world.response(*id);
                if response.requires.is_empty()
                    || response.requires.iter().all(|r| self.requirement_met(r))
                {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }
    fn lookup_character(&self, id: CharacterId) -> &CharacterEntity {
        self.world.character(id)
    }
    fn lookup_response(&self, id: ResponseId) -> &ResponseEntity {
        self.world.response(id)
    }
}
impl Update for GameState {
    fn enter_room(&mut self, id: RoomId) {
        self.current_room = id;
    }
    fn do_action(&mut self, id: ActionId) -> bool {
        let action = self.world.action(id).clone();
        info!("do_action({action:#?})");
        if self.action_requirement_met(&action) {
            self.complete_action(&action);
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
    use bon::builder;
    use indexmap::IndexMap;
    use rstest::*;

    use super::*;

    #[fixture]
    fn game() -> GameState {
        make_game().call()
    }

    #[fixture]
    fn dialogue_game() -> GameState {
        let dialogues = vec![vec![
            DialogueVariantEntity {
                text: "Hiya stranger!".into(),
                requires: vec![],
                responses: vec![0.into(), 1.into()],
            },
            DialogueVariantEntity {
                text: "Who goes there?".into(),
                requires: vec![Requirement::HasItem(1.into())],
                responses: vec![0.into(), 1.into()],
            },
        ]];
        let responses = vec![
            ResponseEntity {
                text: "Hello!".into(),
                requires: vec![],
                leads_to: None,
                triggers: None,
            },
            ResponseEntity {
                text: "I have the ring.".into(),
                requires: vec![Requirement::HasItem(1.into())],
                leads_to: None,
                triggers: None,
            },
        ];
        make_game().dialogues(dialogues).responses(responses).call()
    }

    #[fixture]
    fn response_game() -> GameState {
        let dialogues = vec![vec![DialogueVariantEntity {
            text: "Hiya stranger!".into(),
            requires: vec![],
            responses: vec![0.into(), 1.into()],
        }]];
        let responses = vec![
            ResponseEntity {
                text: "Hello!".into(),
                requires: vec![],
                leads_to: None,
                triggers: None,
            },
            ResponseEntity {
                text: "I have the ring.".into(),
                requires: vec![Requirement::HasItem(1.into())],
                leads_to: None,
                triggers: None,
            },
        ];
        make_game().dialogues(dialogues).responses(responses).call()
    }

    #[builder]
    fn make_game(
        items: Option<Vec<Item>>,
        actions: Option<Vec<ActionEntity>>,
        rooms: Option<Vec<Vec<RoomVariantEntity>>>,
        dialogues: Option<Vec<DialogueEntity>>,
        characters: Option<Vec<CharacterEntity>>,
        responses: Option<Vec<ResponseEntity>>,
    ) -> GameState {
        let world = World::builder()
            .title(
                GameTitle::builder()
                    .title("".into())
                    .greeting("".into())
                    .credits("".into())
                    .start_room(0usize.into())
                    .build(),
            )
            .language(Language::default())
            .theme(Theme::default())
            .items(items.unwrap_or_else(|| {
                vec![
                    Item {
                        name: "key".parse().unwrap(),
                        description: "a key".into(),
                    },
                    Item {
                        name: "ring".parse().unwrap(),
                        description: "a ring".into(),
                    },
                ]
            }))
            .actions(actions.unwrap_or_default())
            .rooms(rooms.unwrap_or_else(|| {
                vec![
                    vec![
                        RoomVariantEntity {
                            name: "WoodShed".into(),
                            description: "A Shed".into(),
                            characters: vec![],
                            exits: IndexMap::new(),
                            actions: vec![],
                        },
                        RoomVariantEntity {
                            name: "WoodShed".into(),
                            description: "A Shed variant".into(),
                            characters: vec![],
                            exits: IndexMap::new(),
                            actions: vec![],
                        },
                    ],
                    vec![RoomVariantEntity {
                        name: "Field".into(),
                        description: "An open field".into(),
                        characters: vec![],
                        exits: IndexMap::new(),
                        actions: vec![],
                    }],
                ]
            }))
            .dialogues(dialogues.unwrap_or_default())
            .characters(characters.unwrap_or_default())
            .responses(responses.unwrap_or_default())
            .build();

        let mut game = GameState::new(world);
        game.inventory.insert(0.into());
        game
    }

    #[rstest]
    #[case::has_item_true(Requirement::HasItem(0.into()), true)]
    #[case::has_item_false(Requirement::HasItem(1.into()), false)]
    #[case::does_not_have_item_true(Requirement::DoesNotHave(1.into()), true)]
    #[case::does_not_have_item_false(Requirement::DoesNotHave(0.into()), false)]
    #[case::room_variant_true(Requirement::RoomVariant(0.into(), None), true)]
    #[case::room_variant_false(Requirement::RoomVariant(0.into(), Some(1.into())), false)]
    fn requirement_met(game: GameState, #[case] req: Requirement, #[case] expected: bool) {
        assert_eq!(
            game.requirement_met(&req),
            expected,
            "Failed for requirement {req:?}"
        );
    }

    #[rstest]
    #[case::positive(Requirement::RoomVariant(0.into(), None), false)]
    #[case::negative(Requirement::RoomVariant(0.into(), Some(1.into())), true)]
    fn requirement_met_room_variant(
        mut game: GameState,
        #[case] req: Requirement,
        #[case] expected: bool,
    ) {
        game.active_room_variants.insert(0.into(), 1.into());
        assert_eq!(
            game.requirement_met(&req),
            expected,
            "Failed for requirement {req:?}"
        );
    }

    #[rstest]
    #[case::required_found(Some(0.into()), true)]
    #[case::required_not_found(Some(1.into()), false)]
    #[case::no_requirement(None, true)]
    fn action_requirement_met_give_item(
        game: GameState,
        #[case] required: Option<ItemId>,
        #[case] expected: bool,
    ) {
        let action = ActionEntity::GiveItem(GiveItem {
            name: "give_ring".parse().unwrap(),
            description: "Give the ring".into(),
            items: vec![1.into()],
            required,
        });

        assert_eq!(game.action_requirement_met(&action), expected);
    }

    #[rstest]
    #[case::give_item(
        ActionEntity::GiveItem(GiveItem {
            name: "give_ring".parse().unwrap(),
            description: "".into(),
            items: vec![1.into()],
            required: Some(0.into()),
        }),
        vec![1.into()],
        vec![0.into()]
    )]
    #[case::replace_item(
        ActionEntity::ReplaceItem(ReplaceItem {
            name: "replace_key_with_ring".parse().unwrap(),
            description: "".into(),
            original: 0.into(),
            replacement: 1.into(),
        }),
        vec![1.into()],
        vec![0.into()]
    )]
    #[case::take_item(
        ActionEntity::TakeItem(TakeItem {
            name: "take_key".parse().unwrap(),
            description: "".into(),
            items: vec![0.into()],
        }),
        vec![],
        vec![0.into()]
    )]
    fn complete_action_inventory_changes(
        #[case] action: ActionEntity,
        #[case] expected_add: Vec<ItemId>,
        #[case] expected_remove: Vec<ItemId>,
    ) {
        let mut game = make_game().actions(vec![action.clone()]).call();
        game.complete_action(&action);

        for item in expected_add {
            assert!(
                game.inventory.contains(&item),
                "Item {item:?} should be added"
            );
        }
        for item in expected_remove {
            assert!(
                !game.inventory.contains(&item),
                "Item {item:?} should be removed"
            );
        }
    }

    #[rstest]
    fn complete_action_change_room() {
        let action = ActionEntity::ChangeRoom(ChangeRoom {
            name: "close_door".parse().unwrap(),
            description: "".into(),
            room: 0.into(),
            variant: Some(1.into()),
            required: Some(0.into()),
        });
        let mut game = make_game().actions(vec![action.clone()]).call();
        game.complete_action(&action);
        assert_eq!(
            game.active_room_variants.get(&0.into()),
            Some(&1.into()),
            "Expected room variant to be updated"
        );
    }

    #[rstest]
    fn complete_action_teleport() {
        let action = ActionEntity::Teleport(Teleport {
            name: "beam_me_up".parse().unwrap(),
            description: "".into(),
            room: 1.into(),
            required: Some(0.into()),
        });
        let mut game = make_game().actions(vec![action.clone()]).call();
        game.complete_action(&action);
        assert_eq!(game.current_room, 1.into(), "Expected teleport to new room");
    }

    #[rstest]
    fn complete_action_sequence_executes_all_actions() {
        let sequence = ActionEntity::Sequence(Sequence {
            name: "do_multiple".parse().unwrap(),
            description: "".into(),
            actions: vec![ActionId::from(0), ActionId::from(1)],
            required: None,
        });
        let take_key = ActionEntity::TakeItem(TakeItem {
            name: "take_key".parse().unwrap(),
            description: "".into(),
            items: vec![0.into()],
        });
        let give_ring = ActionEntity::GiveItem(GiveItem {
            name: "give_ring".parse().unwrap(),
            description: "".into(),
            items: vec![1.into()],
            required: None,
        });
        let mut game = make_game()
            .actions(vec![take_key, give_ring, sequence.clone()])
            .call();

        game.complete_action(&sequence);

        assert!(!game.inventory.contains(&0.into()), "Key should be removed");
        assert!(game.inventory.contains(&1.into()), "Ring should be added");
    }

    #[rstest]
    fn lookup_dialogue_returns_default_variant_when_no_requirements_met(dialogue_game: GameState) {
        let dialogue = dialogue_game.lookup_dialogue(0usize.into());
        assert_eq!(dialogue.text, "Hiya stranger!");
    }

    #[rstest]
    fn lookup_dialogue_returns_conditional_variant_when_requirements_met(
        mut dialogue_game: GameState,
    ) {
        dialogue_game.inventory.insert(1.into());
        let dialogue = dialogue_game.lookup_dialogue(0usize.into());
        assert_eq!(dialogue.text, "Who goes there?");
    }

    #[rstest]
    fn filter_responses_excludes_responses_with_unmet_requirements(response_game: GameState) {
        let filtered = response_game.filter_responses(&[0.into(), 1.into()]);
        assert_eq!(filtered.len(), 1, "Only one response should be allowed");
        let allowed_response = response_game.lookup_response(filtered[0]);
        assert_eq!(allowed_response.text, "Hello!");
    }

    #[rstest]
    fn filter_responses_includes_all_when_requirements_met(mut response_game: GameState) {
        response_game.inventory.insert(1.into());
        let filtered = response_game.filter_responses(&[0.into(), 1.into()]);
        assert_eq!(filtered.len(), 2, "Both responses should be allowed");
    }
}
