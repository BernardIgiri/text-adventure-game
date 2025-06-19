use std::rc::Rc;

use ini::SectionIter;

use crate::{
    core::{Identifier, Item},
    error,
};

use super::{
    iter::{EntitySection, RecordProperty, SectionRecordIter},
    types::ItemMap,
};

pub fn parse_items<'a>(ini_iter: SectionIter<'a>) -> Result<ItemMap, error::Application> {
    let mut map = ItemMap::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Item.into()) {
        let record = record?;
        record
            .properties
            .expect_keys(&["description"], &[], &record)?;
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

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use ini::Ini;

    use super::*;
    use asserting::prelude::*;

    const GOOD_DATA: &str = r"
                [Item:gold_watch]
                description=Look how it dazzles in the light!
            
                [Item:royal_robe]
                description=Such vibrant hues of purple, red, and gold!

                [Item:rusty_knife]
                description=Dull and twisty, but quite useful.

                [Item:potato_sack]
                description=This will do...
            ";
    const BAD_DATA: &str = r"
                [Item:gold_watch]
            
                [Item:royal_robe]
                description=Such vibrant hues of purple, red, and gold!

                [Item:rusty_knife]
                description=Dull and twisty, but quite useful.

                [Item:potato_sack]
                description=This will do...
            ";

    #[test]
    fn test_parse_items() {
        let ini = Ini::load_from_str(GOOD_DATA).unwrap();
        let items = parse_items(ini.iter()).unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(
            items
                .get(&"gold_watch".parse().unwrap())
                .unwrap()
                .name()
                .to_string(),
            "gold_watch".to_string()
        );
        assert_eq!(
            items
                .get(&"gold_watch".parse().unwrap())
                .unwrap()
                .description(),
            &"Look how it dazzles in the light!".to_string()
        );
        assert_eq!(
            items
                .get(&"potato_sack".parse().unwrap())
                .unwrap()
                .description(),
            &"This will do...".to_string()
        );
        assert!(items.contains_key(&"royal_robe".parse().unwrap()));
        assert!(items.contains_key(&"rusty_knife".parse().unwrap()));
        assert!(items.contains_key(&"potato_sack".parse().unwrap()));
    }

    #[test]
    fn test_parse_bad_data() {
        let ini = Ini::load_from_str(BAD_DATA).unwrap();
        let items = parse_items(ini.iter());
        assert!(items.is_err());
        assert_that!(items.err().unwrap().to_string().as_str())
            .contains("gold_watch")
            .contains("description")
            .contains("Item");
    }
}
