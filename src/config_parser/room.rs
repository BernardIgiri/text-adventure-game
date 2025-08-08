use indexmap::IndexMap;
use ini::SectionIter;

use crate::{
    core::{Identifier, RoomRaw, Title},
    error,
};

use super::iter::{EntitySection, IterRequireWith, ParseWith, SectionRecordIter};

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
                    .require_next(&record, "exit=<direction>")?
                    .trim()
                    .parse_with::<Identifier>(&record, "exit=<direction>")?;
                let room = parts
                    .require_next(&record, "exit=direction:<room>")?
                    .trim()
                    .parse_with::<Title>(&record, "exit=direction:<room>")?;
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
