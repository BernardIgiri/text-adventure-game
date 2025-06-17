use std::{collections::HashMap, rc::Rc};

pub use crate::core::{ActionMap, DialogueMap, RoomMap};
use crate::core::{Character, Identifier, Item, Response, Room, Title};

pub type CharacterMap = HashMap<Title, Rc<Character>>;
pub type ResponseMap = HashMap<Identifier, Rc<Response>>;
pub type ItemMap = HashMap<Identifier, Rc<Item>>;

pub trait RoomVariant {
    fn get_room(&self, room: &Title, variant: &Option<Identifier>) -> Option<Rc<Room>>;
}

impl RoomVariant for RoomMap {
    fn get_room(&self, room: &Title, variant: &Option<Identifier>) -> Option<Rc<Room>> {
        self.get(room).and_then(|r| r.get(variant).cloned())
    }
}
