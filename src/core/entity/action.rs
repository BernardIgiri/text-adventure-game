use std::rc::Rc;

use bon::Builder;
use derive_getters::Getters;
use derive_new::new;

use super::{
    invariant::Identifier,
    room::{Item, Room},
    Title,
};

macro_rules! define_action {
    (
        $name:ident {
            $(
                $(#[$meta:meta])*
                $field:ident : $type:ty
            ),* $(,)?
        }
    ) => {
        // Both ChangeRoom and TakeItem falsely report that required is unused
        // required is used both in the parser and in GameState::do_action
        #[allow(dead_code)]
        #[derive(Getters, Builder, Debug, PartialEq, Eq)]
        pub struct $name {
            name: Identifier,
            description: String,
            $(
                $(#[$meta])*
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
    required: Option<Rc<Item>>,
    items: Vec<Rc<Item>>,
});

define_action!(TakeItem {
    items: Vec<Rc<Item>>,
});

define_action!(Teleport {
    required: Option<Rc<Item>>,
    room_name: Title,
});

define_action!(Sequence {
    required: Option<Rc<Item>>,
    #[getter(skip)]
    actions: Vec<Identifier>,
});

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    ChangeRoom(ChangeRoom),
    GiveItem(GiveItem),
    ReplaceItem(ReplaceItem),
    TakeItem(TakeItem),
    Teleport(Teleport),
    Sequence(Sequence),
}

impl Action {
    pub fn name(&self) -> &Identifier {
        match &self {
            Self::ChangeRoom(change_room) => change_room.name(),
            Self::GiveItem(give_item) => give_item.name(),
            Self::ReplaceItem(replace_item) => replace_item.name(),
            Self::TakeItem(take_item) => take_item.name(),
            Self::Teleport(teleport) => teleport.name(),
            Self::Sequence(chain) => chain.name(),
        }
    }
    pub fn description(&self) -> &String {
        match &self {
            Self::ChangeRoom(change_room) => change_room.description(),
            Self::GiveItem(give_item) => give_item.description(),
            Self::ReplaceItem(replace_item) => replace_item.description(),
            Self::TakeItem(take_item) => take_item.description(),
            Self::Teleport(teleport) => teleport.description(),
            Self::Sequence(chain) => chain.description(),
        }
    }
}

#[derive(new)]
pub struct SequenceRefs<'a>(&'a Sequence);

impl<'a> SequenceRefs<'a> {
    pub const fn actions(&self) -> &Vec<Identifier> {
        &self.0.actions
    }
}
