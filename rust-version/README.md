 - [ ] Remap depedencies. If all of the potential circular depedencies are broken up, then parsing could be made much simpler.


Item

Room
	- Room (w)
	- Item
	- Action (w)

Action
	- Room
	- Item

Response
	- Dialog (w)
	- Action
	- Requirement
		- Item
		- Room

Dialogue
	- Response
	- Requirement
		- Item
		- Room

Character
	- Room
	- Dialogue

// In absense of Responses there needs to be a generic Goodby response
// In absense of Dialogue there needs to be a generic "..." dialogue
// Add a GameOver action
use ini::Ini;

mod utils;

const GOOD_CHARACTER_DATA: &str = r"
                [Character:OldMan]
                start_dialogue=greeting_old_man

                [Character:Merchant]
                start_dialogue=buy_or_leave

                [Character:Guard]
                start_dialogue=halt_intruder
            ";
const BAD_CHARACTER_DATA: &str = r"
                [Character:OldMan]

                [Character:Merchant]
                start_dialogue=buy_or_leave
            ";

#[test]
fn test_parse_characters_success() {
    let ini = Ini::load_from_str(GOOD_CHARACTER_DATA);
    let characters = parse_characters(ini.iter()).unwrap();
    assert_eq!(characters.len(), 3);

    let c = characters.get(&"OldMan".parse().unwrap()).unwrap();
    assert_eq!(c.name().to_string().as_str(), "Old Man");

    assert!(characters.contains_key(&"Merchant".parse().unwrap()));
    assert!(characters.contains_key(&"Guard".parse().unwrap()));
}

#[test]
fn test_parse_characters_missing_field() {
    let ini = Ini::load_from_str(BAD_CHARACTER_DATA);
    let characters = parse_characters(ini.iter()).unwrap();

    assert!(result.is_err());
    let err = result.err().unwrap().to_string();
    assert_that!(err.as_str())
        .contains("Character")
        .contains("start_dialogue")
        .contains("OldMan");
}
