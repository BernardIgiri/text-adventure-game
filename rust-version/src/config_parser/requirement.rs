use std::rc::Rc;

use crate::{
    error,
    world::{Identifier, Requirement},
};

use super::{
    iter::{title_variant_from_qualified, ListProperty, Record},
    types::{ItemMap, RoomMap, RoomVariant},
};

pub fn parse_requirements(
    record: &Record,
    entity_type: &'static str,
    item_map: &ItemMap,
    room_map: &RoomMap,
) -> Result<Vec<Rc<Requirement>>, error::Application> {
    record
        .properties
        .get_list("requires")
        .map(|s| parse_one_requirement(record, entity_type, item_map, room_map, s))
        .collect()
}

fn parse_one_requirement(
    record: &Record,
    entity_type: &'static str,
    item_map: &ItemMap,
    room_map: &RoomMap,
    string: &str,
) -> Result<Rc<Requirement>, error::Application> {
    let mut parts = string.splitn(2, ':').map(str::trim);
    let r_type = parts
        .next()
        .ok_or_else(|| error::PropertyNotFound {
            entity: entity_type,
            property: "requires:<requirement_type>",
            id: record.qualified_name.into(),
        })?
        .to_lowercase();
    let requirement = match r_type.as_str() {
        "has_item" => {
            let item_name = parts.next().ok_or_else(|| error::PropertyNotFound {
                entity: entity_type,
                property: "requires:has_item:<item_id>",
                id: record.qualified_name.into(),
            })?;
            let item_name: Identifier = item_name.parse()?;
            let item = item_map
                .get(&item_name)
                .cloned()
                .ok_or_else(|| error::EntityNotFound {
                    etype: "Item",
                    id: item_name.to_string(),
                })?;

            Requirement::HasItem(item)
        }
        "room_variant" => {
            let qualified_name = parts.next().ok_or_else(|| error::PropertyNotFound {
                entity: entity_type,
                property: "requires:room_variant:<room>",
                id: record.qualified_name.into(),
            })?;
            let (room_name, variant) = title_variant_from_qualified(qualified_name)?;
            let room =
                room_map
                    .get_room(&room_name, &variant)
                    .ok_or_else(|| error::EntityNotFound {
                        etype: "Room",
                        id: qualified_name.into(),
                    })?;

            Requirement::RoomVariant(room)
        }
        _ => {
            return Err(error::InvalidPropertyValue {
                value: string.into(),
                field: "requirement",
            });
        }
    };
    Ok(Rc::new(requirement))
}
