use super::room::{Item, Room};

// TODO: Implement this
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Requirement {
    HasItem(Item),
    RoomVariant(Room),
}
