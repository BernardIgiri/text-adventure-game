use super::{
    ActionEntity, ActionId, CharacterEntity, CharacterId, DialogueId, DialogueVariantEntity,
    ResponseEntity, ResponseId, RoomId, RoomVariantEntity,
};

pub trait Lookup {
    fn lookup_character(&self, id: CharacterId) -> &CharacterEntity;
    fn lookup_action(&self, id: ActionId) -> &ActionEntity;
    fn lookup_room(&self, id: RoomId) -> &RoomVariantEntity;
    fn lookup_dialogue(&self, id: DialogueId) -> &DialogueVariantEntity;
    fn lookup_response(&self, id: ResponseId) -> &ResponseEntity;
    fn filter_responses(&self, unfiltered: &[ResponseId]) -> Vec<ResponseId>;
}

pub trait Update {
    fn enter_room(&mut self, id: RoomId);
    fn do_action(&mut self, id: ActionId) -> bool;
}

pub trait Database: Lookup + Update {}
impl<T: Lookup + Update> Database for T {}
