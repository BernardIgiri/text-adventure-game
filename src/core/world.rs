use std::rc::Rc;

use bon::Builder;

use super::{
    ActionEntity, ActionId, CharacterEntity, CharacterId, DialogueEntity, DialogueId, GameTitle,
    Item, ItemId, Language, ResponseEntity, ResponseId, RoomEntity, RoomId, RoomVariantEntity,
    RoomVariantId, Theme,
};

#[derive(Debug, Builder)]
pub struct World {
    title: GameTitle,
    #[builder(setters(vis = "", name = "theme_rc"))]
    theme: Rc<Theme>,
    #[builder(setters(vis = "", name = "language_rc"))]
    language: Rc<Language>,
    items: Vec<Item>,
    actions: Vec<ActionEntity>,
    rooms: Vec<RoomEntity>,
    dialogues: Vec<DialogueEntity>,
    characters: Vec<CharacterEntity>,
    responses: Vec<ResponseEntity>,
}

use world_builder::{IsUnset, SetLanguage, SetTheme, State};

impl<S: State> WorldBuilder<S> {
    pub fn theme(self, theme: Theme) -> WorldBuilder<SetTheme<S>>
    where
        S::Theme: IsUnset,
    {
        self.theme_rc(Rc::new(theme))
    }
    pub fn language(self, language: Language) -> WorldBuilder<SetLanguage<S>>
    where
        S::Language: IsUnset,
    {
        self.language_rc(Rc::new(language))
    }
}

impl World {
    pub const fn title(&self) -> &GameTitle {
        &self.title
    }
    pub fn theme(&self) -> Rc<Theme> {
        self.theme.clone()
    }
    pub fn language(&self) -> Rc<Language> {
        self.language.clone()
    }
    pub fn item(&self, id: ItemId) -> &Item {
        &self.items[usize::from(id)]
    }
    pub fn action(&self, id: ActionId) -> &ActionEntity {
        &self.actions[usize::from(id)]
    }
    pub fn room(&self, id: RoomId, variant_id: Option<RoomVariantId>) -> &RoomVariantEntity {
        &self.rooms[usize::from(id)][variant_id.map_or(0usize, |id| id.into())]
    }
    pub fn dialogue(&self, id: DialogueId) -> &DialogueEntity {
        &self.dialogues[usize::from(id)]
    }
    pub fn character(&self, id: CharacterId) -> &CharacterEntity {
        &self.characters[usize::from(id)]
    }
    pub fn response(&self, id: ResponseId) -> &ResponseEntity {
        &self.responses[usize::from(id)]
    }
}
