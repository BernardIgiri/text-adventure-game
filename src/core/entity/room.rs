use std::{collections::HashMap, rc::Rc};

use bon::Builder;
use derive_getters::Getters;
use derive_new::new;

use super::{
    invariant::{Identifier, Title},
    Character,
};

#[derive(Getters, Builder, Debug, PartialEq, Eq)]
pub struct Room {
    name: Title,
    variant: Option<Identifier>,
    description: String,
    characters: Vec<Rc<Character>>,
    #[getter(skip)]
    exits: HashMap<Identifier, Title>,
    #[getter(skip)]
    actions: Vec<Identifier>,
}

impl Room {
    pub fn exit_directions(&self) -> impl Iterator<Item = &Identifier> {
        self.exits.keys()
    }
    pub fn is_trap(&self) -> bool {
        self.exits.is_empty()
    }
}

#[derive(new)]
pub struct RoomRefs<'a>(&'a Room);

impl<'a> RoomRefs<'a> {
    pub const fn exits(&self) -> &HashMap<Identifier, Title> {
        &self.0.exits
    }
    pub const fn actions(&self) -> &Vec<Identifier> {
        &self.0.actions
    }
}

#[derive(Getters, new, Hash, Debug, PartialEq, Eq)]
pub struct Item {
    name: Identifier,
    description: String,
}
