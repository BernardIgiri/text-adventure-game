use ini::{Ini, SectionIter};

use crate::{
    config_parser::iter::Record,
    core::{GameTitleRaw, Language, Theme},
    error,
};

use super::iter::{EntitySection, SectionRecordIter};

pub fn parse_title(ini: &Ini) -> Result<GameTitleRaw, error::Application> {
    let properties = ini
        .section(None::<String>)
        .ok_or(error::EntitySectionNotFound(""))?;
    let record = Record::from_root(
        properties,
        &["title", "greeting", "credits", "start_room"],
        &[],
    )?;
    let title = record.require("title")?.to_string();
    let greeting = record.require("greeting")?.to_string();
    let credits = record.require("credits")?.to_string();
    let start_room = record.require_parsed("start_room")?;
    Ok(GameTitleRaw {
        title,
        greeting,
        credits,
        start_room,
    })
}

pub fn parse_theme<'a>(ini_iter: SectionIter<'a>) -> Result<Theme, error::Application> {
    let mut iter = SectionRecordIter::new(ini_iter, EntitySection::Theme);
    let record = if let Some(r) = iter.next() {
        r?.into_record(
            &[
                "title",
                "heading",
                "background",
                "text",
                "highlight",
                "highlight_text",
                "subdued",
            ],
            &[],
        )?
    } else {
        return Ok(Theme::default());
    };
    let title = record.require_parsed("title")?;
    let heading = record.require_parsed("heading")?;
    let background = record.require_parsed("background")?;
    let text = record.require_parsed("text")?;
    let highlight = record.require_parsed("highlight")?;
    let highlight_text = record.require_parsed("highlight_text")?;
    let subdued = record.require_parsed("subdued")?;
    Ok(Theme::builder()
        .title(title)
        .heading(heading)
        .background(background)
        .text(text)
        .highlight(highlight)
        .highlight_text(highlight_text)
        .subdued(subdued)
        .build())
}

pub fn parse_language<'a>(ini_iter: SectionIter<'a>) -> Result<Language, error::Application> {
    let mut iter = SectionRecordIter::new(ini_iter, EntitySection::Language);
    let record = if let Some(r) = iter.next() {
        r?.into_record(
            &[
                "characters_found",
                "exits_found",
                "talk",
                "interact",
                "view_inventory",
                "inventory",
                "go_somewhere",
                "end_game",
                "choose_exit",
                "cancel_exit",
                "choose_chat",
                "cancel_chat",
                "choose_response",
                "cancel_response",
                "choose_action",
                "cancel_action",
                "action_failed",
                "continue_game",
                "press_q_to_quit",
            ],
            &[],
        )?
    } else {
        return Ok(Language::default());
    };
    let characters_found = record.require("characters_found")?.into();
    let exits_found = record.require("exits_found")?.into();
    let end_game = record.require("end_game")?.into();
    let go_somewhere = record.require("go_somewhere")?.into();
    let talk = record.require("talk")?.into();
    let interact = record.require("interact")?.into();
    let view_inventory = record.require("view_inventory")?.into();
    let inventory = record.require("inventory")?.into();
    let choose_exit = record.require("choose_exit")?.into();
    let cancel_exit = record.require("cancel_exit")?.into();
    let choose_chat = record.require("choose_chat")?.into();
    let cancel_chat = record.require("cancel_chat")?.into();
    let choose_response = record.require("choose_response")?.into();
    let cancel_response = record.require("cancel_response")?.into();
    let choose_action = record.require("choose_action")?.into();
    let cancel_action = record.require("cancel_action")?.into();
    let action_failed = record.require("action_failed")?.into();
    let continue_game = record.require("continue_game")?.into();
    let press_q_to_quit = record.require("press_q_to_quit")?.into();
    Ok(Language::builder()
        .characters_found(characters_found)
        .exits_found(exits_found)
        .end_game(end_game)
        .go_somewhere(go_somewhere)
        .talk(talk)
        .interact(interact)
        .view_inventory(view_inventory)
        .inventory(inventory)
        .choose_exit(choose_exit)
        .cancel_exit(cancel_exit)
        .choose_chat(choose_chat)
        .cancel_chat(cancel_chat)
        .choose_response(choose_response)
        .cancel_response(cancel_response)
        .choose_action(choose_action)
        .cancel_action(cancel_action)
        .action_failed(action_failed)
        .continue_game(continue_game)
        .press_q_to_quit(press_q_to_quit)
        .build())
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use asserting::{assert_that, prelude::AssertResult};
    use ini::Ini;

    use crate::config_parser::test_utils::t;

    use super::*;

    const GOOD_THEME_DATA: &str = r"
        [Theme]
        title = Red
        heading = Green
        background = #FF3344
        text = #FFFFFF
        highlight = RGB(10, 50, 10)
        highlight_text = Blue
        subdued = Gray
    ";
    const GOOD_LANGUAGE_DATA: &str = r"
        [Language]
        characters_found = You look around and see:
        exits_found = You can exit in these directions:
        talk = Talk
        interact = Interact
        go_somewhere = Go someplace?
        view_inventory = Look through your stuff?
        inventory = Your Bag Of Stuff
        end_game = End the game?
        choose_exit = Get out of here?
        cancel_exit = Don't leave
        choose_chat = Talk to:
        cancel_chat = Nevermind
        choose_response = You say:
        cancel_response = ...
        choose_action = You decide to:
        cancel_action = Nevermind
        action_failed = That didn't work
        continue_game = Keep Going?
        press_q_to_quit = The letter q is for quit!
    ";
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
        assert_eq!(title.title, "The Beach Trip".to_string());
        assert_eq!(title.greeting, "Welcome to the beach bro!".to_string());
        assert_eq!(title.credits, "Special thanks to my mom!".to_string());
        assert_eq!(title.start_room, t("TheCar"));
    }

    #[test]
    fn missing_start_room() {
        let ini = Ini::load_from_str(BAD_DATA_NO_START).unwrap();
        let result = parse_title(&ini);
        assert_that!(result)
            .is_err()
            .satisfies(|r| matches!(r, Err(error::MissingProperties { .. })));
    }

    #[test]
    fn empty_data() {
        let ini = Ini::load_from_str(BAD_DATA_EMPTY).unwrap();
        let result = parse_title(&ini);
        assert_that!(result)
            .is_err()
            .satisfies(|r| matches!(r, Err(error::MissingProperties { .. })));
    }

    #[test]
    fn bad_start_room_name() {
        let ini = Ini::load_from_str(BAD_DATA_BAD_START).unwrap();
        let result = parse_title(&ini);
        assert_that!(result)
            .is_err()
            .satisfies(|r| matches!(r, Err(error::ConversionFailed { .. })));
    }

    #[test]
    fn theme_good_data() {
        let ini = Ini::load_from_str(GOOD_THEME_DATA).unwrap();
        let theme = parse_theme(ini.iter()).unwrap();
        assert_eq!(theme.title().r(), &255u8);
        assert_eq!(theme.title().g(), &0u8);
        assert_eq!(theme.title().b(), &0u8);
    }

    #[test]
    fn language_good_data() {
        let ini = Ini::load_from_str(GOOD_LANGUAGE_DATA).unwrap();
        let language = parse_language(ini.iter()).unwrap();
        assert_eq!(language.characters_found(), "You look around and see:");
        assert_eq!(language.exits_found(), "You can exit in these directions:");
        assert_eq!(language.talk(), "Talk");
        assert_eq!(language.interact(), "Interact");
        assert_eq!(language.go_somewhere(), "Go someplace?");
        assert_eq!(language.end_game(), "End the game?");
        assert_eq!(language.choose_exit(), "Get out of here?");
        assert_eq!(language.cancel_exit(), "Don't leave");
        assert_eq!(language.choose_chat(), "Talk to:");
        assert_eq!(language.cancel_chat(), "Nevermind");
        assert_eq!(language.choose_response(), "You say:");
        assert_eq!(language.cancel_response(), "...");
        assert_eq!(language.choose_action(), "You decide to:");
        assert_eq!(language.cancel_action(), "Nevermind");
        assert_eq!(language.action_failed(), "That didn't work");
        assert_eq!(language.continue_game(), "Keep Going?");
        assert_eq!(language.press_q_to_quit(), "The letter q is for quit!");
    }
}
