use derive_getters::Getters;
use derive_new::new;

use super::{
    action::Action,
    invariant::{Identifier, Title},
    requirement::Requirement,
    room::Room,
};

#[derive(Getters, new, Debug, Clone, PartialEq, Eq)]
pub struct Character {
    name: Title,
    origin: Room,
    start_dialog: Dialogue,
}

#[derive(Getters, new, Debug, Clone, PartialEq, Eq)]
pub struct Dialogue {
    variant: Option<Identifier>,
    text: String,
    responses: Vec<Response>,
    requires: Vec<Requirement>,
    action: Option<Action>,
}

#[derive(Getters, new, Debug, Clone, PartialEq, Eq)]
pub struct Response {
    text: String,
    leads_to: Option<Dialogue>,
    requires: Vec<Requirement>,
}
