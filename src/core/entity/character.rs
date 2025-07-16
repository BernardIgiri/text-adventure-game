use derive_getters::Getters;
use std::rc::Rc;

use bon::Builder;
use derive_new::new;

use crate::proxied_entity;

use super::{
    Action, Database, Item, RoomEntity, ToProxy,
    action::ActionEntity,
    invariant::{Identifier, Title},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Requirement {
    HasItem(Rc<Item>),
    RoomVariant(Rc<RoomEntity>),
    DoesNotHave(Rc<Item>),
}

proxied_entity!(Character, CharacterEntity, CharacterHandle {
    name: Title,
}, {
    start_dialogue: Identifier,
});

proxied_entity!(Dialogue, DialogueEntity, DialogueHandle {
    text: String,
}, {
    responses: Vec<Rc<ResponseEntity>>,
    requires: Vec<Requirement>,
});

proxied_entity!(Response, ResponseEntity, ResponseHandle {
    text: String,
}, {
    leads_to: Option<Identifier>,
    triggers: Option<Rc<ActionEntity>>,
    requires: Vec<Requirement>,
});

impl<'a, T: Database> Character<'a, T> {
    pub fn start_dialogue(&self) -> Dialogue<'_, T> {
        self.db
            .lookup_dialogue(&self.handle.0.start_dialogue)
            .to_proxy(self.db)
    }
}
impl<'a, T: Database> Dialogue<'a, T> {
    pub fn responses(&self) -> impl Iterator<Item = Response<'_, T>> {
        self.db
            .lookup_responses(&self.handle.0.responses)
            .into_iter()
            .map(|r| r.to_proxy(self.db))
    }
}
impl<'a, T: Database> Response<'a, T> {
    pub fn leads_to(&self) -> Option<Dialogue<'_, T>> {
        self.handle
            .0
            .leads_to
            .clone()
            .map(|id| self.db.lookup_dialogue(&id).to_proxy(self.db))
    }
    pub fn trigger(&self) -> Option<Action> {
        self.handle.0.triggers.clone().map(From::from)
    }
}
