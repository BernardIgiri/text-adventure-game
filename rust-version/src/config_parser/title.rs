use ini::Ini;

use crate::{error, world::GameTitle};

pub fn parse_title(ini: &Ini) -> Result<GameTitle, error::Application> {
    let top_level = ini
        .section(None::<String>)
        .ok_or(error::EntitySectionNotFound("Top Level"))?;
    let title = top_level
        .get("title")
        .ok_or_else(|| error::PropertyNotFound {
            entity: "Top Level",
            property: "title",
            id: "n/a".into(),
        })?;
    let start_room = top_level
        .get("start_room")
        .ok_or_else(|| error::PropertyNotFound {
            entity: "Top Level",
            property: "start_room",
            id: "n/a".into(),
        })?;
    Ok(GameTitle::new(title.into(), start_room.parse()?))
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
        start_room = TheCar
    ";
    const BAD_DATA_NO_START: &str = r"
        title = The Beach Trip
    ";
    const BAD_DATA_EMPTY: &str = r"
    ";
    const BAD_DATA_BAD_START: &str = r"
        title = The Beach Trip
        start_room = theCar
    ";

    #[test]
    fn test_good_data() {
        let ini = Ini::load_from_str(GOOD_DATA).unwrap();
        let title = parse_title(&ini).unwrap();
        assert_eq!(title.title(), &"The Beach Trip".to_string());
        assert_eq!(title.start_room(), &t("TheCar"));
    }

    #[test]
    fn test_missing_start_room() {
        let ini = Ini::load_from_str(BAD_DATA_NO_START).unwrap();
        let result = parse_title(&ini);
        assert!(
            matches!(result, Err(error::PropertyNotFound { property, .. }) if property == "start_room")
        );
    }

    #[test]
    fn test_empty_data() {
        let ini = Ini::load_from_str(BAD_DATA_EMPTY).unwrap();
        let result = parse_title(&ini);
        dbg!(&result);
        assert!(matches!(result, Err(error::PropertyNotFound { .. })));
    }

    #[test]
    fn test_bad_start_room_name() {
        let ini = Ini::load_from_str(BAD_DATA_BAD_START).unwrap();
        let result = parse_title(&ini);
        assert!(matches!(result, Err(error::IllegalConversion { .. })));
    }
}
