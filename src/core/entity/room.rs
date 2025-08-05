use derive_getters::Getters;
use derive_new::new;
use indexmap::IndexMap;

use crate::{define_id, define_id_and_proxy};

use super::{
    Action, ActionId, Character, CharacterId, Database, IntoProxy,
    database::Lookup,
    invariant::{Identifier, Title},
};

define_id!(ItemId);
define_id_and_proxy!(RoomId, Room);
define_id!(RoomVariantId);

#[derive(Getters, new, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item {
    pub name: Identifier,
    pub description: String,
}

#[derive(Debug)]
pub struct RoomRaw {
    pub name: Title,
    pub variant: Option<Identifier>,
    pub description: String,
    pub characters: Vec<Title>,
    pub exits: IndexMap<Identifier, Title>,
    pub actions: Vec<Identifier>,
}

pub type RoomEntity = Vec<RoomVariantEntity>;

#[derive(Debug, PartialEq, Eq)]
pub struct RoomVariantEntity {
    pub name: String,
    pub description: String,
    pub characters: Vec<CharacterId>,
    pub exits: IndexMap<Identifier, RoomId>,
    pub actions: Vec<ActionId>,
}

#[derive(Getters)]
pub struct Exit<'a, T: Lookup> {
    direction: Identifier,
    room: Room<'a, T>,
}

impl<'a, DB: Lookup> Room<'a, DB> {
    fn room(&self) -> &RoomVariantEntity {
        self.db.lookup_room(self.id)
    }
    pub fn is_trap(&self) -> bool {
        self.room().exits.is_empty()
    }
    pub fn name(&self) -> &str {
        self.room().name.as_str()
    }
    pub fn description(&self) -> &str {
        self.room().description.as_str()
    }
    pub fn actions(&self) -> impl Iterator<Item = Action<'a, DB>> {
        self.room().actions.iter().map(|id| id.into_proxy(self.db))
    }
    pub fn characters(&self) -> impl Iterator<Item = Character<'_, DB>> {
        self.room()
            .characters
            .iter()
            .map(|id| id.into_proxy(self.db))
    }
    pub fn exits(&self) -> impl Iterator<Item = Exit<'_, DB>> {
        self.room().exits.iter().map(|(direction, id)| Exit {
            direction: direction.clone(),
            room: id.into_proxy(self.db),
        })
    }
    pub fn enter(id: RoomId, db: &mut impl Database) {
        db.enter_room(id);
    }
}
