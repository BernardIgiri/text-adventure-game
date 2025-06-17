mod entity;
mod game;
mod world;

use std::{collections::HashMap, rc::Rc};

pub use entity::{
    Action, ChangeRoom, Character, Dialogue, GameTitle, GiveItem, Identifier, Item, ReplaceItem,
    Requirement, Response, Room, TakeItem, Title,
};
pub use game::GameState;
pub use world::World;

pub type ActionMap = HashMap<Identifier, Rc<Action>>;
pub type DialogueMap = HashMap<Identifier, HashMap<Option<Identifier>, Rc<Dialogue>>>;
pub type RoomMap = HashMap<Title, HashMap<Option<Identifier>, Rc<Room>>>;
