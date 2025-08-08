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
    use ini::Ini;

    use crate::config_parser::test_utils::i;

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
    fn parse_items_good_data() {
        let ini = Ini::load_from_str(GOOD_DATA).unwrap();
        let items = parse_items(ini.iter()).unwrap();
        assert_eq!(items.len(), 4);
        assert_that!(items).contains_exactly_in_any_order([
            Item::new(i("gold_watch"), "Look how it dazzles in the light!".into()),
            Item::new(
                i("royal_robe"),
                "Such vibrant hues of purple, red, and gold!".into(),
            ),
            Item::new(
                i("rusty_knife"),
                "Dull and twisty, but quite useful.".into(),
            ),
            Item::new(i("potato_sack"), "This will do...".into()),
        ]);
    }

    #[test]
    fn parse_item_bad_data() {
        let ini = Ini::load_from_str(BAD_DATA).unwrap();
        let items = parse_items(ini.iter());
        assert!(items.is_err());
        assert_that!(items.err().unwrap().to_string().as_str())
            .contains("gold_watch")
            .contains("description")
            .contains("Item");
    }
}
