use ini::Properties;
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
};
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

use crate::entity::{
    invariant::{EntityName, Identifier, Title},
    room::Room,
};

use super::world::WorldData;

pub type EntityStaging<'a> = HashMap<EntityName, StagedEntity<'a>>;
pub type Staging<'a> = HashMap<EntitySection, EntityStaging<'a>>;

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
pub struct StagedEntity<'a> {
    pub section: &'a str,
    pub name: &'a str,
    pub variant: Option<Identifier>,
    pub properties: &'a Properties,
}

pub fn list_incomplete_entities(world: &WorldData, staging: &Staging) -> Vec<String> {
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
                    list.extend(entity_staging.iter().filter_map(|(name, staged)| {
                        let Ok(name) = name.clone().try_into() else {
                            return None;
                        };
                        if world.dialogue.contains_key(&name) {
                            None
                        } else {
                            let qualified_name =
                                qualify_entity_name(name.as_ref(), &staged.variant);
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
                    list.extend(entity_staging.iter().filter_map(|(name, staged)| {
                        let Ok(name) = name.clone().try_into() else {
                            return None;
                        };
                        if world.room.contains_key(&name) {
                            None
                        } else {
                            let qualified_name =
                                qualify_entity_name(name.as_ref(), &staged.variant);
                            Some(format!("Room:{}", qualified_name))
                        }
                    }));
                }
            };
        }
    }
    list
}

pub fn get_room_variant<'a>(
    world: &'a WorldData,
    room_name: &'a Title,
    variant: &'a Option<Identifier>,
) -> Option<&'a Room> {
    world.room.get(room_name).and_then(|r| r.get(variant))
}

pub fn qualify_entity_name(name: &str, variant: &Option<Identifier>) -> String {
    let mut qualified = String::new();
    write!(qualified, "{}", name).unwrap();
    if let Some(v) = variant {
        write!(qualified, "|{}", v).unwrap();
    }
    qualified
}
