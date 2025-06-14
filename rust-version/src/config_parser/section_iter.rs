use ini::{Properties, SectionIter};
use std::str::FromStr;
use strum::{EnumIter, EnumString, IntoStaticStr};

use crate::{error, world::Identifier};

#[derive(IntoStaticStr, EnumString, EnumIter, Hash, Debug, PartialEq, Eq, Clone, Copy)]
#[strum(serialize_all = "PascalCase")]
pub enum EntitySection {
    Action,
    Character,
    Dialogue,
    Item,
    Response,
    Room,
}

#[derive(Debug)]
pub struct StagedEntity<'a> {
    pub section: &'a str,
    pub name: &'a str,
    pub variant: Option<Identifier>,
    pub properties: &'a Properties,
    pub qualified_name: &'a str,
}

pub struct SectionRecordIter<'a>(SectionIter<'a>, &'a str);

impl<'a> SectionRecordIter<'a> {
    pub const fn new(section_iter: SectionIter<'a>, section: &'a str) -> Self {
        Self(section_iter, section)
    }
}

impl<'a> Iterator for SectionRecordIter<'a> {
    type Item = Result<StagedEntity<'a>, error::Application>;

    fn next(&mut self) -> Option<Self::Item> {
        for (input_opt, properties) in &mut self.0 {
            let input = match input_opt {
                Some(i) => i,
                None => continue,
            };
            match get_record(input, properties) {
                Ok(r) if r.section.trim().eq_ignore_ascii_case(self.1.trim()) => {
                    return Some(Ok(r));
                }
                Err(e) => return Some(Err(e)),
                _ => continue,
            }
        }
        None
    }
}

fn get_record<'a>(
    input: &'a str,
    properties: &'a Properties,
) -> Result<StagedEntity<'a>, error::Application> {
    let mut section_parts = input.split(':');
    let section = section_parts
        .next()
        .ok_or_else(|| error::InvalidPropertyValue {
            value: input.into(),
            field: "Section Name",
        })?;
    let qualified_name = section_parts
        .next()
        .ok_or_else(|| error::InvalidPropertyValue {
            value: input.into(),
            field: "Qualified Entitity Name",
        })?;
    let mut entity_name_parts = qualified_name.split('|');
    let name = entity_name_parts
        .next()
        .ok_or_else(|| error::InvalidPropertyValue {
            value: input.into(),
            field: "Entity Name",
        })?;
    let variant = entity_name_parts
        .next()
        .map(str::parse::<Identifier>)
        .transpose()?;
    Ok(StagedEntity {
        section,
        name,
        variant,
        properties,
        qualified_name,
    })
}

pub trait RequireProperty {
    fn require(
        &self,
        prop: &'static str,
        staged: &StagedEntity,
    ) -> Result<&str, error::Application>;
}

impl RequireProperty for Properties {
    fn require(
        &self,
        prop: &'static str,
        staged: &StagedEntity,
    ) -> Result<&str, error::Application> {
        self.get(prop).ok_or_else(|| error::PropertyNotFound {
            // It should be impossible to get here without a valid staged.section
            #[allow(clippy::unwrap_used)]
            entity: EntitySection::from_str(staged.section).unwrap().into(),
            property: prop,
            id: staged.qualified_name.into(),
        })
    }
}
