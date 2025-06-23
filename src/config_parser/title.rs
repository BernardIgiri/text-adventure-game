use ini::Ini;

use crate::{core::GameTitle, error};

pub fn parse_title(ini: &Ini) -> Result<GameTitle, error::Application> {
    let top_level = ini
        .section(None::<String>)
        .ok_or(error::EntitySectionNotFound(""))?;
    let title = top_level
        .get("title")
        .ok_or_else(|| error::PropertyNotFound {
            entity: "",
            property: "title",
            id: "n/a".into(),
        })?;
    let greeting = top_level
        .get("greeting")
        .ok_or_else(|| error::PropertyNotFound {
            entity: "",
            property: "greeting",
            id: "n/a".into(),
        })?;
    let credits = top_level
        .get("credits")
        .ok_or_else(|| error::PropertyNotFound {
            entity: "",
            property: "credits",
            id: "n/a".into(),
        })?;
    let start_room = top_level
        .get("start_room")
        .ok_or_else(|| error::PropertyNotFound {
            entity: "",
            property: "start_room",
            id: "n/a".into(),
        })?;
    Ok(GameTitle::new(
        title.into(),
        greeting.into(),
        credits.into(),
        start_room
            .parse()
            .map_err(|source| error::ConversionFailed {
                etype: "",
                property: "start_room",
                source,
            })?,
    ))
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use ini::Ini;

    use crate::config_parser::test_utils::t;

    use super::*;

    const GOOD_DATA: &str = r"
        title = The Beach Trip
        greeting = Welcome to the beach bro!
        credits = Special thanks to my mom!
        start_room = TheCar
    ";
    const BAD_DATA_NO_START: &str = r"
        title = The Beach Trip
        greeting = Welcome to the beach bro!
        credits = Special thanks to my mom!
    ";
    const BAD_DATA_EMPTY: &str = r"
    ";
    const BAD_DATA_BAD_START: &str = r"
        title = The Beach Trip
        greeting = Welcome to the beach bro!
        credits = Special thanks to my mom!
        start_room = theCar
    ";

    #[test]
    fn good_data() {
        let ini = Ini::load_from_str(GOOD_DATA).unwrap();
        let title = parse_title(&ini).unwrap();
        assert_eq!(title.title(), &"The Beach Trip".to_string());
        assert_eq!(title.greeting(), &"Welcome to the beach bro!".to_string());
        assert_eq!(title.credits(), &"Special thanks to my mom!".to_string());
        assert_eq!(title.start_room(), &t("TheCar"));
    }

    #[test]
    fn missing_start_room() {
        let ini = Ini::load_from_str(BAD_DATA_NO_START).unwrap();
        let result = parse_title(&ini);
        assert!(
            matches!(result, Err(error::PropertyNotFound { property, .. }) if property == "start_room")
        );
    }

    #[test]
    fn empty_data() {
        let ini = Ini::load_from_str(BAD_DATA_EMPTY).unwrap();
        let result = parse_title(&ini);
        assert!(matches!(result, Err(error::PropertyNotFound { .. })));
    }

    #[test]
    fn bad_start_room_name() {
        let ini = Ini::load_from_str(BAD_DATA_BAD_START).unwrap();
        let result = parse_title(&ini);
        assert!(matches!(result, Err(error::ConversionFailed { .. })));
    }
}
