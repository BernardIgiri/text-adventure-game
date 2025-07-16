use crate::core::{ActionHandle, CharacterHandle, DialogueHandle, ResponseHandle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Player {
    ChatWith(CharacterHandle, Option<DialogueHandle>),
    DoActionInChatResponse(ActionHandle, CharacterHandle, ResponseHandle),
    DoingAction(ActionHandle),
    GameOver,
    Idle,
    Leaving,
    SelectingAction,
    StartingChat,
    ViewInventory,
}
