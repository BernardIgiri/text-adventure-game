// Allowed in Tests
#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::core::{Identifier, Title};

pub fn t(s: &str) -> Title {
    s.parse().unwrap()
}

pub fn i(s: &str) -> Identifier {
    s.parse().unwrap()
}

pub mod data {
    use indexmap::IndexMap;

    use super::*;
    use std::{collections::HashMap, rc::Rc};

    use crate::{
        config_parser::types::{CharacterMap, ItemMap, ResponseMap},
        core::{
            ActionEntity, ActionMap, ChangeRoom, CharacterEntity, DialogueEntity, DialogueMap,
            GiveItem, Item, ReplaceItem, Requirement, ResponseEntity, RoomEntity, RoomMap,
            Sequence, TakeItem,
        },
    };

    pub trait TakeClone<T> {
        fn take(&self, k: &str) -> &Rc<T>;
        fn take_clone(&self, k: &str) -> Rc<T> {
            self.take(k).clone()
        }
    }
    pub trait TakeCloneVariant<T> {
        fn take(&self, n: &str, v: Option<&str>) -> &Rc<T>;
        fn take_clone(&self, n: &str, v: Option<&str>) -> Rc<T> {
            self.take(n, v).clone()
        }
    }

    impl TakeClone<Item> for ItemMap {
        fn take(&self, k: &str) -> &Rc<Item> {
            self.get(&i(k)).expect("mock data should be present")
        }
    }
    impl TakeClone<CharacterEntity> for CharacterMap {
        fn take(&self, k: &str) -> &Rc<CharacterEntity> {
            self.get(&t(k)).expect("mock data should be present")
        }
    }
    impl TakeCloneVariant<RoomEntity> for RoomMap {
        fn take(&self, n: &str, v: Option<&str>) -> &Rc<RoomEntity> {
            self.get(&t(n))
                .and_then(|m| m.get(&v.map(i)))
                .expect("mock data should be present")
        }
    }
    impl TakeCloneVariant<DialogueEntity> for DialogueMap {
        fn take(&self, n: &str, v: Option<&str>) -> &Rc<DialogueEntity> {
            self.get(&i(n))
                .and_then(|m| m.get(&v.map(i)))
                .expect("mock data should be present")
        }
    }
    impl TakeClone<ResponseEntity> for ResponseMap {
        fn take(&self, k: &str) -> &Rc<ResponseEntity> {
            self.get(&i(k)).expect("mock data should be present")
        }
    }
    impl TakeClone<ActionEntity> for ActionMap {
        fn take(&self, k: &str) -> &Rc<ActionEntity> {
            self.get(&i(k)).expect("mock data should be present")
        }
    }

    pub fn character_map() -> CharacterMap {
        CharacterMap::from([
            (
                t("NeighborFrank"),
                Rc::new(
                    CharacterEntity::builder()
                        .name(t("NeighborFrank"))
                        .start_dialogue(i("hello"))
                        .build(),
                ),
            ),
            (
                t("CuriousCalvin"),
                Rc::new(
                    CharacterEntity::builder()
                        .name(t("CuriousCalvin"))
                        .start_dialogue(i("curious"))
                        .build(),
                ),
            ),
            (
                t("BlueBird"),
                Rc::new(
                    CharacterEntity::builder()
                        .name(t("BlueBird"))
                        .start_dialogue(i("chirp"))
                        .build(),
                ),
            ),
        ])
    }

    pub fn response_map_with_items(action_map: &ActionMap, item_map: &ItemMap) -> ResponseMap {
        ResponseMap::from([
            (
                i("goodbye"),
                Rc::new(
                    ResponseEntity::builder()
                        .text("Goodbye.".into())
                        .requires(vec![])
                        .build(),
                ),
            ),
            (
                i("hello"),
                Rc::new(
                    ResponseEntity::builder()
                        .text("Hello!".into())
                        .requires(vec![])
                        .build(),
                ),
            ),
            (
                i("im_sorry"),
                Rc::new(
                    ResponseEntity::builder()
                        .text("No, I don't I do...".into())
                        .requires(vec![])
                        .build(),
                ),
            ),
            (
                i("sure"),
                Rc::new(
                    ResponseEntity::builder()
                        .text("Well, one drink couldn't hurt!".into())
                        .requires(vec![])
                        .triggers(action_map.get(&i("robbed")).unwrap().clone())
                        .build(),
                ),
            ),
            (
                i("you_have_it"),
                Rc::new(
                    ResponseEntity::builder()
                        .text("You found the ring!".into())
                        .requires(vec![Requirement::HasItem(
                            item_map.get(&i("ring")).unwrap().clone(),
                        )])
                        .build(),
                ),
            ),
            (
                i("you_have_both"),
                Rc::new(
                    ResponseEntity::builder()
                        .text("You found the ring and the key!".into())
                        .requires(vec![
                            Requirement::HasItem(item_map.get(&i("ring")).unwrap().clone()),
                            Requirement::HasItem(item_map.get(&i("key")).unwrap().clone()),
                        ])
                        .build(),
                ),
            ),
        ])
    }

    pub fn response_map(action_map: &ActionMap) -> ResponseMap {
        ResponseMap::from([
            (
                i("goodbye"),
                Rc::new(
                    ResponseEntity::builder()
                        .text("Goodbye.".into())
                        .requires(vec![])
                        .build(),
                ),
            ),
            (
                i("hello"),
                Rc::new(
                    ResponseEntity::builder()
                        .text("Hello!".into())
                        .requires(vec![])
                        .build(),
                ),
            ),
            (
                i("im_sorry"),
                Rc::new(
                    ResponseEntity::builder()
                        .text("No, I don't I do...".into())
                        .requires(vec![])
                        .build(),
                ),
            ),
            (
                i("sure"),
                Rc::new(
                    ResponseEntity::builder()
                        .text("Well, one drink couldn't hurt!".into())
                        .requires(vec![])
                        .triggers(action_map.get(&i("robbed")).unwrap().clone())
                        .build(),
                ),
            ),
        ])
    }

    pub fn item_map() -> ItemMap {
        ItemMap::from([
            (i("ring"), Rc::new(Item::new(i("ring"), "Shiny!".into()))),
            (
                i("lever"),
                Rc::new(Item::new(i("lever"), "A big red lever.".into())),
            ),
            (
                i("valve"),
                Rc::new(Item::new(i("valve"), "A creaky old valve.".into())),
            ),
            (
                i("key"),
                Rc::new(Item::new(i("key"), "A dingy old key.".into())),
            ),
            (
                i("silver_coin"),
                Rc::new(Item::new(i("silver_coin"), "I'm rich!".into())),
            ),
            (
                i("lamp"),
                Rc::new(Item::new(i("lamp"), "It's so bright!".into())),
            ),
            (
                i("half_eaten_apple"),
                Rc::new(Item::new(
                    i("half_eaten_apple"),
                    "A half eaten brown apple.".into(),
                )),
            ),
        ])
    }

    pub fn room_map(with_actions: bool) -> RoomMap {
        let woodshed = Rc::new(
            RoomEntity::builder()
                .name(t("WoodShed"))
                .description("A dusty shed full of tools, lumber, and a giant table saw.".into())
                .characters(vec![])
                .exits(IndexMap::from([(i("north"), "Field".parse().unwrap())]))
                .actions(if with_actions {
                    vec![i("pull_lever")]
                } else {
                    vec![]
                })
                .build(),
        );
        let woodshed_closed = Rc::new(
            RoomEntity::builder()
                .name(t("WoodShed"))
                .variant(i("closed"))
                .description("Its really dark and dusty in here.".into())
                .characters(vec![])
                .exits(IndexMap::from([(
                    "north".parse().unwrap(),
                    "Field".parse().unwrap(),
                )]))
                .actions(vec![])
                .build(),
        );
        let field = Rc::new(
            RoomEntity::builder()
                .name(t("Field"))
                .description("A wide open grassy field.".into())
                .characters(vec![])
                .exits(IndexMap::from([(
                    "north".parse().unwrap(),
                    "Field".parse().unwrap(),
                )]))
                .actions(vec![])
                .build(),
        );
        RoomMap::from([
            (
                t("WoodShed"),
                HashMap::from([(None, woodshed), (Some(i("closed")), woodshed_closed)]),
            ),
            (t("Field"), HashMap::from([(None, field)])),
        ])
    }

    pub fn action_map(room_map: &RoomMap, item_map: &ItemMap) -> ActionMap {
        ActionMap::from([(
            i("open_chest"),
            Rc::new(ActionEntity::TakeItem(
                TakeItem::builder()
                    .name(i("open_chest"))
                    .description("You carefully open the chest with an intense curiosity!".into())
                    .items(vec![
                        item_map.get(&i("half_eaten_apple")).unwrap().clone(),
                        item_map.get(&i("silver_coin")).unwrap().clone(),
                    ])
                    .build(),
            )),
        ), (
            i("look_closer"),
            Rc::new(ActionEntity::ReplaceItem(
                ReplaceItem::builder()
                    .name(i("look_closer"))
                    .description("You lean in to see what's inside the vase. Then out of no where a monkey snatches your apple knocking you over. You tumble into the bookshelf, only for the key to fall right into your hands!".into())
                    .original(item_map.get(&i("half_eaten_apple")).unwrap().clone())
                    .replacement(item_map.get(&i("key")).unwrap().clone())
                    .build()
            )),
        ), (
            i("robbed"),
            Rc::new(
                ActionEntity::GiveItem(
                    GiveItem::builder()
                        .name(i("robbed"))
                        .description("You wake up groggy laying in vomit. You pat youself down only to notice that your ring is missing!".into())
                        .items(vec![item_map.get(&i("ring")).unwrap().clone()])
                        .build()
                )
            )
        ), (
            i("pull_lever"),
            Rc::new(
                ActionEntity::ChangeRoom(
                    ChangeRoom::builder()
                        .name(i("pull_lever"))
                        .description("You insert the lever into the slot and pull it back. Two hefty ropes snap and the barn doors slam shut! It's dark in here!".into())
                        .required(item_map.get(&i("lever")).unwrap().clone())
                        .room(room_map.get(&t("WoodShed")).unwrap().get(&Some(i("closed"))).unwrap().clone())
                        .build()
                )
            )
        ), (
            i("open_door"),
            Rc::new(
                ActionEntity::ChangeRoom(
                    ChangeRoom::builder()
                        .name(i("open_door"))
                        .description("The door bursts open".into())
                        .room(room_map.get(&t("WoodShed")).unwrap().get(&None).unwrap().clone())
                        .build()
                )
            )
        ), (
            i("multiple"),
            Rc::new(
                ActionEntity::Sequence(
                    Sequence::builder()
                    .name(i("multiple"))
                    .description("Multiple things happen".into())
                    .required(item_map.get(&i("ring")).unwrap().clone())
                    .actions(vec![i("open_door"),i("open_chest")])
                    .build()
                )
            )
        )])
    }

    pub fn dialogue_map(
        response_map: &ResponseMap,
        item_map: &ItemMap,
        room_map: &RoomMap,
    ) -> DialogueMap {
        let mut hello_responses = vec![
            response_map.get(&i("hello")).unwrap().clone(),
            response_map.get(&i("goodbye")).unwrap().clone(),
        ];
        if response_map.contains_key(&i("you_have_it")) {
            hello_responses.push(response_map.get(&i("you_have_it")).unwrap().clone());
            hello_responses.push(response_map.get(&i("you_have_both")).unwrap().clone());
        }
        DialogueMap::from([
            (
                i("hello"),
                HashMap::from([
                    (
                        None,
                        Rc::new(
                            DialogueEntity::builder()
                                .text("Hiya stranger!".into())
                                .responses(hello_responses)
                                .requires(vec![])
                                .build(),
                        ),
                    ),
                    (
                        Some(i("scared")),
                        Rc::new(
                            DialogueEntity::builder()
                                .text("Who goes there? I can't see ya, but I can smell ya!".into())
                                .responses(vec![])
                                .requires(vec![Requirement::RoomVariant(room_map.get(&t("WoodShed")).unwrap().get(&Some(i("closed"))).unwrap().clone())])
                                .build(),
                        ),
                    ),
                    (
                        Some(i("relieved")),
                        Rc::new(
                            DialogueEntity::builder()
                                .text("Whew! Thank you buddy! I was scared for a second! I think they way out is this way!".into())
                                .responses(vec![])
                                .requires(vec![
                                    Requirement::RoomVariant(
                                        room_map.get(&t("WoodShed")).unwrap().get(&Some(i("closed"))).unwrap().clone()),
                                    Requirement::HasItem(item_map.get(&i("lamp")).unwrap().clone())
                                ])
                                .build(),
                        ),
                    ),
                ]),
            ),
            (
                i("curious"),
                HashMap::from([(
                    None,
                    Rc::new(
                        DialogueEntity::builder()
                            .text("Hey, I remember you from somewhere -long pause- yeah, I we used to be neighbors! Remember the guy with the loud drunk guy who was always screaming at kids?".into())
                            .responses(vec![
                                response_map.get(&i("im_sorry")).unwrap().clone(),
                            ])
                            .requires(vec![Requirement::HasItem(
                                item_map.get(&i("ring")).unwrap().clone(),
                            )])
                            .build(),
                    ),
                )]),
            ),
            (
                i("trick"),
                HashMap::from([(
                    None,
                    Rc::new(
                        DialogueEntity::builder()
                            .text("Don't worry buddy! We all forget things. Hey, how about a drink for old times sake?".into())
                            .responses(vec![
                                response_map.get(&i("sure")).unwrap().clone(),
                            ])
                            .requires(vec![Requirement::HasItem(
                                item_map.get(&i("ring")).unwrap().clone(),
                            )])
                            .build(),
                    ),
                )]),
            ),
            (
                i("chirp"),
                HashMap::from([(
                    None,
                    Rc::new(
                        DialogueEntity::builder()
                            .text("The bird chatters melodiously; chirp, chirp!".into())
                            .responses(vec![
                            ])
                            .requires(vec![])
                            .build(),
                    ),
                )]),
            ),
        ])
    }
}
