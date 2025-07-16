use std::rc::Rc;

use bon::Builder;
use derive_getters::Getters;
use derive_new::new;

use super::{
    Database, Title,
    invariant::Identifier,
    room::{Item, RoomEntity},
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
    room: Rc<RoomEntity>,
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
    actions: Vec<Identifier>,
});

#[derive(Debug, PartialEq, Eq)]
pub enum ActionEntity {
    ChangeRoom(ChangeRoom),
    GiveItem(GiveItem),
    ReplaceItem(ReplaceItem),
    TakeItem(TakeItem),
    Teleport(Teleport),
    Sequence(Sequence),
}

#[derive(new, Clone, Debug, PartialEq, Eq)]
pub struct ActionHandle(Rc<ActionEntity>);
impl ActionHandle {
    pub fn into_proxy(self) -> Action {
        self.into()
    }
}
impl From<Rc<ActionEntity>> for ActionHandle {
    fn from(value: Rc<ActionEntity>) -> Self {
        Self::new(value)
    }
}
impl From<&Rc<ActionEntity>> for ActionHandle {
    fn from(value: &Rc<ActionEntity>) -> Self {
        Self::new(value.clone())
    }
}

#[derive(new)]
pub struct Action {
    handle: ActionHandle,
}
impl From<Action> for ActionHandle {
    fn from(proxy: Action) -> Self {
        proxy.handle
    }
}
impl From<ActionHandle> for Action {
    fn from(value: ActionHandle) -> Self {
        Self::new(value)
    }
}
impl From<&Rc<ActionEntity>> for Action {
    fn from(value: &Rc<ActionEntity>) -> Self {
        Self::new(value.into())
    }
}
impl From<Rc<ActionEntity>> for Action {
    fn from(value: Rc<ActionEntity>) -> Self {
        Self::new(value.into())
    }
}
impl Action {
    pub fn name(&self) -> &Identifier {
        use ActionEntity as A;
        match &*self.handle.0 {
            A::ChangeRoom(change_room) => &change_room.name,
            A::GiveItem(give_item) => &give_item.name,
            A::ReplaceItem(replace_item) => &replace_item.name,
            A::TakeItem(take_item) => &take_item.name,
            A::Teleport(teleport) => &teleport.name,
            A::Sequence(chain) => &chain.name,
        }
    }
    pub fn description(&self) -> &String {
        use ActionEntity as A;
        match &*self.handle.0 {
            A::ChangeRoom(change_room) => &change_room.description,
            A::GiveItem(give_item) => &give_item.description,
            A::ReplaceItem(replace_item) => &replace_item.description,
            A::TakeItem(take_item) => &take_item.description,
            A::Teleport(teleport) => &teleport.description,
            A::Sequence(chain) => &chain.description,
        }
    }
    pub fn do_it<'b, T: Database>(&'b self, db: &'b mut T) -> bool {
        db.do_action(&self.handle.0)
    }
    #[allow(dead_code)]
    pub fn into_handle(self) -> ActionHandle {
        self.into()
    }
    pub fn handle_clone(&self) -> ActionHandle {
        self.handle.clone()
    }
}
