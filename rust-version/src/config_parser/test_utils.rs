// Allowed in Tests
#![allow(clippy::unwrap_used)]

use crate::world::{Identifier, Title};

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
        world::{
            Action, ActionMap, ChangeRoom, Character, DialogueMap, GiveItem, Item, ReplaceItem,
            Response, Room, RoomMap, TakeItem,
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

    pub fn response_map() -> ResponseMap {
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
                "hello".parse().unwrap(),
                Rc::new(
                    Response::builder()
                        .text("Hello!".into())
                        .requires(vec![])
                        .build(),
                ),
            ),
            // (
            //     "chirp".parse().unwrap(),
            //     Rc::new(
            //         Response::builder()
            //             .text("The bird chatters melodiously; chirp, chirp!".into())
            //             .requires(vec![])
            //             .build(),
            //     ),
            // ),
            // (
            //     "curious".parse().unwrap(),
            //     Rc::new(
            //         Response::builder()
            //             .text("That's a shiny ring! May I hold it?".into())
            //             .requires(vec![])
            //             .build(),
            //     ),
            // ),
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

    /*
    TODO! cleanup
    let actions = parse_actions(ini.iter(), &rooms, &items)?;
    let dialogues = parse_dialogues(ini.iter(), &responses, &items, &rooms)?;
    */
    #[allow(dead_code, unused_variables)]
    pub fn action_map(room_map: &RoomMap, item_map: &ItemMap) -> ActionMap {
        // TODO! ReplaceItem, TakeItem, ChangeRoom
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
                    .original(item_map.get(&i("apple")).unwrap().clone())
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
                        .required(item_map.get(&i("pull_lever")).unwrap().clone())
                        .room(room_map.get(&t("WoodShed")).unwrap().get(&Some(i("closed"))).unwrap().clone())
                        .build()
                )
            )
        )])
    }
    // TODO! Implement this!
    #[allow(dead_code, unused_variables)]
    pub fn dialogue_map(
        response_map: &ResponseMap,
        item_map: &ItemMap,
        room_map: &RoomMap,
    ) -> DialogueMap {
        todo!()
    }
}
