use std::rc::Rc;

use ini::SectionIter;

use crate::{
    config_parser::{
        iter::{EntitySection, SectionRecordIter},
        types::RoomVariant,
    },
    core::{
        Action, ChangeRoom, GiveItem, Identifier, Item, ReplaceItem, Sequence, TakeItem, Teleport,
        Title,
    },
    error,
};

use super::{
    iter::{Record, UnverifiedRecord},
    types::{ActionMap, ItemMap, RoomMap},
};

type ActionResult = Result<Option<(Identifier, Action)>, error::Application>;

pub fn parse_actions<'a>(
    ini_iter: SectionIter<'a>,
    room_map: &RoomMap,
    item_map: &ItemMap,
) -> Result<ActionMap, error::Application> {
    let mut map = ActionMap::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Action) {
        let record = record?;
        let action = if record.contains_key("change_room") {
            next_change_room_action(record, room_map, item_map)
        } else if record.contains_key("teleport_to") {
            next_teleport_action(record, item_map)
        } else if record.contains_key("replace_item") {
            next_replace_item_action(record, item_map)
        } else if record.contains_key("give_item") {
            next_give_item_action(record, item_map)
        } else if record.contains_key("take_item") {
            next_take_item_action(record, item_map)
        } else if record.contains_key("sequence") {
            next_sequence_action(record, item_map)
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
    record: UnverifiedRecord,
    room_map: &RoomMap,
    item_map: &ItemMap,
) -> ActionResult {
    let record = record.into_record(&["change_room", "description"], &["required"])?;
    let (room_name, variant) = {
        let change_room = record.require("change_room")?;
        let mut parts = change_room.splitn(2, "->");
        let room_name = parts
            .next()
            .ok_or_else(|| error::PropertyNotFound {
                entity: "Action",
                property: "change_room:<name>",
                id: record.qualified_name().into(),
            })?
            .trim()
            .parse::<Title>()
            .map_err(|source| error::ConversionFailed {
                etype: "Action",
                property: "change_room:<RoomName>",
                source,
            })?;
        let variant =
            match parts.next() {
                Some(v) => Some(v.trim().parse::<Identifier>().map_err(|source| {
                    error::ConversionFailed {
                        etype: "Action",
                        property: "change_room:RoomName-><variant>",
                        source,
                    }
                })?),
                None => None,
            };
        (room_name, variant)
    };
    let description = record.require("description")?;
    let room = match room_map.get_room(&room_name, &variant) {
        Some(r) => r,
        None => return Ok(None),
    };
    let required = required_item_from_record(&record, item_map)?;
    let name = record.parse_name::<Identifier>()?;
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

fn next_teleport_action(record: UnverifiedRecord, item_map: &ItemMap) -> ActionResult {
    let record = record.into_record(&["teleport_to", "description"], &["required"])?;
    let room_name = record.require_parsed("teleport_to")?;
    let description = record.require("description")?;
    let required = required_item_from_record(&record, item_map)?;
    let name = record.parse_name::<Identifier>()?;
    Ok(Some((
        name.clone(),
        Action::Teleport(
            Teleport::builder()
                .name(name)
                .description(description.into())
                .room_name(room_name)
                .maybe_required(required)
                .build(),
        ),
    )))
}

fn next_give_item_action(record: UnverifiedRecord, item_map: &ItemMap) -> ActionResult {
    let record = record.into_record(&["give_item", "description"], &[])?;
    let items = items_from_record(&record, "give_item", item_map)?;
    let description = record.require("description")?;
    let required = required_item_from_record(&record, item_map)?;
    let name = record.parse_name::<Identifier>()?;
    Ok(Some((
        name.clone(),
        Action::GiveItem(
            GiveItem::builder()
                .name(name)
                .description(description.into())
                .items(items)
                .maybe_required(required)
                .build(),
        ),
    )))
}

fn next_take_item_action(record: UnverifiedRecord, item_map: &ItemMap) -> ActionResult {
    let record = record.into_record(&["take_item", "description"], &["required"])?;
    let items = items_from_record(&record, "take_item", item_map)?;
    let description = record.require("description")?;
    let name = record.parse_name::<Identifier>()?;
    Ok(Some((
        name.clone(),
        Action::TakeItem(
            TakeItem::builder()
                .name(name)
                .description(description.into())
                .items(items)
                .build(),
        ),
    )))
}

fn next_replace_item_action(record: UnverifiedRecord, item_map: &ItemMap) -> ActionResult {
    let record = record.into_record(&["replace_item", "description"], &[])?;
    let description = record.require("description")?;
    let replace_item = record.require("replace_item")?;
    let mut parts = replace_item.splitn(2, "->");
    let original = parts.next().ok_or_else(|| error::PropertyNotFound {
        entity: "Action",
        property: "replace_item:<original>",
        id: record.qualified_name().into(),
    })?;
    let original = require_item_from_map(original, item_map, "replace_item:<original>")?;
    let replacement = parts.next().ok_or_else(|| error::PropertyNotFound {
        entity: "Action",
        property: "replace_item:original-><replacement>",
        id: record.qualified_name().into(),
    })?;
    let replacement = require_item_from_map(
        replacement,
        item_map,
        "replace_item:original-><replacement>",
    )?;
    let name = record.parse_name::<Identifier>()?;
    Ok(Some((
        name.clone(),
        Action::ReplaceItem(
            ReplaceItem::builder()
                .name(name)
                .description(description.into())
                .original(original)
                .replacement(replacement)
                .build(),
        ),
    )))
}

fn next_sequence_action(record: UnverifiedRecord, item_map: &ItemMap) -> ActionResult {
    let record = record.into_record(&["sequence", "description"], &["required"])?;
    let actions = record
        .get_list("sequence")
        .map(|s| s.trim().parse::<Identifier>())
        .collect::<Result<Vec<Identifier>, _>>()
        .map_err(|source| error::ConversionFailed {
            etype: "Action",
            property: "sequence",
            source,
        })?;
    let description = record.require("description")?;
    let required = required_item_from_record(&record, item_map)?;
    let name = record.parse_name::<Identifier>()?;
    Ok(Some((
        name.clone(),
        Action::Sequence(
            Sequence::builder()
                .name(name)
                .description(description.into())
                .maybe_required(required)
                .actions(actions)
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
        .require_list(prop)?
        .map(|item_name| require_item_from_map(item_name, item_map, prop))
        .collect()
}

fn required_item_from_record<'a>(
    record: &'a Record<'a>,
    item_map: &'a ItemMap,
) -> Result<Option<Rc<Item>>, error::Application> {
    let required = record.get("required").filter(|s| !s.is_empty());
    match required {
        Some(item_name) => Ok(Some(require_item_from_map(
            item_name, item_map, "required",
        )?)),
        None => Ok(None),
    }
}

fn require_item_from_map(
    item_name: &str,
    item_map: &ItemMap,
    property_name: &'static str,
) -> Result<Rc<Item>, error::Application> {
    Ok(item_map
        .get(
            &item_name
                .parse()
                .map_err(|source| error::ConversionFailed {
                    etype: "Action",
                    property: property_name,
                    source,
                })?,
        )
        .ok_or_else(|| error::EntityNotFound {
            etype: "Item",
            id: item_name.into(),
        })?
        .clone())
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use asserting::prelude::*;
    use ini::Ini;

    use crate::{
        config_parser::test_utils::{
            data::{item_map, room_map},
            i, t,
        },
        core::Action,
    };

    use super::parse_actions;

    const GOOD_DATA: &str = r"
        [Action:pull_lever]
        change_room=WoodShed->closed
        description=You pull the hefty lever and hear a sastifying clunk! Immediately, the lights go out, and lever seizes in place!

        [Action:pay_bribe]
        take_item=silver_coin
        description=You give away your last coin begrudgingly.

        [Action:unlock_chest]
        replace_item=key->ring
        description=You unlock the chest and discover a golden ring!

        [Action:pickup_key]
        give_item=key
        description=You pick up the dingy key on the floor.
        
        [Action:beam_me_up]
        teleport_to=Enterprise
        required=silver_coin
        description=Scotty teleports you abord the ship!
    ";

    #[test]
    fn good_data() {
        let ini = Ini::load_from_str(GOOD_DATA).unwrap();
        let items = item_map();
        let rooms = room_map(true);
        let actions = parse_actions(ini.iter(), &rooms, &items).unwrap();

        assert_that!(&actions)
            .has_length(5)
            .contains_key(i("pull_lever"))
            .contains_key(i("pay_bribe"))
            .contains_key(i("unlock_chest"))
            .contains_key(i("pickup_key"))
            .contains_key(i("beam_me_up"));

        let pull_lever = actions.get(&i("pull_lever")).unwrap();
        assert_that!(pull_lever).satisfies_with_message("expected ChangeRoom", |a| {
            matches!(a.as_ref(), Action::ChangeRoom(a)
                if a.description().contains("hefty lever")
                && a.room().name() == &t("WoodShed")
                && a.room().variant() == &Some(i("closed"))
            )
        });

        let pay_bribe = actions.get(&i("pay_bribe")).unwrap();
        assert_that!(pay_bribe).satisfies_with_message("expected GiveItem", |a| {
            matches!(a.as_ref(), Action::TakeItem(a)
                if a.description().contains("begrudgingly")
                && a.items().len() == 1
                && a.items()[0].name() == &i("silver_coin")
            )
        });

        let unlock_chest = actions.get(&i("unlock_chest")).unwrap();
        assert_that!(unlock_chest).satisfies_with_message("expected ReplaceItem", |a| {
            matches!(a.as_ref(), Action::ReplaceItem(a)
                if a.description().contains("golden ring")
                && a.original().name() == &i("key")
                && a.replacement().name() == &i("ring")
            )
        });

        let pickup_key = actions.get(&i("pickup_key")).unwrap();
        assert_that!(pickup_key).satisfies_with_message("expected TakeItem", |a| {
            matches!(a.as_ref(), Action::GiveItem(a)
                if a.description().contains("dingy key")
                && a.required().is_none()
                && a.items().len() == 1
                && a.items()[0].name() == &i("key")
            )
        });

        let teleport = actions.get(&i("beam_me_up")).unwrap();
        assert_that!(teleport).satisfies_with_message("expected Teleport", |a| {
            matches!(a.as_ref(), Action::Teleport(tp)
                if tp.description().contains("teleports you")
                && tp.required().as_ref().map(|i| i.name()) == Some(&i("silver_coin"))
                && tp.room_name().clone() == t("Enterprise")
            )
        });
    }
}
