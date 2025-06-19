use std::rc::Rc;

use crate::core::{Action, Character, Dialogue, Response};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Player {
    Idle,
    StartingChat,
    ChatWith(Rc<Character>, Option<Rc<Dialogue>>),
    DoActionInChatResponse(Rc<Action>, Rc<Character>, Rc<Response>),
    SelectingAction,
    DoingAction(Rc<Action>, bool),
    Leaving,
    GameOver,
}
