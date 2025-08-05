use crate::core::{ActionId, CharacterId, DialogueId, ResponseId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Player {
    ChatWith(CharacterId, Option<DialogueId>),
    DoActionInChatResponse(ActionId, CharacterId, ResponseId),
    DoingAction(ActionId),
    GameOver,
    Idle,
    Leaving,
    SelectingAction,
    StartingChat,
    ViewInventory,
}
