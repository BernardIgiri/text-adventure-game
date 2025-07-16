use std::rc::Rc;

use super::{ActionEntity, DialogueEntity, Identifier, ResponseEntity, RoomEntity, Title};

pub trait Database {
    fn lookup_action(&self, name: &Identifier) -> Rc<ActionEntity>;
    fn lookup_room(&self, name: &Title) -> Rc<RoomEntity>;
    fn lookup_dialogue(&self, id: &Identifier) -> Rc<DialogueEntity>;
    fn lookup_responses(&self, unfiltered: &[Rc<ResponseEntity>]) -> Vec<Rc<ResponseEntity>>;
    fn enter_room(&mut self, room: &RoomEntity);
    fn do_action(&mut self, action: &ActionEntity) -> bool;
}
