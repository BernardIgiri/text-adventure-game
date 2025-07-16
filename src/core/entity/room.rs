use std::rc::Rc;

use bon::Builder;
use derive_getters::Getters;
use derive_new::new;
use indexmap::IndexMap;

use crate::proxied_entity;

use super::{
    Action, CharacterEntity, Database, ToProxy,
    invariant::{Identifier, Title},
};

proxied_entity!(Room, RoomEntity, RoomHandle {
    name: Title,
    variant: Option<Identifier>,
    description: String,
    characters: Vec<Rc<CharacterEntity>>,
}, {
    exits: IndexMap<Identifier, Title>,
    actions: Vec<Identifier>,
});

#[derive(Getters, new, Hash, Debug, PartialEq, Eq)]
pub struct Item {
    name: Identifier,
    description: String,
}

#[derive(Getters)]
pub struct Exit<'a, T: Database> {
    direction: Identifier,
    room: Room<'a, T>,
}

impl<'a, T: Database> Room<'a, T> {
    pub fn is_trap(&self) -> bool {
        self.handle.0.exits.is_empty()
    }
    pub fn exits(&self) -> impl Iterator<Item = Exit<'_, T>> {
        self.handle
            .0
            .exits
            .iter()
            .map(|(direction, room_name)| Exit {
                direction: direction.clone(),
                room: self.db.lookup_room(room_name).to_proxy(self.db),
            })
    }
    pub fn enter(handle: &RoomHandle, db: &mut T) {
        db.enter_room(&handle.0);
    }
    pub fn actions(&self) -> impl Iterator<Item = Action> {
        self.handle
            .0
            .actions
            .iter()
            .map(|name| Action::new(self.db.lookup_action(name).into()))
    }
}
