#![deny(clippy::unwrap_used, clippy::expect_used)]
#![warn(clippy::all, clippy::nursery)]

mod config_parser;
mod core;
mod error;
mod player;
mod ui;

use clap::Parser;
use config_parser::preprocess_to_ini_from_file;
use core::GameState;
use player::Player;
use std::{fs::File, path::PathBuf};
use tracing::info;
use tracing_subscriber::{fmt::writer::BoxMakeWriter, EnvFilter};
use ui::*;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    file: PathBuf,
}

fn main() {
    #[allow(clippy::expect_used)]
    let file = File::create("game.log").expect("log file");
    let writer = BoxMakeWriter::new(file);
    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    if let Err(e) = play() {
        eprintln!("Error: {}", e);
    }
}

// Required for UI
#[allow(clippy::expect_used)]
fn play() -> Result<(), error::Application> {
    use Player as P;
    let args = Args::parse();
    info!("Loading data");
    let ini = preprocess_to_ini_from_file(args.file.as_path())
        .map_err(|e| error::CouldNotLoadFile(e.to_string()))?;
    let mut state = GameState::from_ini(ini)?;
    let mut player = Player::Idle;
    let mut ui = UI::new();
    info!("Staring game");
    ui.greet(state.title(), state.greeting());
    while player != P::GameOver {
        info!("State {:#?}", player.clone());
        player = match player {
            P::Idle => {
                let room = state.current_room();
                let characters = room
                    .characters()
                    .iter()
                    .map(|v| v.name().to_string())
                    .collect::<Vec<_>>();
                let exits = room
                    .exit_directions()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>();
                let actions = !state.room_actions(&room).is_empty();
                let choice = ui.present_room(
                    room.name().as_str(),
                    room.description(),
                    &characters,
                    &exits,
                    actions,
                );
                use RoomChoice as C;
                match choice {
                    C::Chat => P::StartingChat,
                    C::Interact => P::SelectingAction,
                    C::Leave => P::Leaving,
                    C::GameOver => P::GameOver,
                }
            }
            P::StartingChat => {
                let room = state.current_room();
                let characters = room.characters();
                let characters_names = characters
                    .iter()
                    .map(|v| v.name().to_string())
                    .collect::<Vec<_>>();
                let choice = ui.present_chat_targets(
                    room.name().as_str(),
                    room.description(),
                    &characters_names,
                );
                use StartChatChoice as C;
                match choice {
                    C::TalkTo(i) => P::ChatWith(
                        characters
                            .get(i)
                            .expect("Only valid character choices should be in the menu!")
                            .clone(),
                        None,
                    ),
                    C::NoOne => P::Idle,
                }
            }
            P::ChatWith(character, dialogue) => {
                let dialogue =
                    dialogue.unwrap_or_else(|| state.character_start_dialogue(&character));
                let responses = state.dialogue_responses(&dialogue);
                let response_text = responses
                    .iter()
                    .map(|v| v.text().to_string())
                    .collect::<Vec<_>>();
                let choice =
                    ui.present_chat(character.name().as_str(), dialogue.text(), &response_text);
                use ChatChoice as C;
                match choice {
                    C::RespondWith(i) => {
                        let response = responses
                            .get(i)
                            .expect("Only valid response choices should be in the menu!")
                            .clone();
                        if let Some(action) = state.trigger_response(&response) {
                            P::DoActionInChatResponse(action, character, response)
                        } else {
                            state
                                .response_reply(&response)
                                .map_or(P::Idle, |d| P::ChatWith(character, Some(d)))
                        }
                    }
                    C::Leave => P::Idle,
                }
            }
            P::DoActionInChatResponse(action, character, response) => {
                ui.present_action(action.name().as_str(), action.description().as_str(), true);
                state
                    .response_reply(&response)
                    .map_or(P::Idle, |d| P::ChatWith(character, Some(d)))
            }
            P::SelectingAction => {
                let room = state.current_room();
                let actions = state.room_actions(&room);
                let action_names = actions
                    .iter()
                    .map(|v| v.name().to_string())
                    .collect::<Vec<_>>();
                let choice = ui.present_action_select(
                    room.name().as_str(),
                    room.description(),
                    &action_names,
                );
                use InteractionChoice as C;
                match choice {
                    C::Do(i) => {
                        let action = actions
                            .get(i)
                            .expect("Only valid actions choices should be in the menu!")
                            .clone();
                        let success = state.do_action(&action);
                        P::DoingAction(action, success)
                    }
                    C::Nothing => P::Idle,
                }
            }
            P::DoingAction(a, success) => {
                ui.present_action(a.name().as_str(), a.description().as_str(), success);
                P::Idle
            }
            P::Leaving => {
                let room = state.current_room();
                let exits = state.room_exits(&room);
                let directions = room
                    .exit_directions()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>();
                let choice =
                    ui.present_exit_select(room.name().as_str(), room.description(), &directions);
                use LeaveChoice as C;
                match choice {
                    C::GoTo(i) => {
                        state.enter_room(
                            exits
                                .get(i)
                                .expect("Only valid exit choices should be in the menu!")
                                .clone(),
                        );
                        P::Idle
                    }
                    C::Stay => P::Idle,
                }
            }
            P::GameOver => panic!("GameOver state should be unreachable in update loop!"),
        }
    }
    if state.current_room().is_trap() {
        ui.roll_credits(state.title(), state.credits());
    }
    Ok(())
}
