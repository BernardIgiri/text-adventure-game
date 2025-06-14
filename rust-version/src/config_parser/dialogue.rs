use std::rc::Rc;

use ini::SectionIter;

use crate::{
    error,
    world::{Dialogue, Response},
};

use super::{
    iter::{EntitySection, ListProperty, RequireProperty, SectionRecordIter},
    requirement::parse_requirements,
    types::{DialogueMap, ItemMap, ResponseMap, RoomMap},
};

pub fn parse_dialogues(
    ini_iter: SectionIter,
    response_map: &ResponseMap,
    item_map: &ItemMap,
    room_map: &RoomMap,
) -> Result<DialogueMap, error::Application> {
    let mut map = DialogueMap::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Dialogue.into()) {
        let record = record?;
        let text = record.properties.require("text", &record)?;
        let responses = record
            .properties
            .get_list("response")
            .map(|s| {
                Ok(response_map
                    .get(&s.parse()?)
                    .ok_or_else(|| error::EntityNotFound {
                        etype: "Response",
                        id: s.into(),
                    })?
                    .clone())
            })
            .collect::<Result<Vec<Rc<Response>>, error::Application>>()?;
        let requires = parse_requirements(&record, "Response", item_map, room_map)?;
        let dialogue = Rc::new(
            Dialogue::builder()
                .text(text.into())
                .responses(responses)
                .requires(requires)
                .build(),
        );
        map.entry(record.name.parse()?)
            .or_default()
            .insert(record.variant.clone(), dialogue);
    }
    Ok(map)
}
