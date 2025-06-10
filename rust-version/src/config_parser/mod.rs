mod action;
mod staging;
mod world;

use action::list_actions;
use ini::{Ini, Properties, SectionIter};
use staging::{list_incomplete_entities, EntitySection, StagedEntity, Staging};
use strum::IntoEnumIterator;
use world::WorldData;

use crate::{
    entity::{
        invariant::{EntityName, Identifier},
        room::Item,
    },
    error,
};

pub use world::World;

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
                    let list = list_actions(&staging, &world)?;
                    unpaired_entities -= list.len();
                    world.action.extend(list);
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

impl<'a> Iterator for RecordBySection<'a> {
    type Item = Result<StagedEntity<'a>, error::Game>;

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

fn get_record<'a>(
    input: &'a str,
    properties: &'a Properties,
) -> Result<StagedEntity<'a>, error::Game> {
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
    Ok(StagedEntity {
        section,
        name,
        variant,
        properties,
    })
}
