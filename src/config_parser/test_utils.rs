// Allowed in Tests
#![allow(clippy::unwrap_used)]

use crate::core::{Identifier, Title};

pub fn t(s: &str) -> Title {
    s.parse().unwrap()
}

pub fn i(s: &str) -> Identifier {
    s.parse().unwrap()
}

pub mod data {
    use super::*;
    use std::{collections::HashMap, rc::Rc};

    use crate::{
        config_parser::types::{CharacterMap, ItemMap, ResponseMap},
        core::{
            Action, ActionMap, ChangeRoom, Character, Dialogue, DialogueMap, GiveItem, Item,
            ReplaceItem, Requirement, Response, Room, RoomMap, TakeItem,
        },
    };

    pub fn character_map() -> CharacterMap {
        CharacterMap::from([
            (
                t("NeighborFrank"),
                Rc::new(Character::new(t("NeighborFrank"), i("hello"))),
            ),
            (
                t("CuriousCalvin"),
                Rc::new(Character::new(t("CuriousCalvin"), i("curious"))),
            ),
            (
                t("BlueBird"),
                Rc::new(Character::new(t("BlueBird"), i("chirp"))),
            ),
        ])
    }

    pub fn response_map(action_map: &ActionMap) -> ResponseMap {
        ResponseMap::from([
            (
                i("goodbye"),
                Rc::new(
                    Response::builder()
                        .text("Goodbye.".into())
                        .requires(vec![])
                        .build(),
                ),
            ),
            (
                i("hello"),
                Rc::new(
                    Response::builder()
                        .text("Hello!".into())
                        .requires(vec![])
                        .build(),
                ),
            ),
            (
                i("im_sorry"),
                Rc::new(
                    Response::builder()
                        .text("No, I don't I do...".into())
                        .requires(vec![])
                        .build(),
                ),
            ),
            (
                i("sure"),
                Rc::new(
                    Response::builder()
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
                i("half_eaten_apple"),
                Rc::new(Item::new(
                    i("half_eaten_apple"),
                    "A half eaten brown apple.".into(),
                )),
            ),
        ])
    }

    pub fn room_map(items: &ItemMap, with_actions: bool) -> RoomMap {
        let woodshed = Rc::new(
            Room::builder()
                .name(t("WoodShed"))
                .description("A dusty shed full of tools, lumber, and a giant table saw.".into())
                .items(vec![items.get(&i("lever")).unwrap().clone()])
                .characters(vec![])
                .exits(HashMap::from([(i("north"), "Field".parse().unwrap())]))
                .actions(if with_actions {
                    vec![i("pull_lever")]
                } else {
                    vec![]
                })
                .build(),
        );
        let woodshed_closed = Rc::new(
            Room::builder()
                .name(t("WoodShed"))
                .variant(i("closed"))
                .description("Its really dark and dusty in here.".into())
                .items(vec![])
                .characters(vec![])
                .exits(HashMap::from([(
                    "north".parse().unwrap(),
                    "Field".parse().unwrap(),
                )]))
                .actions(vec![])
                .build(),
        );
        let field = Rc::new(
            Room::builder()
                .name(t("Field"))
                .description("A wide open grassy field.".into())
                .items(vec![])
                .characters(vec![])
                .exits(HashMap::from([(
                    "north".parse().unwrap(),
                    "Field".parse().unwrap(),
                )]))
                .actions(vec![])
                .build(),
        );
        RoomMap::from([
            (
                woodshed.name().clone(),
                HashMap::from([
                    (woodshed.variant().clone(), woodshed),
                    (woodshed_closed.variant().clone(), woodshed_closed),
                ]),
            ),
            (
                field.name().clone(),
                HashMap::from([(field.variant().clone(), field)]),
            ),
        ])
    }

    pub fn action_map(room_map: &RoomMap, item_map: &ItemMap) -> ActionMap {
        ActionMap::from([(
            i("open_chest"),
            Rc::new(Action::GiveItem(
                GiveItem::builder()
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
            Rc::new(Action::ReplaceItem(
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
                Action::TakeItem(
                    TakeItem::builder()
                        .name(i("robbed"))
                        .description("You wake up groggy laying in vomit. You pat youself down only to notice that your ring is missing!".into())
                        .items(vec![item_map.get(&i("ring")).unwrap().clone()])
                        .build()
                )
            )
        ), (
            i("pull_lever"),
            Rc::new(
                Action::ChangeRoom(
                    ChangeRoom::builder()
                        .name(i("pull_lever"))
                        .description("You insert the lever into the slot and pull it back. Two hefty ropes snap and the barn doors slam shut! It's dark in here!".into())
                        .required(item_map.get(&i("lever")).unwrap().clone())
                        .room(room_map.get(&t("WoodShed")).unwrap().get(&Some(i("closed"))).unwrap().clone())
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
        DialogueMap::from([
            (
                i("hello"),
                HashMap::from([
                    (
                        None,
                        Rc::new(
                            Dialogue::builder()
                                .text("Hiya stranger!".into())
                                .responses(vec![
                                    response_map.get(&i("hello")).unwrap().clone(),
                                    response_map.get(&i("goodbye")).unwrap().clone(),
                                ])
                                .requires(vec![])
                                .build(),
                        ),
                    ),
                    (
                        Some(i("scared")),
                        Rc::new(
                            Dialogue::builder()
                                .text("Who goes there? I can't see ya, but I can smell ya!".into())
                                .responses(vec![])
                                .requires(vec![Requirement::RoomVariant(room_map.get(&t("WoodShed")).unwrap().get(&Some(i("closed"))).unwrap().clone())])
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
                        Dialogue::builder()
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
                        Dialogue::builder()
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
                        Dialogue::builder()
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
