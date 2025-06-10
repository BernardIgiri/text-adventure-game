use std::collections::HashMap;

use bon::Builder;
use derive_getters::Getters;
use derive_new::new;

use super::{
    action::Action,
    invariant::{Identifier, Title},
};

#[derive(Getters, Builder, Debug, Clone, PartialEq, Eq)]
pub struct Room {
    name: Title,
    variant: Option<Identifier>,
    description: String,
    exits: HashMap<Identifier, Box<Room>>,
    items: Vec<Item>,
    actions: Vec<Action>,
}

#[derive(Getters, new, Debug, Clone, PartialEq, Eq)]
pub struct Item {
    name: Identifier,
    description: String,
}
