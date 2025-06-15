#![cfg(test)]
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
        world::{Character, Item, Response, Room, RoomMap},
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
            (
                "chirp".parse().unwrap(),
                Rc::new(
                    Response::builder()
                        .text("The bird chatters melodiously; chirp, chirp!".into())
                        .requires(vec![])
                        .build(),
                ),
            ),
            (
                "curious".parse().unwrap(),
                Rc::new(
                    Response::builder()
                        .text("That's a shiny ring! May I hold it?".into())
                        .requires(vec![])
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

    // fn wood_shed() -> Room {
    //     Room::builder()
    //         .name("WoodShed".parse().unwrap())
    //         .description("A dusty shed full of tools, lumber, and a giant table saw.".into())
    //         .items()
    // }
}
