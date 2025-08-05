use ini::SectionIter;

use crate::{
    core::{CharacterRaw, Title},
    error,
};

use super::iter::{EntitySection, SectionRecordIter};

pub fn parse_characters(ini_iter: SectionIter) -> Result<Vec<CharacterRaw>, error::Application> {
    let mut list = Vec::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Character) {
        let record = record?.into_record(&["start_dialogue"], &[])?;
        let start_dialogue = record.require_parsed("start_dialogue")?;
        let name = record.parse_name::<Title>()?;
        list.push(CharacterRaw {
            name,
            start_dialogue,
        });
    }
    Ok(list)
}
