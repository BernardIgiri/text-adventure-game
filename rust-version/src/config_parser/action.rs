use crate::{
    config_parser::{
        staging::{get_room_variant, EntitySection, StagedEntity, Staging},
        world::WorldData,
    },
    entity::{
        action::{Action, ChangeRoom, GiveItem},
        invariant::{EntityName, Identifier, Title},
    },
    error,
};

pub fn list_actions(
    staging: &Staging,
    world: &WorldData,
) -> Result<Vec<(Identifier, Action)>, error::Game> {
    staging
        .get(&EntitySection::Action)
        .into_iter()
        .flat_map(|map| map.values())
        .filter_map(|staged| {
            staged
                .properties
                .get("change_room")
                .map(|val| (staged, val))
        })
        .map(|(staged, val)| {
            next_change_room_action(staged, world, staging, val)
                .map(|opt| opt.map(|a| (a.name().clone(), Action::ChangeRoom(a))))
        })
        .collect::<Result<Option<Vec<_>>, _>>()
        .map(|opt| opt.unwrap_or_default())
}

fn next_change_room_action(
    staged: &StagedEntity,
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
        staged
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
    let required = match staged.properties.get("required") {
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
    staging: &Staging,
    change_room: &str,
) -> Result<GiveItem, error::Game> {
    todo!()
}
