use crate::{
    core::{Identifier, RequirementRaw},
    error,
};

use super::iter::{IterRequireWith, ParseWith, Record};

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
        .require_next(record, "requires:<requirement_type>")?
        .to_lowercase();
    let requirement = match r_type.as_str() {
        "has_item" => {
            let item = parts.require_next(record, "requires:has_item:<item_id>")?;
            let item: Identifier = item.parse_with(record, "requires:has_item:<item_id>")?;
            RequirementRaw::HasItem(item)
        }
        "does_not_have" => {
            let item = parts.require_next(record, "requires:does_not_have:<item_id>")?;
            let item: Identifier = item.parse_with(record, "requires:does_not_have:<item_id>")?;
            RequirementRaw::DoesNotHave(item)
        }
        "room_variant" => {
            let qualified_name = parts.require_next(record, "requires:room_variant:<room>")?;
            let (room, variant) = record.parse_qualified_name(qualified_name)?;
            RequirementRaw::RoomVariant(room, variant)
        }
        _ => {
            return Err(error::InvalidPropertyValue {
                etype: record.entity_type().into(),
                value: string.into(),
                field: "requirement".into(),
            });
        }
    };
    Ok(requirement)
}
