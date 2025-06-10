use std::collections::HashMap;

use derive_new::new;

use crate::entity::{
    action::Action,
    character::{Character, Dialogue, Response},
    invariant::{Identifier, Title},
    requirement::Requirement,
    room::{Item, Room},
};

pub type ActionMap = HashMap<Identifier, Action>;
pub type CharacterMap = HashMap<Title, Character>;
pub type DialogMap = HashMap<Identifier, HashMap<Option<Identifier>, Dialogue>>;
pub type ItemMap = HashMap<Identifier, Item>;
pub type RequirementMap = HashMap<Identifier, Requirement>;
pub type ResponseMap = HashMap<Identifier, Response>;
pub type RoomMap = HashMap<Title, HashMap<Option<Identifier>, Room>>;

#[derive(Debug, Default)]
pub struct WorldData {
    pub action: ActionMap,
    pub character: CharacterMap,
    pub dialogue: DialogMap,
    pub item: ItemMap,
    pub requirement: RequirementMap,
    pub response: ResponseMap,
    pub room: RoomMap,
}

#[derive(Debug, new)]
pub struct World(WorldData);
