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
        // TODO: Implment this!
        #[allow(dead_code)]
        #[derive(Getters, Builder, Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ChangeRoom(ChangeRoom),
    GiveItem(GiveItem),
    ReplaceItem(ReplaceItem),
    TakeItem(TakeItem),
}
