use std::rc::Rc;

use crate::core::{Action, Character, Dialogue, Response};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Player {
    ChatWith(Rc<Character>, Option<Rc<Dialogue>>),
    DoActionInChatResponse(Rc<Action>, Rc<Character>, Rc<Response>),
    DoingAction(Rc<Action>, bool),
    GameOver,
    Idle,
    Leaving,
    SelectingAction,
    StartingChat,
    ViewInventory,
}
