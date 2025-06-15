mod action;
mod character;
mod dialogue;
mod item;
mod iter;
mod requirement;
mod response;
mod room;
mod title;
mod types;

#[cfg(test)]
mod test_utils;

use action::parse_actions;
use character::parse_characters;
use dialogue::parse_dialogues;
use ini::Ini;
use item::parse_items;
use response::parse_responses;
use room::parse_rooms;
use title::parse_title;

use crate::{error, world::World};

pub fn parse(ini: Ini) -> Result<World, error::Application> {
    let title = parse_title(&ini)?;
    let characters = parse_characters(ini.iter())?;
    let items = parse_items(ini.iter())?;
    dbg!(3);
    let rooms = parse_rooms(ini.iter(), &characters, &items)?;
    dbg!(4);
    let actions = parse_actions(ini.iter(), &rooms, &items)?;
    let responses = parse_responses(ini.iter(), &actions, &items, &rooms)?;
    let dialogues = parse_dialogues(ini.iter(), &responses, &items, &rooms)?;
    World::try_new(
        title,
        actions,
        rooms,
        dialogues,
        characters.values().cloned().collect(),
        responses.values().cloned().collect(),
    )
}
