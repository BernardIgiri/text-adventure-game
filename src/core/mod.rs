mod entity;
mod state;
mod world;

use std::{collections::HashMap, rc::Rc};

pub use entity::{
    Action, ChangeRoom, Character, Dialogue, GameTitle, GiveItem, Identifier, IllegalConversion,
    Item, ReplaceItem, Requirement, Response, Room, Sequence, TakeItem, Teleport, Title,
};
pub use state::GameState;
pub use world::World;

pub type ActionMap = HashMap<Identifier, Rc<Action>>;
pub type DialogueMap = HashMap<Identifier, HashMap<Option<Identifier>, Rc<Dialogue>>>;
pub type RoomMap = HashMap<Title, HashMap<Option<Identifier>, Rc<Room>>>;
pub type CharacterMap = HashMap<Title, Rc<Character>>;
pub type ResponseMap = HashMap<Identifier, Rc<Response>>;
pub type ItemMap = HashMap<Identifier, Rc<Item>>;
