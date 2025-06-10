use super::room::{Item, Room};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Requirement {
    HasItem(Item),
    RoomVariant(Room),
}
