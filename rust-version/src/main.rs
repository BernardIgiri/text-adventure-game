#![deny(clippy::unwrap_used, clippy::expect_used)]
#![warn(clippy::all, clippy::nursery)]

mod config_parser;
mod error;
mod player;
mod world;

use clap::Parser;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{enable_raw_mode, Clear, ClearType},
};
use ini::Ini;
use player::Player;
use std::{io::stdout, path::PathBuf, time::Duration};
use world::{GameQuery, GameState};

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

fn play() -> Result<(), error::Application> {
    let args = Args::parse();
    let file = args.file.to_str().ok_or(error::CouldNotLoadFile)?;
    let ini = Ini::load_from_file(file).map_err(|_| error::CouldNotLoadFile)?;
    let world = config_parser::parse(ini)?;
    let state = GameState::default();
    let mut player = Player::Idle;
    enable_raw_mode().map_err(e_unknown)?;
    let msg = format!("Welcome to: {}", world.title());
    greeting(&msg)?;
    loop {
        let query = GameQuery::new(&world, &state);
        if !query.is_survivable() {
            player = Player::Dying;
        }
        use Player as P;
        match player {
            P::Idle => {
                use KeyCode as K;
                match get_key()? {
                    Some(K::Char('q')) => {
                        break;
                    }
                    _ => {}
                }
            }
            P::Dying => {
                if get_key()?.is_some() {
                    break;
                }
            }
        }
    }
    dbg!(world);
    Ok(())
}

fn greeting(s: &str) -> Result<(), error::Application> {
    Ok(execute!(
        stdout(),
        Clear(ClearType::All),
        MoveTo(0, 0),
        SetForegroundColor(Color::Blue),
        SetBackgroundColor(Color::Red),
        Print(s),
        ResetColor
    )
    .map_err(e_unknown)?)
}

pub fn get_key() -> Result<Option<KeyCode>, error::Application> {
    if event::poll(Duration::from_millis(500)).map_err(e_unknown)? {
        match event::read().map_err(e_unknown)? {
            Event::Key(key_event) => Ok(Some(key_event.code)),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

fn e_unknown<E>(_: E) -> error::Application {
    error::UknownError
}
