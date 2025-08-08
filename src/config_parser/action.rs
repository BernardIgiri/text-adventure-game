use ini::SectionIter;

use crate::{
    config_parser::iter::{EntitySection, SectionRecordIter},
    core::{
        ActionRaw, ChangeRoomRaw, GiveItemRaw, Identifier, ReplaceItemRaw, SequenceRaw,
        TakeItemRaw, TeleportRaw,
    },
    error,
};

use super::iter::{IterRequireWith, ParseWith, UnverifiedRecord};

type ActionResult = Result<ActionRaw, error::Application>;

pub fn parse_actions<'a>(ini_iter: SectionIter<'a>) -> Result<Vec<ActionRaw>, error::Application> {
    let mut list = Vec::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Action) {
        let record = record?;
        let action = if record.contains_key("change_room") {
            next_change_room_action(record)
        } else if record.contains_key("teleport_to") {
            next_teleport_action(record)
        } else if record.contains_key("replace_item") {
            next_replace_item_action(record)
        } else if record.contains_key("give_item") {
            next_give_item_action(record)
        } else if record.contains_key("take_item") {
            next_take_item_action(record)
        } else if record.contains_key("sequence") {
            next_sequence_action(record)
        } else {
            Err(error::EntityDataIncomplete("Action".into()))
        }?;
        list.push(action);
    }
    Ok(list)
}

fn next_change_room_action(record: UnverifiedRecord) -> ActionResult {
    let record = record.into_record(&["change_room", "description"], &["required"])?;
    let (room, variant) = {
        let change_room = record.require("change_room")?;
        let mut parts = change_room.splitn(2, "->");
        let room = parts
            .require_next(&record, "change_room:<name>")?
            .trim()
            .parse_with(&record, "change_room:<RoomName>")?;
        let variant = match parts.next() {
            Some(v) => Some(
                v.trim()
                    .parse_with(&record, "change_room:RoomName-><variant>")?,
            ),
            None => None,
        };
        (room, variant)
    };
    let description = record.require("description")?.to_string();
    let required = record.get_parsed("required")?;
    let name = record.parse_name::<Identifier>()?;
    Ok(ActionRaw::ChangeRoom(ChangeRoomRaw {
        name,
        description,
        required,
        room,
        variant,
    }))
}

fn next_teleport_action(record: UnverifiedRecord) -> ActionResult {
    let record = record.into_record(&["teleport_to", "description"], &["required"])?;
    let room = record.require_parsed("teleport_to")?;
    let description = record.require("description")?.to_string();
    let required = record.get_parsed("required")?;
    let name = record.parse_name::<Identifier>()?;
    Ok(ActionRaw::Teleport(TeleportRaw {
        name,
        description,
        required,
        room,
    }))
}

fn next_give_item_action(record: UnverifiedRecord) -> ActionResult {
    let record = record.into_record(&["give_item", "description"], &[])?;
    let items = record
        .get_list_parsed("give_item")
        .collect::<Result<Vec<_>, error::Application>>()?;
    let description = record.require("description")?.to_string();
    let required = record.get_parsed("required")?;
    let name = record.parse_name::<Identifier>()?;
    Ok(ActionRaw::GiveItem(GiveItemRaw {
        name,
        description,
        required,
        items,
    }))
}

fn next_take_item_action(record: UnverifiedRecord) -> ActionResult {
    let record = record.into_record(&["take_item", "description"], &["required"])?;
    let items = record
        .get_list_parsed("give_item")
        .collect::<Result<Vec<_>, error::Application>>()?;
    let description = record.require("description")?.to_string();
    let name = record.parse_name::<Identifier>()?;
    Ok(ActionRaw::TakeItem(TakeItemRaw {
        name,
        description,
        items,
    }))
}

fn next_replace_item_action(record: UnverifiedRecord) -> ActionResult {
    let record = record.into_record(&["replace_item", "description"], &[])?;
    let description = record.require("description")?.to_string();
    let replace_item = record.require("replace_item")?;
    let mut parts = replace_item.splitn(2, "->");
    let original = parts.require_next(&record, "replace_item:<original>")?;
    let original = original.parse_with(&record, "replace_item:<original>")?;
    let replacement = parts.require_next(&record, "replace_item:original-><replacement>")?;
    let replacement = replacement.parse_with(&record, "replace_item:original-><replacement>")?;
    let name = record.parse_name::<Identifier>()?;
    Ok(ActionRaw::ReplaceItem(ReplaceItemRaw {
        name,
        description,
        original,
        replacement,
    }))
}

fn next_sequence_action(record: UnverifiedRecord) -> ActionResult {
    let record = record.into_record(&["sequence", "description"], &["required"])?;
    let actions = record
        .get_list("sequence")
        .map(|s| s.trim().parse_with(&record, "sequence"))
        .collect::<Result<Vec<Identifier>, _>>()?;
    let description = record.require("description")?.to_string();
    let required = record.get_parsed("required")?;
    let name = record.parse_name::<Identifier>()?;
    Ok(ActionRaw::Sequence(SequenceRaw {
        name,
        description,
        required,
        actions,
    }))
}
