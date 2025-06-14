use std::{collections::HashMap, rc::Rc};

use ini::SectionIter;

use crate::{
    error,
    world::{Character, Identifier, Item, Room, Title},
};

use super::{
    section_iter::{EntitySection, RequireProperty, SectionRecordIter},
    types::{CharacterMap, ItemMap, RoomMap},
};

pub fn parse_rooms<'a>(
    ini_iter: SectionIter<'a>,
    character_map: &CharacterMap,
    item_map: &ItemMap,
) -> Result<RoomMap, error::Application> {
    let mut map = RoomMap::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Room.into()) {
        let record = record?;
        let description = record.properties.require("description", &record)?;
        let exits = record
            .properties
            .get("exits")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .map(|exit| {
                let mut parts = exit.split(":");
                let direction = parts
                    .next()
                    .ok_or_else(|| error::PropertyNotFound {
                        entity: "Room",
                        property: "exit:<direction>",
                        id: record.qualified_name.into(),
                    })?
                    .parse::<Identifier>()?;
                let room = parts
                    .next()
                    .ok_or_else(|| error::PropertyNotFound {
                        entity: "Room",
                        property: "exit=direction:<room>",
                        id: record.qualified_name.into(),
                    })?
                    .parse::<Title>()?;
                Ok((direction, room))
            })
            .collect::<Result<HashMap<Identifier, Title>, error::Application>>()?;
        let items = record
            .properties
            .get("items")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .map(|item_name| {
                Ok(item_map
                    .get(&item_name.parse()?)
                    .ok_or_else(|| error::EntityNotFound {
                        etype: "Item",
                        id: item_name.into(),
                    })?
                    .clone())
            })
            .collect::<Result<Vec<Rc<Item>>, error::Application>>()?;
        let characters = record
            .properties
            .get("characters")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .map(|character_name| {
                Ok(character_map
                    .get(&character_name.parse()?)
                    .ok_or_else(|| error::EntityNotFound {
                        etype: "Character",
                        id: character_name.into(),
                    })?
                    .clone())
            })
            .collect::<Result<Vec<Rc<Character>>, error::Application>>()?;
        let actions = record
            .properties
            .require("actions", &record)?
            .split(',')
            .map(str::trim)
            .map(|s| s.parse::<Identifier>())
            .collect::<Result<Vec<Identifier>, error::Application>>()?;
        let name = record.name.parse::<Title>()?;
        let room = Rc::new(
            Room::builder()
                .name(name.clone())
                .maybe_variant(record.variant.clone())
                .description(description.to_owned())
                .exits(exits)
                .actions(actions)
                .items(items)
                .characters(characters)
                .build(),
        );
        map.entry(name).or_default().insert(record.variant, room);
    }
    Ok(map)
}
