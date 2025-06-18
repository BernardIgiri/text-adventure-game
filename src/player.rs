use std::rc::Rc;

use crate::core::{Action, Character, Dialogue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Player {
    Idle,
    StartingChat,
    ChatWith(Rc<Character>, Option<Rc<Dialogue>>),
    SelectingAction,
    DoingAction(Rc<Action>, bool),
    Leaving,
    GameOver,
}
