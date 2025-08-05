use crate::{define_id, define_id_and_proxy};

use super::{
    IntoProxy, ItemId, RoomId, RoomVariantId, Title,
    database::{Lookup, Update},
    invariant::Identifier,
};

#[derive(Debug)]
pub struct ChangeRoomRaw {
    pub name: Identifier,
    pub description: String,
    pub required: Option<Identifier>,
    pub room: Title,
    pub variant: Option<Identifier>,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ChangeRoom {
    pub name: String,
    pub description: String,
    pub required: Option<ItemId>,
    pub room: RoomId,
    pub variant: Option<RoomVariantId>,
}

#[derive(Debug)]
pub struct ReplaceItemRaw {
    pub name: Identifier,
    pub description: String,
    pub original: Identifier,
    pub replacement: Identifier,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ReplaceItem {
    pub name: String,
    pub description: String,
    pub original: ItemId,
    pub replacement: ItemId,
}

#[derive(Debug)]
pub struct GiveItemRaw {
    pub name: Identifier,
    pub description: String,
    pub required: Option<Identifier>,
    pub items: Vec<Identifier>,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GiveItem {
    pub name: String,
    pub description: String,
    pub required: Option<ItemId>,
    pub items: Vec<ItemId>,
}

#[derive(Debug)]
pub struct TakeItemRaw {
    pub name: Identifier,
    pub description: String,
    pub items: Vec<Identifier>,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TakeItem {
    pub name: String,
    pub description: String,
    pub items: Vec<ItemId>,
}

#[derive(Debug)]
pub struct TeleportRaw {
    pub name: Identifier,
    pub description: String,
    pub required: Option<Identifier>,
    pub room: Title,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Teleport {
    pub name: String,
    pub description: String,
    pub required: Option<ItemId>,
    pub room: RoomId,
}

#[derive(Debug)]
pub struct SequenceRaw {
    pub name: Identifier,
    pub description: String,
    pub required: Option<Identifier>,
    pub actions: Vec<Identifier>,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Sequence {
    pub name: String,
    pub description: String,
    pub required: Option<ItemId>,
    pub actions: Vec<ActionId>,
}

define_id_and_proxy!(ActionId, Action);

#[derive(Debug)]
pub enum ActionRaw {
    ChangeRoom(ChangeRoomRaw),
    GiveItem(GiveItemRaw),
    ReplaceItem(ReplaceItemRaw),
    TakeItem(TakeItemRaw),
    Teleport(TeleportRaw),
    Sequence(SequenceRaw),
}
impl ActionRaw {
    pub const fn name(&self) -> &Identifier {
        match self {
            Self::ChangeRoom(change_room) => &change_room.name,
            Self::GiveItem(give_item) => &give_item.name,
            Self::ReplaceItem(replace_item) => &replace_item.name,
            Self::TakeItem(take_item) => &take_item.name,
            Self::Teleport(teleport) => &teleport.name,
            Self::Sequence(chain) => &chain.name,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionEntity {
    ChangeRoom(ChangeRoom),
    GiveItem(GiveItem),
    ReplaceItem(ReplaceItem),
    TakeItem(TakeItem),
    Teleport(Teleport),
    Sequence(Sequence),
}
impl<'a, DB: Lookup> Action<'a, DB> {
    fn action(&self) -> &ActionEntity {
        self.db.lookup_action(self.id)
    }
    pub fn name(&self) -> String {
        use ActionEntity as A;
        match self.action() {
            A::ChangeRoom(change_room) => change_room.name.to_string(),
            A::GiveItem(give_item) => give_item.name.to_string(),
            A::ReplaceItem(replace_item) => replace_item.name.to_string(),
            A::TakeItem(take_item) => take_item.name.to_string(),
            A::Teleport(teleport) => teleport.name.to_string(),
            A::Sequence(chain) => chain.name.to_string(),
        }
    }
    pub fn description(&self) -> String {
        use ActionEntity as A;
        match self.action() {
            A::ChangeRoom(change_room) => change_room.description.to_string(),
            A::GiveItem(give_item) => give_item.description.to_string(),
            A::ReplaceItem(replace_item) => replace_item.description.to_string(),
            A::TakeItem(take_item) => take_item.description.to_string(),
            A::Teleport(teleport) => teleport.description.to_string(),
            A::Sequence(chain) => chain.description.to_string(),
        }
    }
    pub fn do_it(id: ActionId, db: &mut impl Update) -> bool {
        db.do_action(id)
    }
}
