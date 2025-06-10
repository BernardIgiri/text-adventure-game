use crate::{
    config_parser::{
        staging::{get_room_variant, EntitySection, StagedEntity, Staging},
        world::WorldData,
    },
    entity::{
        Action, ChangeRoom, EntityName, GiveItem, Identifier, Item, ReplaceItem, TakeItem, Title,
    },
    error,
};

pub fn list_actions(
    staging: &Staging,
    world: &WorldData,
) -> Result<Vec<(Identifier, Action)>, error::Game> {
    Ok(staging
        .get(&EntitySection::Action)
        .into_iter()
        .flat_map(|map| map.values())
        .map(|staged| {
            if staged.properties.contains_key("change_room") {
                next_change_room_action(staged, world)
                    .map(|opt| opt.map(|a| (a.name().clone(), Action::ChangeRoom(a))))
            } else if staged.properties.contains_key("replace_item") {
                next_replace_item_action(staged, world)
                    .map(|opt| opt.map(|a| (a.name().clone(), Action::ReplaceItem(a))))
            } else if staged.properties.contains_key("give_item") {
                next_give_item_action(staged, world)
                    .map(|opt| opt.map(|a| (a.name().clone(), Action::GiveItem(a))))
            } else if staged.properties.contains_key("take_item") {
                next_take_item_action(staged, world)
                    .map(|opt| opt.map(|a| (a.name().clone(), Action::TakeItem(a))))
            } else {
                Err(error::Game::EntityDataIncomplete("Action"))
            }
        })
        .collect::<Result<Vec<Option<(Identifier, Action)>>, error::Game>>()?
        .into_iter()
        .flatten()
        .collect())
}

fn next_change_room_action(
    staged: &StagedEntity,
    world: &WorldData,
) -> Result<Option<ChangeRoom>, error::Game> {
    let (room_name, variant) =
        {
            let change_room = staged.properties.get("change_room").ok_or_else(|| {
                error::Game::PropertyNotFound {
                    entity: "Room",
                    property: "change_room",
                    id: staged.qualified_name.into(),
                }
            })?;
            let mut parts = change_room.split("->");
            let room_name = parts
                .next()
                .ok_or_else(|| error::Game::PropertyNotFound {
                    entity: "Action",
                    property: "change_room:<name>",
                    id: staged.qualified_name.into(),
                })?
                .parse::<Title>()?;
            let variant = match parts.next() {
                Some(v) => Some(v.parse::<Identifier>()?),
                None => None,
            };
            (room_name, variant)
        };
    let description = description_from_staged(staged)?;
    let room = match get_room_variant(world, &room_name, &variant) {
        Some(r) => r,
        None => return Ok(None),
    };
    let required = required_item_from_staged(staged, world)?;
    Ok(Some(
        ChangeRoom::builder()
            .name(staged.name.parse()?)
            .description(description.into())
            .room(room.clone())
            .maybe_required(required)
            .build(),
    ))
}

fn next_give_item_action(
    staged: &StagedEntity,
    world: &WorldData,
) -> Result<Option<GiveItem>, error::Game> {
    let items = items_from_staged(staged, world)?;
    let description = description_from_staged(staged)?;
    Ok(Some(
        GiveItem::builder()
            .name(staged.name.parse()?)
            .description(description.into())
            .items(items)
            .build(),
    ))
}

// TODO: Implement later
#[allow(unused_variables)]
fn next_replace_item_action(
    staged: &StagedEntity,
    world: &WorldData,
) -> Result<Option<ReplaceItem>, error::Game> {
    let description = description_from_staged(staged)?;
    // let original = staged.properties.get("original")
    //     .ok_or(error)
    todo!()
}

fn next_take_item_action(
    staged: &StagedEntity,
    world: &WorldData,
) -> Result<Option<TakeItem>, error::Game> {
    let items = items_from_staged(staged, world)?;
    let description = description_from_staged(staged)?;
    let required = required_item_from_staged(staged, world)?;
    Ok(Some(
        TakeItem::builder()
            .name(staged.name.parse()?)
            .description(description.into())
            .items(items)
            .maybe_required(required)
            .build(),
    ))
}

fn description_from_staged<'a>(staged: &'a StagedEntity<'a>) -> Result<&'a str, error::Game> {
    staged
        .properties
        .get("description")
        .ok_or_else(|| error::Game::PropertyNotFound {
            entity: "Action",
            property: "description",
            id: staged.qualified_name.into(),
        })
}

fn items_from_staged<'a>(
    staged: &'a StagedEntity<'a>,
    world: &'a WorldData,
) -> Result<Vec<Item>, error::Game> {
    staged
        .properties
        .get("items")
        .ok_or(error::Game::PropertyNotFound {
            entity: "Action",
            property: "items",
            id: staged.qualified_name.into(),
        })?
        .split(",")
        .map(str::trim)
        .map(|item_name| item_from_world(item_name, world))
        .collect()
}

fn required_item_from_staged<'a>(
    staged: &'a StagedEntity<'a>,
    world: &'a WorldData,
) -> Result<Option<Item>, error::Game> {
    let required = staged.properties.get("required").filter(|s| !s.is_empty());
    match required {
        Some(item_name) => Ok(Some(item_from_world(item_name, world)?)),
        None => Ok(None),
    }
}

fn item_from_world(item_name: &str, world: &WorldData) -> Result<Item, error::Game> {
    Ok(world
        .item
        .get(&item_name.parse()?)
        .ok_or_else(|| error::Game::EntityNotFound {
            etype: "Item",
            id: item_name.into(),
        })?
        .clone())
}
