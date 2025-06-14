use std::rc::Rc;

use bon::Builder;
use derive_getters::Getters;
use derive_new::new;

use super::{
    action::Action,
    invariant::{Identifier, Title},
    room::Room,
    Item,
};

#[derive(Getters, new, Debug, Clone, PartialEq, Eq)]
pub struct Character {
    name: Title,
    #[getter(skip)]
    start_dialogue: Identifier,
}

#[derive(new)]
pub struct CharacterRefs<'a>(&'a Character);

impl<'a> CharacterRefs<'a> {
    // TODO: Use this
    #[allow(dead_code)]
    pub const fn get(&self) -> &Character {
        self.0
    }
    pub const fn start_dialogue(&self) -> &Identifier {
        &self.0.start_dialogue
    }
}

#[derive(Getters, Builder, Debug, Clone, PartialEq, Eq)]
pub struct Dialogue {
    text: String,
    responses: Vec<Rc<Response>>,
    requires: Vec<Rc<Requirement>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Requirement {
    HasItem(Rc<Item>),
    RoomVariant(Rc<Room>),
}

#[derive(Getters, Builder, Debug, Clone, PartialEq, Eq)]
pub struct Response {
    text: String,
    #[getter(skip)]
    leads_to: Option<Identifier>,
    triggers: Option<Rc<Action>>,
    requires: Vec<Rc<Requirement>>,
}

#[derive(new)]
pub struct ResponseRefs<'a>(&'a Response);

impl<'a> ResponseRefs<'a> {
    // TODO: Use this
    #[allow(dead_code)]
    pub const fn get(&self) -> &Response {
        self.0
    }
    pub const fn leads_to(&self) -> &Option<Identifier> {
        &self.0.leads_to
    }
}
