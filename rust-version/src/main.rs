#![deny(clippy::unwrap_used, clippy::expect_used)]
#![warn(clippy::all, clippy::nursery)]

mod config_parser;
mod entity;
mod error;
mod world;

use clap::Parser;
use ini::Ini;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    file: PathBuf,
}

fn main() -> Result<(), error::Application> {
    let args = Args::parse();
    let file = args.file.to_str().ok_or(error::CouldNotLoadFile)?;
    let ini = Ini::load_from_file(file).map_err(|_| error::CouldNotLoadFile)?;
    let world = config_parser::parse(ini)?;
    dbg!(world);
    Ok(())
}
