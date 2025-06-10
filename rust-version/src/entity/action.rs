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
        // TODO: Implement later
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
    required: Option<Item>,
    room: Room,
});

define_action!(ReplaceItem {
    original: Item,
    replacement: Item,
});

define_action!(GiveItem {
    items: Vec<Item>,
});

define_action!(TakeItem {
    required: Option<Item>,
    items: Vec<Item>,
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ChangeRoom(ChangeRoom),
    GiveItem(GiveItem),
    ReplaceItem(ReplaceItem),
    TakeItem(TakeItem),
}
