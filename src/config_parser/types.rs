use std::rc::Rc;

pub use crate::core::{ActionMap, CharacterMap, DialogueMap, ItemMap, ResponseMap, RoomMap};
use crate::core::{Identifier, Room, Title};

pub trait RoomVariant {
    fn get_room(&self, room: &Title, variant: &Option<Identifier>) -> Option<Rc<Room>>;
}

impl RoomVariant for RoomMap {
    fn get_room(&self, room: &Title, variant: &Option<Identifier>) -> Option<Rc<Room>> {
        self.get(room).and_then(|r| r.get(variant).cloned())
    }
}
