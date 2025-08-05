use bon::Builder;
use derive_getters::Getters;

use super::{RoomId, ThemeColor, Title};

#[derive(Debug)]
pub struct GameTitleRaw {
    pub title: String,
    pub greeting: String,
    pub credits: String,
    pub start_room: Title,
}

#[derive(Getters, Builder, Debug, PartialEq, Eq)]
pub struct GameTitle {
    title: String,
    greeting: String,
    credits: String,
    start_room: RoomId,
}

#[derive(Getters, Builder, Debug, PartialEq, Eq)]
pub struct Theme {
    title: ThemeColor,
    heading: ThemeColor,
    background: ThemeColor,
    text: ThemeColor,
    highlight: ThemeColor,
    highlight_text: ThemeColor,
    subdued: ThemeColor,
}

#[derive(Getters, Builder, Debug, PartialEq, Eq)]
pub struct Language {
    characters_found: String,
    exits_found: String,
    talk: String,
    interact: String,
    go_somewhere: String,
    view_inventory: String,
    inventory: String,
    end_game: String,
    choose_exit: String,
    cancel_exit: String,
    choose_chat: String,
    cancel_chat: String,
    choose_response: String,
    cancel_response: String,
    choose_action: String,
    cancel_action: String,
    action_failed: String,
    continue_game: String,
    press_q_to_quit: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            title: ThemeColor::new(200, 150, 150),
            heading: ThemeColor::new(110, 110, 255),
            background: ThemeColor::new(0, 0, 0),
            text: ThemeColor::new(240, 240, 240),
            highlight: ThemeColor::new(40, 40, 40),
            highlight_text: ThemeColor::new(255, 255, 80),
            subdued: ThemeColor::new(127, 127, 127),
        }
    }
}
impl Default for Language {
    fn default() -> Self {
        Self {
            characters_found: "There are people here:".into(),
            exits_found: "Your exits are:".into(),
            talk: "Talk".into(),
            interact: "Interact".into(),
            go_somewhere: "Go somewhere else".into(),
            view_inventory: "View inventory".into(),
            inventory: "Inventory".into(),
            end_game: "End game".into(),
            choose_exit: "Where will you go?".into(),
            cancel_exit: "Stay".into(),
            choose_chat: "Who will you talk to?".into(),
            cancel_chat: "No one".into(),
            choose_response: "You respond with:".into(),
            cancel_response: "Nothing".into(),
            choose_action: "What will you do?".into(),
            cancel_action: "Nothing".into(),
            action_failed: "Nothing happened...".into(),
            continue_game: "Continue...".into(),
            press_q_to_quit: "Press 'q' at any time to quit!".into(),
        }
    }
}
