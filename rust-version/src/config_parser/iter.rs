use ini::{Properties, SectionIter};
use std::str::FromStr;
use strum::{EnumIter, EnumString, IntoStaticStr};

use crate::{
    error,
    world::{Identifier, Title},
};

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

pub fn title_variant_from_qualified(
    s: &str,
) -> Result<(Title, Option<Identifier>), error::Application> {
    let (name, variant) = unparsed_variant_from_qualified(s)?;
    Ok((name.parse::<Title>()?, variant))
}

pub fn unparsed_variant_from_qualified(
    s: &str,
) -> Result<(&str, Option<Identifier>), error::Application> {
    let mut parts = s.splitn(2, '|');
    let name = parts
        .next()
        .ok_or_else(|| error::InvalidPropertyValue {
            value: s.into(),
            field: "qualified name",
        })?
        .trim();
    let variant = match parts.next() {
        Some(v) => Some(v.trim().parse::<Identifier>()?),
        None => None,
    };
    Ok((name, variant))
}

#[derive(Debug)]
pub struct Record<'a> {
    pub section: &'a str,
    pub name: &'a str,
    pub variant: Option<Identifier>,
    pub properties: &'a Properties,
    pub qualified_name: &'a str,
}

pub struct SectionRecordIter<'a>(SectionIter<'a>, &'a str);

impl<'a> SectionRecordIter<'a> {
    pub const fn new(iter: SectionIter<'a>, section: &'a str) -> Self {
        Self(iter, section)
    }
}

impl<'a> Iterator for SectionRecordIter<'a> {
    type Item = Result<Record<'a>, error::Application>;

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
) -> Result<Record<'a>, error::Application> {
    let mut section_parts = input.splitn(2, ':');
    let section = section_parts
        .next()
        .ok_or_else(|| error::InvalidPropertyValue {
            value: input.into(),
            field: "Section Name",
        })?
        .trim();
    let qualified_name = section_parts
        .next()
        .ok_or_else(|| error::InvalidPropertyValue {
            value: input.into(),
            field: "Qualified Entitity Name",
        })?
        .trim();
    let (name, variant) = unparsed_variant_from_qualified(qualified_name)?;
    Ok(Record {
        section,
        name,
        variant,
        properties,
        qualified_name,
    })
}

pub trait RequireProperty {
    fn require(&self, prop: &'static str, record: &Record) -> Result<&str, error::Application>;
}

impl RequireProperty for Properties {
    fn require(&self, prop: &'static str, record: &Record) -> Result<&str, error::Application> {
        self.get(prop).ok_or_else(|| error::PropertyNotFound {
            #[allow(clippy::expect_used)]
            entity: EntitySection::from_str(record.section)
                .expect("Valid record sections should already be established by now!")
                .into(),
            property: prop,
            id: record.qualified_name.into(),
        })
    }
}

pub trait ListProperty {
    fn get_list(&self, prop: &'static str) -> impl Iterator<Item = &str>;
}

impl ListProperty for Properties {
    fn get_list(&self, prop: &'static str) -> impl Iterator<Item = &str> {
        self.get(prop)
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }
}
