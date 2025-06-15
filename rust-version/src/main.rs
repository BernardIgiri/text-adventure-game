#![deny(clippy::unwrap_used, clippy::expect_used)]
#![warn(clippy::all, clippy::nursery)]

mod config_parser;
mod error;
mod player;
mod world;

use clap::Parser;
use ini::Ini;
use std::path::PathBuf;
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

fn play() -> Result<(), error::Application> {
    let args = Args::parse();
    let file = args.file.to_str().ok_or(error::CouldNotLoadFile)?;
    let ini = Ini::load_from_file(file).map_err(|_| error::CouldNotLoadFile)?;
    let world = config_parser::parse(ini)?;
    let state = GameState::default();
    println!("{}", world.title());
    dbg!(world);
    Ok(())
}
