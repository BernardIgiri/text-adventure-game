use std::rc::Rc;

use ini::SectionIter;

use crate::{
    config_parser::{
        iter::{EntitySection, RecordProperty, SectionRecordIter},
        requirement::parse_requirements,
    },
    core::{Identifier, Response},
    error,
};

use super::types::{ActionMap, ItemMap, ResponseMap, RoomMap};

pub fn parse_responses<'a>(
    ini_iter: SectionIter<'a>,
    action_map: &ActionMap,
    item_map: &ItemMap,
    room_map: &RoomMap,
) -> Result<ResponseMap, error::Application> {
    let mut map = ResponseMap::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Response.into()) {
        let record = record?;
        record
            .properties
            .expect_keys(&["text"], &["leads_to", "triggers", "requires"], &record)?;
        let text = record.properties.require("text", &record)?;
        let leads_to = match record.properties.get("leads_to") {
            Some(s) => Some(s.parse().map_err(|source| error::ConversionFailed {
                etype: "Response",
                property: "leads_to",
                source,
            })?),
            None => None,
        };
        let action = match record.properties.get("triggers") {
            Some(action_name) => Some(
                action_map
                    .get(&action_name.parse::<Identifier>().map_err(|source| {
                        error::ConversionFailed {
                            etype: "Response",
                            property: "triggers",
                            source,
                        }
                    })?)
                    .ok_or_else(|| error::EntityNotFound {
                        etype: "Action",
                        id: action_name.into(),
                    })?
                    .clone(),
            ),
            None => None,
        };
        let requires = parse_requirements(&record, "Response", item_map, room_map)?;
        let response = Rc::new(
            Response::builder()
                .text(text.into())
                .maybe_leads_to(leads_to)
                .maybe_triggers(action)
                .requires(requires)
                .build(),
        );
        map.insert(
            record
                .name
                .parse()
                .map_err(|source| error::ConversionFailed {
                    etype: "Response",
                    property: "name",
                    source,
                })?,
            response,
        );
    }
    Ok(map)
}
