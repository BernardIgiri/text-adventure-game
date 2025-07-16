use std::rc::Rc;

pub use crate::core::{ActionMap, CharacterMap, DialogueMap, ItemMap, ResponseMap, RoomMap};
use crate::core::{Identifier, RoomEntity, Title};

pub trait RoomVariant {
    fn get_room(&self, room: &Title, variant: &Option<Identifier>) -> Option<Rc<RoomEntity>>;
}

impl RoomVariant for RoomMap {
    fn get_room(&self, room: &Title, variant: &Option<Identifier>) -> Option<Rc<RoomEntity>> {
        self.get(room).and_then(|r| r.get(variant).cloned())
    }
}
