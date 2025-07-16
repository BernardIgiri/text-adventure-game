mod entity;
mod state;
mod world;

use std::{collections::HashMap, rc::Rc};

#[allow(unused_imports)]
pub use entity::{
    Action, ActionEntity, ActionHandle, ChangeRoom, Character, CharacterEntity, CharacterHandle,
    Database, Dialogue, DialogueEntity, DialogueHandle, GameTitle, GiveItem, Identifier,
    IllegalConversion, Item, Language, ReplaceItem, Requirement, Response, ResponseEntity,
    ResponseHandle, Room, RoomEntity, RoomHandle, Sequence, TakeItem, Teleport, Theme, ThemeColor,
    Title, ToProxy,
};
pub use state::GameState;
pub use world::World;

pub type ActionMap = HashMap<Identifier, Rc<ActionEntity>>;
pub type DialogueMap = HashMap<Identifier, HashMap<Option<Identifier>, Rc<DialogueEntity>>>;
pub type RoomMap = HashMap<Title, HashMap<Option<Identifier>, Rc<RoomEntity>>>;
pub type CharacterMap = HashMap<Title, Rc<CharacterEntity>>;
pub type ResponseMap = HashMap<Identifier, Rc<ResponseEntity>>;
pub type ItemMap = HashMap<Identifier, Rc<Item>>;
