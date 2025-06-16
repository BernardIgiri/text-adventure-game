#![deny(clippy::unwrap_used, clippy::expect_used)]
#![warn(clippy::all, clippy::nursery)]

mod config_parser;
mod error;
mod player;
mod ui;
mod world;

use clap::Parser;
use config_parser::preprocess_to_ini_from_file;
use player::Player;
use std::path::PathBuf;
use ui::*;
use world::GameState;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    file: PathBuf,
}

fn main() {
    if let Err(e) = play() {
        eprintln!("Error: {}", e);
    }
}

// Required for UI
#[allow(clippy::expect_used)]
fn play() -> Result<(), error::Application> {
    use Player as P;
    let args = Args::parse();
    let ini = preprocess_to_ini_from_file(args.file.as_path())
        .map_err(|e| error::CouldNotLoadFile(e.to_string()))?;
    let world = config_parser::parse(ini)?;
    let mut state = GameState::new(&world);
    let mut player = Player::Idle;
    let mut ui = UI::new();
    ui.greet(world.title(), world.greeting());
    while player != P::GameOver {
        player = match player {
            P::Idle => {
                let room = state.current_room();
                let items = room
                    .items()
                    .iter()
                    .map(|v| v.name().to_string())
                    .collect::<Vec<_>>();
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
                    &items,
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
                let choice = ui.present_chat_select(
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
            P::ChatWith(c, d) => {
                let dialogue = d.unwrap_or_else(|| state.character_dialogue(&c));
                let responses = state.dialogue_responses(&dialogue);
                let responses_text = responses
                    .iter()
                    .map(|v| v.text().to_string())
                    .collect::<Vec<_>>();
                let choice = ui.present_chat(c.name().as_str(), dialogue.text(), &responses_text);
                use ChatChoice as C;
                match choice {
                    C::RespondWith(i) => {
                        let response = responses
                            .get(i)
                            .expect("Only valid response choices should be in the menu!")
                            .clone();
                        state.trigger_response(&response);
                        state
                            .response_reply(&response)
                            .map_or(P::Idle, |d| P::ChatWith(c, Some(d)))
                    }
                    C::Leave => P::Idle,
                }
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
                        state.do_action(&action);
                        P::DoingAction(action)
                    }
                    C::Nothing => P::Idle,
                }
            }
            P::DoingAction(a) => {
                let room = state.current_room();
                ui.present_action(room.name().as_str(), a.description().as_str());
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
        ui.roll_credits(world.title(), world.credits());
    }
    Ok(())
}
