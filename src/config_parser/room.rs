use std::rc::Rc;

use indexmap::IndexMap;
use ini::SectionIter;

use crate::{
    core::{CharacterEntity, Identifier, RoomEntity, Title},
    error,
};

use super::{
    iter::{EntitySection, SectionRecordIter},
    types::{CharacterMap, RoomMap},
};

pub fn parse_rooms<'a>(
    ini_iter: SectionIter<'a>,
    character_map: &CharacterMap,
) -> Result<RoomMap, error::Application> {
    let mut map = RoomMap::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Room) {
        let record = record?.into_record(&["description"], &["characters", "exits", "actions"])?;
        let description = record.require("description")?;
        let exits = record
            .get_list("exits")
            .map(|exit| {
                let mut parts = exit.split(":");
                let direction = parts
                    .next()
                    .ok_or_else(|| error::PropertyNotFound {
                        entity: "Room",
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
                        entity: "Room",
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
            .get_list("characters")
            .map(|character_name| {
                Ok(character_map
                    .get(
                        &character_name
                            .parse()
                            .map_err(|source| error::ConversionFailed {
                                etype: "Room",
                                property: "characters",
                                source,
                            })?,
                    )
                    .ok_or_else(|| error::EntityNotFound {
                        etype: "Character",
                        id: character_name.into(),
                    })?
                    .clone())
            })
            .collect::<Result<Vec<Rc<CharacterEntity>>, error::Application>>()?;
        let actions = record
            .get_list("actions")
            .map(|s| s.parse::<Identifier>())
            .collect::<Result<Vec<Identifier>, _>>()
            .map_err(|source| error::ConversionFailed {
                etype: "Room",
                property: "actions",
                source,
            })?;
        let name = record.parse_name::<Title>()?;
        let room = Rc::new(
            RoomEntity::builder()
                .name(name.clone())
                .maybe_variant(record.variant().clone())
                .description(description.to_owned())
                .exits(exits)
                .actions(actions)
                .characters(characters)
                .build(),
        );
        map.entry(name)
            .or_default()
            .insert(record.variant().clone(), room);
    }
    Ok(map)
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use asserting::prelude::*;
    use ini::Ini;

    use crate::config_parser::test_utils::{data::character_map, t};

    use super::parse_rooms;

    const GOOD_DATA: &str = r"
        [Room:Study]
        description=Shelves full of books line the wall. A nice writing desk with a lamp occupies the south corner.
        exits=east:DiningRoom,west:Patio
        characters=CuriousCalvin

        [Room:DiningRoom]
        description=The aroma of freshly baked bread and mom's spicy fried chicken fills the air.
        exits=west:Study

        [Room:Patio]
        description=A beautiful sun lit, picket fenced, lawn, sprawls out to a lovely neighborhood wi- a bird just pooped on you!
        exits=east:Study,south:SinkHole
        characters=NeighborFrank,BlueBird

        [Room:SinkHole]
        description=On second thought, maybe you have stayed inside? You cannot escape!
    ";
    const BAD_DATA: &str = r"
        [Room:Study]
        description=Shelves full of books line the wall. A nice writing desk with a lamp occupies the south corner.
        exits=east:DiningRoom,west:Patio
        characters=Waldo

        [Room:DiningRoom]
        description=The aroma of freshly baked bread and mom's spicy fried chicken fills the air.
        exits=west:Study

        [Room:Patio]
        description=A beautiful sun lit, picket fenced, lawn, sprawls out to a lovely neighborhood wi- a bird just pooped on you!
        exits=east:Study
        characters=NeighborFrank,BlueBird
    ";

    #[test]
    fn good_data() {
        let ini = Ini::load_from_str(GOOD_DATA).unwrap();
        let characters = character_map();
        let rooms = parse_rooms(ini.iter(), &characters).unwrap();
        assert_that!(&rooms)
            .has_length(4)
            .contains_key(t("DiningRoom"))
            .contains_key(t("Study"))
            .contains_key(t("Patio"));
    }

    #[test]
    fn bad_data() {
        let ini = Ini::load_from_str(BAD_DATA).unwrap();
        let characters = character_map();
        let rooms = parse_rooms(ini.iter(), &characters);
        assert_that!(rooms)
            .is_err()
            .extracting(|e| e.err().unwrap().to_string())
            .contains("not find")
            .contains("Waldo")
            .contains("Character");
    }
}
