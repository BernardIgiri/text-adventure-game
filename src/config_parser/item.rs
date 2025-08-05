use ini::SectionIter;

use crate::{
    core::{Identifier, Item},
    error,
};

use super::iter::{EntitySection, SectionRecordIter};

pub fn parse_items<'a>(ini_iter: SectionIter<'a>) -> Result<Vec<Item>, error::Application> {
    let mut list = Vec::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Item) {
        let record = record?.into_record(&["description"], &[])?;
        let description = record.require("description")?.to_string();
        let name = record.parse_name::<Identifier>()?;
        list.push(Item { name, description });
    }
    Ok(list)
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    // TODO fix tests
    // use ini::Ini;

    // use super::*;
    // use asserting::prelude::*;

    // const GOOD_DATA: &str = r"
    //             [Item:gold_watch]
    //             description=Look how it dazzles in the light!

    //             [Item:royal_robe]
    //             description=Such vibrant hues of purple, red, and gold!

    //             [Item:rusty_knife]
    //             description=Dull and twisty, but quite useful.

    //             [Item:potato_sack]
    //             description=This will do...
    //         ";
    // const BAD_DATA: &str = r"
    //             [Item:gold_watch]

    //             [Item:royal_robe]
    //             description=Such vibrant hues of purple, red, and gold!

    //             [Item:rusty_knife]
    //             description=Dull and twisty, but quite useful.

    //             [Item:potato_sack]
    //             description=This will do...
    //         ";

    // #[test]
    // fn parse_items_good_data() {
    //     let ini = Ini::load_from_str(GOOD_DATA).unwrap();
    //     let items = parse_items(ini.iter()).unwrap();
    //     assert_eq!(items.len(), 4);
    //     assert_eq!(
    //         items
    //             .get(&"gold_watch".parse().unwrap())
    //             .unwrap()
    //             .name()
    //             .to_string(),
    //         "gold_watch".to_string()
    //     );
    //     assert_eq!(
    //         items
    //             .get(&"gold_watch".parse().unwrap())
    //             .unwrap()
    //             .description(),
    //         &"Look how it dazzles in the light!".to_string()
    //     );
    //     assert_eq!(
    //         items
    //             .get(&"potato_sack".parse().unwrap())
    //             .unwrap()
    //             .description(),
    //         &"This will do...".to_string()
    //     );
    //     assert!(items.contains_key(&"royal_robe".parse().unwrap()));
    //     assert!(items.contains_key(&"rusty_knife".parse().unwrap()));
    //     assert!(items.contains_key(&"potato_sack".parse().unwrap()));
    // }

    // #[test]
    // fn parse_item_bad_data() {
    //     let ini = Ini::load_from_str(BAD_DATA).unwrap();
    //     let items = parse_items(ini.iter());
    //     assert!(items.is_err());
    //     assert_that!(items.err().unwrap().to_string().as_str())
    //         .contains("gold_watch")
    //         .contains("description")
    //         .contains("Item");
    // }
}
