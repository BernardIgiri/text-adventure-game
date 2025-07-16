#![deny(clippy::unwrap_used, clippy::expect_used)]
#![warn(clippy::all, clippy::nursery)]

mod config_parser;
mod core;
mod error;
mod player;
mod ui;

use clap::Parser;
use config_parser::preprocess_to_ini_from_file;
use core::{
    ActionHandle, CharacterHandle, DialogueHandle, GameState, ResponseHandle, Room, ToProxy,
};
use player::Player;
use std::{fs::File, path::PathBuf};
use tracing::{self, info};
use tracing_subscriber::{EnvFilter, fmt::writer::BoxMakeWriter};
use ui::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: PathBuf,
}

fn main() {
    #[allow(clippy::expect_used)]
    let file = File::create("game.log").expect("Could not write to required log file!");
    let writer = BoxMakeWriter::new(file);
    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    if let Err(e) = play() {
        tracing::error!("Error: {:#?}", e);
        eprintln!("Error: {e}");
    }
}
fn play() -> Result<(), error::Application> {
    use Player as P;
    let args = Args::parse();
    info!("Loading data...");
    let ini = preprocess_to_ini_from_file(args.file.as_path())
        .map_err(|e| error::CouldNotLoadFile(e.to_string()))?;
    let mut state = GameState::from_ini(ini)?;
    let mut ui = UI::new(state.theme(), state.language());
    let mut player = Player::Idle;
    info!("Staring game...");
    ui.greet(state.title(), state.greeting());
    while player != P::GameOver {
        info!("State {:#?}", player.clone());
        player = match player {
            P::Idle => idle(&state, &mut ui),
            P::ViewInventory => view_inventory(&state, &mut ui),
            P::StartingChat => starting_chat(&state, &mut ui),
            P::ChatWith(character, dialogue) => chat_with(&state, &mut ui, character, dialogue),
            P::DoActionInChatResponse(action, character, response) => {
                do_action_in_chat_response(&mut state, &mut ui, action, character, response)
            }
            P::SelectingAction => selecting_action(&state, &mut ui),
            P::DoingAction(action) => doing_action(&mut state, &mut ui, action),
            P::Leaving => leaving(&mut state, &mut ui),
            P::GameOver => panic!("GameOver state should be unreachable in update loop!"),
        }
    }
    if state.current_room().is_trap() {
        info!("Rolling credits...");
        ui.roll_credits(state.title(), state.credits());
    }
    info!("Finished.");
    Ok(())
}
fn idle(state: &GameState, ui: &mut UI) -> Player {
    use Player as P;
    let room = state.current_room();
    let characters = room
        .characters()
        .iter()
        .map(|v| v.name().to_string())
        .collect::<Vec<_>>();
    let exits = room
        .exits()
        .map(|e| e.direction().to_string())
        .collect::<Vec<_>>();
    let actions = room.actions().next().is_some();
    let choice = ui.present_room(
        room.name().as_str(),
        room.description(),
        &characters,
        &exits,
        actions,
        state.has_inventory(),
    );
    use RoomChoice as C;
    match choice {
        C::Chat => P::StartingChat,
        C::Interact => P::SelectingAction,
        C::Leave => P::Leaving,
        C::GameOver => P::GameOver,
        C::ViewInventory => P::ViewInventory,
    }
}
fn view_inventory(state: &GameState, ui: &mut UI) -> Player {
    use Player as P;
    ui.present_inventory(&state.inventory());
    P::Idle
}
fn starting_chat(state: &GameState, ui: &mut UI) -> Player {
    use Player as P;
    let room = state.current_room();
    let characters = room.characters();
    let characters_names = characters
        .iter()
        .map(|v| v.name().to_string())
        .collect::<Vec<_>>();
    let choice =
        ui.present_chat_targets(room.name().as_str(), room.description(), &characters_names);
    use StartChatChoice as C;
    match choice {
        C::TalkTo(i) => P::ChatWith((&characters[i]).into(), None),
        C::NoOne => P::Idle,
    }
}
fn chat_with(
    state: &GameState,
    ui: &mut UI,
    character: CharacterHandle,
    dialogue: Option<DialogueHandle>,
) -> Player {
    use Player as P;
    let character = character.to_proxy(state);
    let dialogue = dialogue
        .map(|d| d.to_proxy(state))
        .unwrap_or_else(|| character.start_dialogue());
    let responses = dialogue.responses().collect::<Vec<_>>();
    let response_text = responses
        .iter()
        .map(|v| v.text().to_string())
        .collect::<Vec<_>>();
    let choice = ui.present_chat(character.name().as_str(), dialogue.text(), &response_text);
    use ChatChoice as C;
    match choice {
        C::RespondWith(i) => {
            let response = &responses[i];
            #[allow(clippy::option_if_let_else)]
            if let Some(action) = response.trigger() {
                P::DoActionInChatResponse(
                    action.into(),
                    character.handle_clone(),
                    response.handle_clone(),
                )
            } else {
                response.leads_to().map_or(P::Idle, |d| {
                    P::ChatWith(character.handle_clone(), Some(d.handle_clone()))
                })
            }
        }
        C::Leave => P::Idle,
    }
}
fn do_action_in_chat_response(
    state: &mut GameState,
    ui: &mut UI,
    action: ActionHandle,
    character: CharacterHandle,
    response: ResponseHandle,
) -> Player {
    use Player as P;
    let (action_name, action_description) = {
        let action = action.into_proxy();
        let action_name = action.name().clone();
        let action_description = action.description().clone();
        action.do_it(state);
        (action_name, action_description)
    };
    ui.present_action(action_name.as_str(), action_description.as_str(), true);
    response
        .to_proxy(state)
        .leads_to()
        .map_or(P::Idle, |d| P::ChatWith(character, Some(d.into())))
}
fn selecting_action(state: &GameState, ui: &mut UI) -> Player {
    use Player as P;
    let room = state.current_room();
    let actions = room.actions().collect::<Vec<_>>();
    let action_names = actions
        .iter()
        .map(|v| v.name().to_string())
        .collect::<Vec<_>>();
    let choice = ui.present_action_select(room.name().as_str(), room.description(), &action_names);
    use InteractionChoice as C;
    match choice {
        C::Do(i) => {
            let action = &actions[i];
            P::DoingAction(action.handle_clone())
        }
        C::Nothing => P::Idle,
    }
}
fn doing_action(state: &mut GameState, ui: &mut UI, action: ActionHandle) -> Player {
    use Player as P;
    let action = action.into_proxy();
    let success = action.do_it(state);
    ui.present_action(
        action.name().as_str(),
        action.description().as_str(),
        success,
    );
    P::Idle
}
fn leaving(state: &mut GameState, ui: &mut UI) -> Player {
    use Player as P;
    let room = state.current_room();
    let exits = room.exits().collect::<Vec<_>>();
    let directions = exits
        .iter()
        .map(|e| e.direction().to_string())
        .collect::<Vec<_>>();
    let choice = ui.present_exit_select(room.name().as_str(), room.description(), &directions);
    use LeaveChoice as C;
    match choice {
        C::GoTo(i) => {
            let room = exits[i].room();
            let room = room.handle_clone();
            Room::enter(&room, state);
            P::Idle
        }
        C::Stay => P::Idle,
    }
}
