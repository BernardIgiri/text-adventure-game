use std::rc::Rc;

use ini::SectionIter;

use crate::{
    error,
    world::{Identifier, Item},
};

use super::{
    section_iter::{EntitySection, SectionRecordIter},
    types::ItemMap,
};

pub fn parse_items<'a>(ini_iter: SectionIter<'a>) -> Result<ItemMap, error::Application> {
    let mut map = ItemMap::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Item.into()) {
        let record = record?;
        let description =
            record
                .properties
                .get("description")
                .ok_or_else(|| error::PropertyNotFound {
                    entity: "Item",
                    property: "description",
                    id: record.qualified_name.into(),
                })?;
        let name = record.name.parse::<Identifier>()?;
        let item = Rc::new(Item::new(name.clone(), description.to_string()));
        map.insert(name, item);
    }
    Ok(map)
}
