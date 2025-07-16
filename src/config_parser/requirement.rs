use crate::{
    core::{Identifier, Requirement},
    error,
};

use super::{
    iter::Record,
    types::{ItemMap, RoomMap, RoomVariant},
};

pub fn parse_requirements(
    record: &Record,
    item_map: &ItemMap,
    room_map: &RoomMap,
) -> Result<Vec<Requirement>, error::Application> {
    record
        .get_list("requires")
        .map(|s| parse_one_requirement(record, item_map, room_map, s))
        .collect()
}

fn parse_one_requirement(
    record: &Record,
    item_map: &ItemMap,
    room_map: &RoomMap,
    string: &str,
) -> Result<Requirement, error::Application> {
    let mut parts = string.splitn(2, ':').map(str::trim);
    let r_type = parts
        .next()
        .ok_or_else(|| error::PropertyNotFound {
            entity: record.entity_type(),
            property: "requires:<requirement_type>",
            id: record.qualified_name().into(),
        })?
        .to_lowercase();
    let requirement = match r_type.as_str() {
        "has_item" => {
            let item_name = parts.next().ok_or_else(|| error::PropertyNotFound {
                entity: record.entity_type(),
                property: "requires:has_item:<item_id>",
                id: record.qualified_name().into(),
            })?;
            let item_name: Identifier =
                item_name
                    .parse()
                    .map_err(|source| error::ConversionFailed {
                        etype: record.entity_type(),
                        property: "requires:has_item:<item_id>",
                        source,
                    })?;
            let item = item_map
                .get(&item_name)
                .cloned()
                .ok_or_else(|| error::EntityNotFound {
                    etype: "Item",
                    id: item_name.to_string(),
                })?;

            Requirement::HasItem(item)
        }
        "does_not_have" => {
            let item_name = parts.next().ok_or_else(|| error::PropertyNotFound {
                entity: record.entity_type(),
                property: "requires:does_not_have:<item_id>",
                id: record.qualified_name().into(),
            })?;
            let item_name: Identifier =
                item_name
                    .parse()
                    .map_err(|source| error::ConversionFailed {
                        etype: record.entity_type(),
                        property: "requires:does_not_have:<item_id>",
                        source,
                    })?;
            let item = item_map
                .get(&item_name)
                .cloned()
                .ok_or_else(|| error::EntityNotFound {
                    etype: "Item",
                    id: item_name.to_string(),
                })?;

            Requirement::DoesNotHave(item)
        }
        "room_variant" => {
            let qualified_name = parts.next().ok_or_else(|| error::PropertyNotFound {
                entity: record.entity_type(),
                property: "requires:room_variant:<room>",
                id: record.qualified_name().into(),
            })?;
            let (room_name, variant) = record.parse_qualified_name(qualified_name)?;
            let room =
                room_map
                    .get_room(&room_name, &variant)
                    .ok_or_else(|| error::EntityNotFound {
                        etype: "Room",
                        id: qualified_name.into(),
                    })?;

            Requirement::RoomVariant(room)
        }
        _ => {
            return Err(error::InvalidPropertyValue {
                value: string.into(),
                field: "requirement",
            });
        }
    };
    Ok(requirement)
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {

    use ini::Ini;

    use crate::{
        config_parser::{
            iter::{EntitySection, SectionRecordIter},
            test_utils::{
                data::{TakeCloneVariant, item_map, room_map},
                i,
            },
        },
        core::Requirement,
    };

    use super::*;
    use asserting::prelude::*;

    fn make_record<'a>(ini: &'a mut Ini, props: &'a [(&'static str, &'static str)]) -> Record<'a> {
        let mut section = ini.with_section::<&str>(Some("Item:test"));
        for (k, v) in props {
            section.set(k.to_string(), v.to_string());
        }
        let mut iter = SectionRecordIter::new(ini.iter(), EntitySection::Item);
        let record = iter
            .next()
            .unwrap()
            .unwrap()
            .into_record(&["requires"], &[]);
        record.unwrap()
    }

    #[test]
    fn parse_has_item_requirement() {
        let items = item_map();
        let rooms = room_map(true);

        let mut ini = Ini::new();
        let record = make_record(&mut ini, &[("requires", "has_item:key")]);

        let result = parse_requirements(&record, &items, &rooms).unwrap();
        assert_that!(&result).has_length(1);

        let r: &Requirement = &result[0];
        match &r {
            Requirement::HasItem(item) => {
                assert_eq!(item.name(), &i("key"));
            }
            _ => panic!("Expected HasItem requirement"),
        }
    }

    #[test]
    fn parse_does_not_have_requirement() {
        let items = item_map();
        let rooms = room_map(true);

        let mut ini = Ini::new();
        let record = make_record(&mut ini, &[("requires", "does_not_have:key")]);

        let result = parse_requirements(&record, &items, &rooms).unwrap();
        assert_that!(&result).has_length(1);

        let r: &Requirement = &result[0];
        match &r {
            Requirement::DoesNotHave(item) => {
                assert_eq!(item.name(), &i("key"));
            }
            _ => panic!("Expected HasItem requirement"),
        }
    }

    #[test]
    fn parse_room_variant_requirement() {
        let items = item_map();
        let rooms = room_map(true);

        let mut ini = Ini::new();
        let record = make_record(&mut ini, &[("requires", "room_variant:WoodShed|closed")]);

        let result = parse_requirements(&record, &items, &rooms).unwrap();
        assert_that!(result).contains_exactly([Requirement::RoomVariant(
            rooms.take_clone("WoodShed", Some("closed")),
        )]);
    }

    #[test]
    fn parse_room_variant_default_requirement() {
        let items = item_map();
        let rooms = room_map(true);

        let mut ini = Ini::new();
        let record = make_record(&mut ini, &[("requires", "room_variant:WoodShed")]);

        let result = parse_requirements(&record, &items, &rooms).unwrap();
        assert_that!(result)
            .contains_exactly([Requirement::RoomVariant(rooms.take_clone("WoodShed", None))]);
    }

    #[test]
    fn parse_invalid_requirement_type() {
        let items = item_map();
        let rooms = room_map(true);

        let mut ini = Ini::new();
        let record = make_record(&mut ini, &[("requires", "nonesense")]);

        let err = parse_requirements(&record, &items, &rooms).unwrap_err();
        assert_that!(err.to_string()).contains("requirement");
    }
}
