use std::rc::Rc;

use ini::SectionIter;

use crate::{
    config_parser::{
        iter::{EntitySection, SectionRecordIter},
        types::RoomVariant,
    },
    error,
    world::{Action, ChangeRoom, GiveItem, Identifier, Item, ReplaceItem, TakeItem, Title},
};

use super::{
    iter::{Record, RequireProperty},
    types::{ActionMap, ItemMap, RoomMap},
};

type ActionResult = Result<Option<(Identifier, Action)>, error::Application>;

pub fn parse_actions<'a>(
    ini_iter: SectionIter<'a>,
    room_map: &RoomMap,
    item_map: &ItemMap,
) -> Result<ActionMap, error::Application> {
    let mut map = ActionMap::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Action.into()) {
        let record = record?;
        let action = if record.properties.contains_key("change_room") {
            next_change_room_action(&record, room_map, item_map)
        } else if record.properties.contains_key("replace_item") {
            next_replace_item_action(&record, item_map)
        } else if record.properties.contains_key("give_item") {
            next_give_item_action(&record, item_map)
        } else if record.properties.contains_key("take_item") {
            next_take_item_action(&record, item_map)
        } else {
            Err(error::EntityDataIncomplete("Action"))
        }?;
        if let Some((name, action)) = action {
            map.insert(name, Rc::new(action));
        }
    }
    Ok(map)
}

fn next_change_room_action(
    record: &Record,
    room_map: &RoomMap,
    item_map: &ItemMap,
) -> ActionResult {
    let (room_name, variant) = {
        let change_room =
            record
                .properties
                .get("change_room")
                .ok_or_else(|| error::PropertyNotFound {
                    entity: "Room",
                    property: "change_room",
                    id: record.qualified_name.into(),
                })?;
        let mut parts = change_room.splitn(2, "->");
        let room_name = parts
            .next()
            .ok_or_else(|| error::PropertyNotFound {
                entity: "Action",
                property: "change_room:<name>",
                id: record.qualified_name.into(),
            })?
            .trim()
            .parse::<Title>()?;
        let variant = match parts.next() {
            Some(v) => Some(v.trim().parse::<Identifier>()?),
            None => None,
        };
        (room_name, variant)
    };
    let description = record.properties.require("description", record)?;
    let room = match room_map.get_room(&room_name, &variant) {
        Some(r) => r,
        None => return Ok(None),
    };
    let required = required_item_from_record(record, item_map)?;
    let name = record.name.parse::<Identifier>()?;
    Ok(Some((
        name.clone(),
        Action::ChangeRoom(
            ChangeRoom::builder()
                .name(name)
                .description(description.into())
                .room(room)
                .maybe_required(required)
                .build(),
        ),
    )))
}

fn next_give_item_action(record: &Record, item_map: &ItemMap) -> ActionResult {
    let items = items_from_record(record, "give_item", item_map)?;
    let description = record.properties.require("description", record)?;
    let name = record.name.parse::<Identifier>()?;
    Ok(Some((
        name.clone(),
        Action::GiveItem(
            GiveItem::builder()
                .name(name)
                .description(description.into())
                .items(items)
                .build(),
        ),
    )))
}

fn next_replace_item_action(record: &Record, item_map: &ItemMap) -> ActionResult {
    let description = record.properties.require("description", record)?;
    let original = record
        .properties
        .require("original", record)
        .and_then(|item_name| require_item_from_map(item_name, item_map))?;
    let replacement = record
        .properties
        .require("replacement", record)
        .and_then(|item_name| require_item_from_map(item_name, item_map))?;
    let name = record.name.parse::<Identifier>()?;
    Ok(Some((
        name,
        Action::ReplaceItem(
            ReplaceItem::builder()
                .name(record.name.parse()?)
                .description(description.into())
                .original(original)
                .replacement(replacement)
                .build(),
        ),
    )))
}

fn next_take_item_action(record: &Record, item_map: &ItemMap) -> ActionResult {
    let items = items_from_record(record, "take_item", item_map)?;
    let description = record.properties.require("description", record)?;
    let required = required_item_from_record(record, item_map)?;
    let name = record.name.parse::<Identifier>()?;
    Ok(Some((
        name.clone(),
        Action::TakeItem(
            TakeItem::builder()
                .name(name)
                .description(description.into())
                .items(items)
                .maybe_required(required)
                .build(),
        ),
    )))
}

fn items_from_record<'a>(
    record: &'a Record<'a>,
    prop: &'static str,
    item_map: &'a ItemMap,
) -> Result<Vec<Rc<Item>>, error::Application> {
    record
        .properties
        .require(prop, record)?
        .split(',')
        .map(str::trim)
        .map(|item_name| require_item_from_map(item_name, item_map))
        .collect()
}

fn required_item_from_record<'a>(
    record: &'a Record<'a>,
    item_map: &'a ItemMap,
) -> Result<Option<Rc<Item>>, error::Application> {
    let required = record.properties.get("required").filter(|s| !s.is_empty());
    match required {
        Some(item_name) => Ok(Some(require_item_from_map(item_name, item_map)?)),
        None => Ok(None),
    }
}

fn require_item_from_map(
    item_name: &str,
    item_map: &ItemMap,
) -> Result<Rc<Item>, error::Application> {
    Ok(item_map
        .get(&item_name.parse()?)
        .ok_or_else(|| error::EntityNotFound {
            etype: "Item",
            id: item_name.into(),
        })?
        .clone())
}
