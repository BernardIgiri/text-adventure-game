mod world;

use ini::{Ini, Properties, SectionIter};
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
};
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};
use world::WorldData;

use crate::{
    entity::{
        action::{Action, ChangeRoom, GiveItem},
        invariant::{EntityName, Identifier, Title},
        room::{Item, Room},
    },
    error,
};

pub use world::World;

pub type Staging<'a> = HashMap<EntitySection, HashMap<EntityName, Record<'a>>>;

#[derive(IntoStaticStr, EnumIter, Hash, Debug, PartialEq, Eq, Clone, Copy)]
#[strum(serialize_all = "PascalCase")]
pub enum EntitySection {
    Action,
    Character,
    Dialogue,
    Item,
    Requirement,
    Response,
    Room,
}

#[derive(Debug)]
struct Record<'a> {
    section: &'a str,
    name: &'a str,
    variant: Option<Identifier>,
    properties: &'a Properties,
}

struct RecordBySection<'a>(SectionIter<'a>, &'a str);

pub fn parse(ini: Ini) -> Result<World, error::Game> {
    let mut world = WorldData::default();
    let mut staging = Staging::new();
    for section in EntitySection::iter() {
        for record in RecordBySection(ini.iter(), section.into()) {
            use EntitySection::*;
            let record = record?;
            match section {
                Item => (),
                Action | Dialogue | Response | Requirement => {
                    let name = EntityName::Identifier(record.name.parse()?);
                    staging.entry(section).or_default().insert(name, record);
                }
                Room | Character => {
                    let name = EntityName::Title(record.name.parse()?);
                    staging.entry(section).or_default().insert(name, record);
                }
            }
        }
    }
    let staging = staging;
    for record in RecordBySection(ini.iter(), EntitySection::Item.into()) {
        let record = record?;
        let description = record
            .properties
            .get("description")
            .ok_or(error::Game::MissingExpectedValue("Item description"))?;
        let name = record.name.parse::<Identifier>()?;
        let item = Item::new(name.clone(), description.to_string());
        world.item.insert(name, item);
    }
    let mut unpaired_entities: usize = staging.values().map(|inner| inner.len()).sum();
    while unpaired_entities > 0 {
        let previous_count = unpaired_entities;
        for section in EntitySection::iter() {
            use EntitySection as E;
            match section {
                E::Item => (),
                E::Action => {
                    let action_staging = match staging.get(&E::Action) {
                        Some(staged) => staged,
                        None => continue,
                    };
                    for record in action_staging.values() {
                        if let Some(change_room) = record.properties.get("change_room") {
                            if let Some(action) =
                                next_change_room_action(record, &world, &staging, change_room)?
                            {
                                world
                                    .action
                                    .insert(action.name().clone(), Action::ChangeRoom(action));
                                unpaired_entities -= 1;
                            }
                        }
                    }
                }
                _ => continue, //todo!(),
            }
        }
        if previous_count == unpaired_entities {
            let incomplete = list_incomplete_entities(&world, &staging);
            dbg!(world);
            dbg!(staging);
            return Err(error::Game::IncompleteEntities(incomplete));
        }
    }
    Ok(World::new(dbg!(world)))
}

fn list_incomplete_entities(world: &WorldData, staging: &Staging) -> Vec<String> {
    let mut list = Vec::new();
    for section in EntitySection::iter() {
        if let Some(entity_staging) = staging.get(&section) {
            use EntitySection::*;
            match section {
                Action => {
                    list.extend(
                        entity_staging
                            .keys()
                            .cloned()
                            .filter_map(|name| name.try_into().ok())
                            .collect::<HashSet<_>>()
                            .difference(&world.action.keys().cloned().collect::<HashSet<_>>())
                            .map(|name| format!("Action:{}", name)),
                    );
                }
                Character => {
                    list.extend(
                        entity_staging
                            .keys()
                            .cloned()
                            .filter_map(|name| name.try_into().ok())
                            .collect::<HashSet<_>>()
                            .difference(&world.character.keys().cloned().collect::<HashSet<_>>())
                            .map(|name| format!("Character:{}", name)),
                    );
                }
                Dialogue => {
                    list.extend(entity_staging.iter().filter_map(|(name, record)| {
                        let Ok(name) = name.clone().try_into() else {
                            return None;
                        };
                        if world.dialogue.contains_key(&name) {
                            None
                        } else {
                            let qualified_name =
                                qualify_entity_name(name.as_ref(), &record.variant);
                            Some(format!("Dialogue:{}", qualified_name))
                        }
                    }));
                }
                Item => {
                    list.extend(
                        entity_staging
                            .keys()
                            .cloned()
                            .filter_map(|name| name.try_into().ok())
                            .collect::<HashSet<_>>()
                            .difference(&world.item.keys().cloned().collect::<HashSet<_>>())
                            .map(|name| format!("Item:{}", name)),
                    );
                }
                Requirement => {
                    list.extend(
                        entity_staging
                            .keys()
                            .cloned()
                            .filter_map(|name| name.try_into().ok())
                            .collect::<HashSet<_>>()
                            .difference(&world.requirement.keys().cloned().collect::<HashSet<_>>())
                            .map(|name| format!("Requirement:{}", name)),
                    );
                }
                Response => {
                    list.extend(
                        entity_staging
                            .keys()
                            .cloned()
                            .filter_map(|name| name.try_into().ok())
                            .collect::<HashSet<_>>()
                            .difference(&world.response.keys().cloned().collect::<HashSet<_>>())
                            .map(|name| format!("Response:{}", name)),
                    );
                }
                Room => {
                    list.extend(entity_staging.iter().filter_map(|(name, record)| {
                        let Ok(name) = name.clone().try_into() else {
                            return None;
                        };
                        if world.room.contains_key(&name) {
                            None
                        } else {
                            let qualified_name =
                                qualify_entity_name(name.as_ref(), &record.variant);
                            Some(format!("Room:{}", qualified_name))
                        }
                    }));
                }
            };
        }
    }
    list
}

impl<'a> Iterator for RecordBySection<'a> {
    type Item = Result<Record<'a>, error::Game>;

    fn next(&mut self) -> Option<Self::Item> {
        for (input_opt, properties) in &mut self.0 {
            let input = match input_opt {
                Some(i) => i,
                None => continue,
            };
            match get_record(input, properties) {
                Ok(r) if r.section.trim().eq_ignore_ascii_case(self.1.trim()) => {
                    return Some(Ok(r));
                }
                Err(e) => return Some(Err(e)),
                _ => continue,
            }
        }
        None
    }
}

fn get_record<'a>(input: &'a str, properties: &'a Properties) -> Result<Record<'a>, error::Game> {
    let mut section_parts = input.split(':');
    let section = section_parts
        .next()
        .ok_or(error::Game::MissingExpectedValue("Section Name"))?;
    let qualified_entity_name = section_parts
        .next()
        .ok_or(error::Game::MissingExpectedValue("Qualified Entity Name"))?;
    let mut entity_name_parts = qualified_entity_name.split('|');
    let name = entity_name_parts
        .next()
        .ok_or(error::Game::MissingExpectedValue("Entity Name"))?;
    let variant = entity_name_parts
        .next()
        .map(str::parse::<Identifier>)
        .transpose()?;
    Ok(Record {
        section,
        name,
        variant,
        properties,
    })
}

fn next_action(
    record: &Record,
    world: &WorldData,
    staging: &Staging,
) -> Result<Option<Action>, error::Game> {
    todo!();
}

fn next_change_room_action(
    record: &Record,
    world: &WorldData,
    staging: &Staging,
    change_room: &str,
) -> Result<Option<ChangeRoom>, error::Game> {
    use EntityName as N;
    use EntitySection as E;
    let (room_name, variant) = {
        let mut parts = change_room.split("->");
        let room_name = parts
            .next()
            .ok_or(error::Game::MissingExpectedValue("Change Room Action Room"))?
            .parse::<Title>()?;
        let variant = match parts.next() {
            Some(v) => Some(v.parse::<Identifier>()?),
            None => None,
        };
        (room_name, variant)
    };
    let description =
        record
            .properties
            .get("description")
            .ok_or(error::Game::MissingExpectedValue(
                "Change Action Description",
            ))?;
    if !staging
        .get(&E::Room)
        .ok_or(error::Game::NoDataForEntityType("Room"))?
        .contains_key(&N::Title(room_name.clone()))
    {
        return Err(error::Game::MissingEntity {
            etype: E::Room.into(),
            id: change_room.to_string(),
        });
    }
    let room = match get_room_variant(world, &room_name, &variant) {
        Some(r) => r,
        None => return Ok(None),
    };
    let required = match record.properties.get("required") {
        Some(item_name) => Some(
            world
                .item
                .get(&item_name.parse()?)
                .ok_or_else(|| error::Game::MissingEntity {
                    etype: "Item",
                    id: item_name.into(),
                })?
                .clone(),
        ),
        None => None,
    };
    Ok(Some(
        ChangeRoom::builder()
            .name(record.name.parse()?)
            .description(description.into())
            .room(room.clone())
            .maybe_required(required)
            .build(),
    ))
}

fn get_give_item_action(
    record: &Record,
    world: &WorldData,
    staging: &Staging,
    change_room: &str,
) -> Result<GiveItem, error::Game> {
    todo!()
}

fn get_room_variant<'a>(
    world: &'a WorldData,
    room_name: &'a Title,
    variant: &'a Option<Identifier>,
) -> Option<&'a Room> {
    world.room.get(room_name).map(|r| r.get(variant)).flatten()
}

fn qualify_entity_name(name: &str, variant: &Option<Identifier>) -> String {
    let mut qualified = String::new();
    write!(qualified, "{}", name).unwrap();
    if let Some(v) = variant {
        write!(qualified, "|{}", v).unwrap();
    }
    qualified
}
