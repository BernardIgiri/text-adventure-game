use crate::{
    core::{Identifier, RequirementRaw},
    error,
};

use super::iter::Record;

pub fn parse_requirements(record: &Record) -> Result<Vec<RequirementRaw>, error::Application> {
    record
        .get_list("requires")
        .map(|s| parse_one_requirement(record, s))
        .collect()
}

fn parse_one_requirement(
    record: &Record,
    string: &str,
) -> Result<RequirementRaw, error::Application> {
    let mut parts = string.splitn(2, ':').map(str::trim);
    let r_type = parts
        .next()
        .ok_or_else(|| error::PropertyNotFound {
            etype: record.entity_type(),
            property: "requires:<requirement_type>",
            id: record.qualified_name().into(),
        })?
        .to_lowercase();
    let requirement = match r_type.as_str() {
        "has_item" => {
            let item = parts.next().ok_or_else(|| error::PropertyNotFound {
                etype: record.entity_type(),
                property: "requires:has_item:<item_id>",
                id: record.qualified_name().into(),
            })?;
            let item: Identifier = item.parse().map_err(|source| error::ConversionFailed {
                etype: record.entity_type(),
                property: "requires:has_item:<item_id>",
                source,
            })?;

            RequirementRaw::HasItem(item)
        }
        "does_not_have" => {
            let item = parts.next().ok_or_else(|| error::PropertyNotFound {
                etype: record.entity_type(),
                property: "requires:does_not_have:<item_id>",
                id: record.qualified_name().into(),
            })?;
            let item: Identifier = item.parse().map_err(|source| error::ConversionFailed {
                etype: record.entity_type(),
                property: "requires:does_not_have:<item_id>",
                source,
            })?;

            RequirementRaw::DoesNotHave(item)
        }
        "room_variant" => {
            let qualified_name = parts.next().ok_or_else(|| error::PropertyNotFound {
                etype: record.entity_type(),
                property: "requires:room_variant:<room>",
                id: record.qualified_name().into(),
            })?;
            let (room, variant) = record.parse_qualified_name(qualified_name)?;

            RequirementRaw::RoomVariant(room, variant)
        }
        _ => {
            return Err(error::InvalidPropertyValue {
                etype: record.entity_type(),
                value: string.into(),
                field: "requirement",
            });
        }
    };
    Ok(requirement)
}
