use std::rc::Rc;

use bon::Builder;
use derive_getters::Getters;

use super::{
    invariant::Identifier,
    room::{Item, Room},
};

macro_rules! define_action {
    (
        $name:ident {
            $($field:ident : $type:ty),* $(,)?
        }
    ) => {
        #[allow(dead_code)]
        #[derive(Getters, Builder, Debug, PartialEq, Eq)]
        pub struct $name {
            name: Identifier,
            description: String,
            $(
                $field: $type,
            )*
        }
    };
}

define_action!(ChangeRoom {
    required: Option<Rc<Item>>,
    room: Rc<Room>,
});

define_action!(ReplaceItem {
    original: Rc<Item>,
    replacement: Rc<Item>,
});

define_action!(GiveItem {
    items: Vec<Rc<Item>>,
});

define_action!(TakeItem {
    required: Option<Rc<Item>>,
    items: Vec<Rc<Item>>,
});

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    ChangeRoom(ChangeRoom),
    GiveItem(GiveItem),
    ReplaceItem(ReplaceItem),
    TakeItem(TakeItem),
}

impl Action {
    pub fn name(&self) -> &Identifier {
        match &self {
            Self::ChangeRoom(change_room) => change_room.name(),
            Self::GiveItem(give_item) => give_item.name(),
            Self::ReplaceItem(replace_item) => replace_item.name(),
            Self::TakeItem(take_item) => take_item.name(),
        }
    }
    pub fn description(&self) -> &String {
        match &self {
            Self::ChangeRoom(change_room) => change_room.description(),
            Self::GiveItem(give_item) => give_item.description(),
            Self::ReplaceItem(replace_item) => replace_item.description(),
            Self::TakeItem(take_item) => take_item.description(),
        }
    }
}
