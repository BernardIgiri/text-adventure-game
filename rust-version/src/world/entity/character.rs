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

#[derive(Getters, new, Debug, PartialEq, Eq)]
pub struct Character {
    name: Title,
    #[getter(skip)]
    start_dialogue: Identifier,
}

#[derive(new)]
pub struct CharacterRefs<'a>(&'a Character);

impl<'a> CharacterRefs<'a> {
    pub const fn start_dialogue(&self) -> &Identifier {
        &self.0.start_dialogue
    }
}

#[derive(Getters, Builder, Debug, PartialEq, Eq)]
pub struct Dialogue {
    text: String,
    #[getter(skip)]
    responses: Vec<Rc<Response>>,
    requires: Vec<Rc<Requirement>>,
}

#[derive(new)]
pub struct DialogueRefs<'a>(&'a Dialogue);

impl<'a> DialogueRefs<'a> {
    pub const fn responses(&self) -> &Vec<Rc<Response>> {
        &self.0.responses
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Requirement {
    HasItem(Rc<Item>),
    RoomVariant(Rc<Room>),
}

#[derive(Getters, Builder, Debug, PartialEq, Eq)]
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
    pub const fn leads_to(&self) -> &Option<Identifier> {
        &self.0.leads_to
    }
}
