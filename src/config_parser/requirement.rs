use std::rc::Rc;

use crate::{
    core::{Identifier, Requirement},
    error,
};

use super::{
    iter::{title_variant_from_qualified, ListProperty, Record},
    types::{ItemMap, RoomMap, RoomVariant},
};

pub fn parse_requirements(
    record: &Record,
    entity_type: &'static str,
    item_map: &ItemMap,
    room_map: &RoomMap,
) -> Result<Vec<Rc<Requirement>>, error::Application> {
    record
        .properties
        .get_list("requires")
        .map(|s| parse_one_requirement(record, entity_type, item_map, room_map, s))
        .collect()
}

fn parse_one_requirement(
    record: &Record,
    entity_type: &'static str,
    item_map: &ItemMap,
    room_map: &RoomMap,
    string: &str,
) -> Result<Rc<Requirement>, error::Application> {
    let mut parts = string.splitn(2, ':').map(str::trim);
    let r_type = parts
        .next()
        .ok_or_else(|| error::PropertyNotFound {
            entity: entity_type,
            property: "requires:<requirement_type>",
            id: record.qualified_name.into(),
        })?
        .to_lowercase();
    let requirement = match r_type.as_str() {
        "has_item" => {
            let item_name = parts.next().ok_or_else(|| error::PropertyNotFound {
                entity: entity_type,
                property: "requires:has_item:<item_id>",
                id: record.qualified_name.into(),
            })?;
            let item_name: Identifier = item_name.parse()?;
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
                entity: entity_type,
                property: "requires:does_not_have:<item_id>",
                id: record.qualified_name.into(),
            })?;
            let item_name: Identifier = item_name.parse()?;
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
                entity: entity_type,
                property: "requires:room_variant:<room>",
                id: record.qualified_name.into(),
            })?;
            let (room_name, variant) = title_variant_from_qualified(qualified_name)?;
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
    Ok(Rc::new(requirement))
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {

    use ini::Properties;

    use crate::{
        config_parser::test_utils::{
            data::{item_map, room_map},
            i, t,
        },
        core::Requirement,
    };

    use super::*;
    use asserting::prelude::*;

    #[test]
    fn test_parse_has_item_requirement() {
        let items = item_map();
        let rooms = room_map(&items, true);

        let mut props = Properties::new();
        props.insert("requires".to_string(), "has_item:key".to_string());

        let record = Record {
            section: "TestSection",
            name: "test_name",
            variant: None,
            qualified_name: "TestSection:test_name",
            properties: &props,
        };

        let result = parse_requirements(&record, "Test", &items, &rooms).unwrap();
        assert_that!(&result).has_length(1);

        match &*result[0] {
            Requirement::HasItem(item) => {
                assert_eq!(item.name(), &i("key"));
            }
            _ => panic!("Expected HasItem requirement"),
        }
    }

    #[test]
    fn test_parse_does_not_have_requirement() {
        let items = item_map();
        let rooms = room_map(&items, true);

        let mut props = Properties::new();
        props.insert("requires".to_string(), "does_not_have:key".to_string());

        let record = Record {
            section: "TestSection",
            name: "test_name",
            variant: None,
            qualified_name: "TestSection:test_name",
            properties: &props,
        };

        let result = parse_requirements(&record, "Test", &items, &rooms).unwrap();
        assert_that!(&result).has_length(1);

        match &*result[0] {
            Requirement::DoesNotHave(item) => {
                assert_eq!(item.name(), &i("key"));
            }
            _ => panic!("Expected HasItem requirement"),
        }
    }

    #[test]
    fn test_parse_room_variant_requirement() {
        let items = item_map();
        let rooms = room_map(&items, true);

        let mut props = Properties::new();
        props.insert(
            "requires".to_string(),
            "room_variant:WoodShed|closed".to_string(),
        );

        let record = Record {
            section: "TestSection",
            name: "test_name",
            variant: None,
            qualified_name: "TestSection:test_name",
            properties: &props,
        };

        let result = parse_requirements(&record, "Test", &items, &rooms).unwrap();
        assert_that!(&result).has_length(1);

        match &*result[0] {
            Requirement::RoomVariant(room) => {
                assert_eq!(room.name(), &t("WoodShed"));
                assert_eq!(room.variant(), &Some(i("closed")));
            }
            _ => panic!("Expected RoomVariant requirement"),
        }
    }

    #[test]
    fn test_parse_room_variant_default_requirement() {
        let items = item_map();
        let rooms = room_map(&items, true);

        let mut props = Properties::new();
        props.insert("requires".to_string(), "room_variant:WoodShed".to_string());

        let record = Record {
            section: "TestSection",
            name: "test_name",
            variant: None,
            qualified_name: "TestSection:test_name",
            properties: &props,
        };

        let result = parse_requirements(&record, "Test", &items, &rooms).unwrap();
        assert_that!(&result).has_length(1);

        match &*result[0] {
            Requirement::RoomVariant(room) => {
                assert_eq!(room.name(), &t("WoodShed"));
                assert_eq!(room.variant(), &None);
            }
            _ => panic!("Expected RoomVariant requirement"),
        }
    }

    #[test]
    fn test_parse_invalid_requirement_type() {
        let items = item_map();
        let rooms = room_map(&items, true);

        let mut props = Properties::new();
        props.insert("requires".to_string(), "nonsense:value".to_string());

        let record = Record {
            section: "TestSection",
            name: "test_name",
            variant: None,
            qualified_name: "TestSection:test_name",
            properties: &props,
        };

        let err = parse_requirements(&record, "Test", &items, &rooms).unwrap_err();
        assert_that!(err.to_string()).contains("requirement");
    }
}
