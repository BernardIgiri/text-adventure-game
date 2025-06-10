mod config_parser;
mod entity;
mod error;

use clap::Parser;
use ini::Ini;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    file: PathBuf,
}

fn main() -> Result<(), error::Game> {
    let args = Args::parse();
    let file = args.file.to_str().ok_or(error::Game::CouldNotLoadFile)?;
    let ini = Ini::load_from_file(file).unwrap();
    let world = config_parser::parse(ini)?;
    println!("{:#?}", world);
    Ok(())
}
