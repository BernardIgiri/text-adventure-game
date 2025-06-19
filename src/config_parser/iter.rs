use ini::{Properties, SectionIter};
use std::{collections::HashSet, str::FromStr};
use strum::{EnumIter, EnumString, IntoStaticStr};

use crate::{
    core::{Identifier, Title},
    error,
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
                Ok(r) if r.section.trim() == self.1.trim() => {
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

pub trait RecordProperty {
    fn require(&self, prop: &'static str, record: &Record) -> Result<&str, error::Application>;
    fn expect_keys(
        &self,
        required: &[&'static str],
        optional: &[&'static str],
        record: &Record,
    ) -> Result<(), error::Application>;
}

impl RecordProperty for Properties {
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

    fn expect_keys(
        &self,
        required: &[&'static str],
        optional: &[&'static str],
        record: &Record,
    ) -> Result<(), error::Application> {
        let found_keys: HashSet<&str> = self.iter().map(|(k, _)| k).collect();
        let required_keys: HashSet<&str> = required.iter().copied().collect();
        let optional_keys: HashSet<&str> = optional.iter().copied().collect();
        let allowed_keys: HashSet<&str> = required_keys.union(&optional_keys).copied().collect();

        if required_keys.difference(&found_keys).next().is_none()
            && found_keys.difference(&allowed_keys).next().is_none()
        {
            Ok(())
        } else {
            let mut expected_props: Vec<&str> = allowed_keys.into_iter().collect();
            expected_props.sort_unstable();
            let expected_props = expected_props
                .iter()
                .map(|p| {
                    if optional_keys.contains(p) {
                        format!("{p}*")
                    } else {
                        p.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let mut found_props: Vec<&str> = found_keys.into_iter().collect();
            found_props.sort_unstable();

            Err(error::PropertyNamesDontMatch {
                #[allow(clippy::expect_used)]
                entity: EntitySection::from_str(record.section)
                    .expect("Valid record sections should already be established by now!")
                    .into(),
                id: record.qualified_name.into(),
                expected_props,
                found_props: found_props.join(", "),
            })
        }
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

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use ini::Properties;

    fn make_record<'a>(section: &'a str, name: &'a str, properties: &'a Properties) -> Record<'a> {
        Record {
            section,
            name,
            variant: None,
            properties,
            qualified_name: name, // for testing, same as name
        }
    }

    fn make_properties(map: &[(&str, &str)]) -> Properties {
        let mut props = Properties::new();
        for (k, v) in map {
            props.insert((*k).to_string(), (*v).to_string());
        }
        props
    }

    #[test]
    fn test_require_success() {
        let props = make_properties(&[("description", "A golden ring.")]);
        let record = make_record("Item", "ring", &props);

        let result = props.require("description", &record);
        assert_eq!(result.unwrap(), "A golden ring.");
    }

    #[test]
    fn test_require_failure() {
        let props = make_properties(&[]);
        let record = make_record("Item", "ring", &props);

        let err = props.require("description", &record).unwrap_err();
        match err {
            error::Application::PropertyNotFound {
                property,
                id,
                entity,
            } => {
                assert_eq!(property, "description");
                assert_eq!(id, "ring");
                assert_eq!(entity, "Item");
            }
            _ => panic!("Expected PropertyNotFound"),
        }
    }

    #[test]
    fn test_expect_keys_success_with_optional() {
        let props = make_properties(&[("description", "A ring"), ("weight", "5")]);
        let record = make_record("Item", "ring", &props);

        let result = props.expect_keys(&["description"], &["weight"], &record);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expect_keys_success_required_only() {
        let props = make_properties(&[("description", "A ring")]);
        let record = make_record("Item", "ring", &props);

        let result = props.expect_keys(&["description"], &["weight"], &record);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expect_keys_missing_required() {
        let props = make_properties(&[("weight", "5")]);
        let record = make_record("Item", "ring", &props);

        let err = props
            .expect_keys(&["description"], &["weight"], &record)
            .unwrap_err();
        match err {
            error::Application::PropertyNamesDontMatch {
                expected_props,
                found_props,
                entity,
                id,
            } => {
                assert_eq!(expected_props, "description, weight*");
                assert_eq!(found_props, "weight");
                assert_eq!(entity, "Item");
                assert_eq!(id, "ring");
            }
            _ => panic!("Expected PropertyNamesDontMatch"),
        }
    }

    #[test]
    fn test_expect_keys_unexpected_extra_key() {
        let props =
            make_properties(&[("description", "A ring"), ("weight", "5"), ("color", "red")]);
        let record = make_record("Item", "ring", &props);

        let err = props
            .expect_keys(&["description"], &["weight"], &record)
            .unwrap_err();
        match err {
            error::Application::PropertyNamesDontMatch {
                expected_props,
                found_props,
                entity,
                id,
            } => {
                assert_eq!(expected_props, "description, weight*");
                assert_eq!(found_props, "color, description, weight");
                assert_eq!(entity, "Item");
                assert_eq!(id, "ring");
            }
            _ => panic!("Expected PropertyNamesDontMatch"),
        }
    }
}
