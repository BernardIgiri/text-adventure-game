use indexmap::IndexMap;
use ini::SectionIter;

use crate::{
    core::{Identifier, RoomRaw, Title},
    error,
};

use super::iter::{EntitySection, SectionRecordIter};

pub fn parse_rooms<'a>(ini_iter: SectionIter<'a>) -> Result<Vec<RoomRaw>, error::Application> {
    let mut list = Vec::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Room) {
        let record = record?.into_record(&["description"], &["characters", "exits", "actions"])?;
        let description = record.require("description")?.to_string();
        let exits = record
            .get_list("exits")
            .map(|exit| {
                let mut parts = exit.split(":");
                let direction = parts
                    .next()
                    .ok_or_else(|| error::PropertyNotFound {
                        etype: "Room",
                        property: "exit:<direction>",
                        id: record.qualified_name().into(),
                    })?
                    .trim()
                    .parse::<Identifier>()
                    .map_err(|source| error::ConversionFailed {
                        etype: "Room",
                        property: "exit:<direction>",
                        source,
                    })?;
                let room = parts
                    .next()
                    .ok_or_else(|| error::PropertyNotFound {
                        etype: "Room",
                        property: "exit=direction:<room>",
                        id: record.qualified_name().into(),
                    })?
                    .trim()
                    .parse::<Title>()
                    .map_err(|source| error::ConversionFailed {
                        etype: "Room",
                        property: "exit=direction:<room>",
                        source,
                    })?;
                Ok((direction, room))
            })
            .collect::<Result<IndexMap<Identifier, Title>, error::Application>>()?;
        let characters = record
            .get_list_parsed("characters")
            .collect::<Result<Vec<_>, error::Application>>()?;
        let actions = record
            .get_list_parsed("actions")
            .collect::<Result<Vec<Identifier>, _>>()?;
        let name = record.parse_name::<Title>()?;
        list.push(RoomRaw {
            name,
            variant: record.variant().clone(),
            description,
            characters,
            exits,
            actions,
        });
    }
    Ok(list)
}
