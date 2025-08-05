use ini::SectionIter;

use crate::{
    config_parser::{
        iter::{EntitySection, SectionRecordIter},
        requirement::parse_requirements,
    },
    core::ResponseRaw,
    error,
};

pub fn parse_responses<'a>(
    ini_iter: SectionIter<'a>,
) -> Result<Vec<ResponseRaw>, error::Application> {
    let mut list = Vec::new();
    for record in SectionRecordIter::new(ini_iter, EntitySection::Response) {
        let record = record?.into_record(&["text"], &["leads_to", "triggers", "requires"])?;
        let text = record.require("text")?.to_string();
        let leads_to = record.get_parsed("leads_to")?;
        let triggers = record.get_parsed("triggers")?;
        let requires = parse_requirements(&record)?;
        let name = record.parse_name()?;
        list.push(ResponseRaw {
            name,
            text,
            leads_to,
            triggers,
            requires,
        });
    }
    Ok(list)
}
