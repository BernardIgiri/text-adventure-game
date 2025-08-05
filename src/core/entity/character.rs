use crate::{define_id, define_id_and_proxy};

use super::{
    Action, ActionId, IntoProxy, ItemId, RoomId, RoomVariantId,
    database::Lookup,
    invariant::{Identifier, Title},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Requirement {
    HasItem(ItemId),
    RoomVariant(RoomId, Option<RoomVariantId>),
    DoesNotHave(ItemId),
}
#[derive(Debug)]
pub enum RequirementRaw {
    HasItem(Identifier),
    RoomVariant(Title, Option<Identifier>),
    DoesNotHave(Identifier),
}

define_id_and_proxy!(CharacterId, Character);
define_id_and_proxy!(DialogueId, Dialogue);
define_id!(DialogueVariantId);
define_id_and_proxy!(ResponseId, Response);

#[derive(Debug)]
pub struct CharacterRaw {
    pub name: Title,
    pub start_dialogue: Identifier,
}
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct CharacterEntity {
    pub name: String,
    pub start_dialogue: DialogueId,
}

#[derive(Debug)]
pub struct DialogueRaw {
    pub name: Identifier,
    pub variant: Option<Identifier>,
    pub text: String,
    pub responses: Vec<Identifier>,
    pub requires: Vec<RequirementRaw>,
}
pub type DialogueEntity = Vec<DialogueVariantEntity>;
#[derive(Debug, PartialEq, Eq)]
pub struct DialogueVariantEntity {
    pub text: String,
    pub responses: Vec<ResponseId>,
    pub requires: Vec<Requirement>,
}

#[derive(Debug)]
pub struct ResponseRaw {
    pub name: Identifier,
    pub text: String,
    pub leads_to: Option<Identifier>,
    pub triggers: Option<Identifier>,
    pub requires: Vec<RequirementRaw>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct ResponseEntity {
    pub text: String,
    pub leads_to: Option<DialogueId>,
    pub triggers: Option<ActionId>,
    pub requires: Vec<Requirement>,
}

impl<'a, T: Lookup> Character<'a, T> {
    fn character(&self) -> &CharacterEntity {
        self.db.lookup_character(self.id)
    }
    pub fn name(&self) -> &str {
        self.character().name.as_str()
    }
    pub fn start_dialogue(&self) -> Dialogue<'_, T> {
        self.character().start_dialogue.into_proxy(self.db)
    }
}
impl<'a, T: Lookup> Dialogue<'a, T> {
    fn dialogue(&self) -> &DialogueVariantEntity {
        self.db.lookup_dialogue(self.id)
    }
    pub fn text(&self) -> &str {
        self.dialogue().text.as_str()
    }
    pub fn responses(&self) -> impl Iterator<Item = Response<'_, T>> {
        self.db
            .filter_responses(&self.dialogue().responses)
            .into_iter()
            .map(|r| r.into_proxy(self.db))
    }
}
impl<'a, T: Lookup> Response<'a, T> {
    fn response(&self) -> &ResponseEntity {
        self.db.lookup_response(self.id)
    }
    pub fn text(&self) -> &str {
        self.response().text.as_str()
    }
    pub fn leads_to(&self) -> Option<Dialogue<'_, T>> {
        self.response().leads_to.map(|id| id.into_proxy(self.db))
    }
    pub fn trigger(&self) -> Option<Action<'_, T>> {
        self.response().triggers.map(|id| id.into_proxy(self.db))
    }
}
