use ini::{Properties, SectionIter};
use std::{collections::HashSet, str::FromStr};
use strum::{EnumIter, EnumString, IntoStaticStr};

use crate::{
    core::{Identifier, IllegalConversion, Title},
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
    Root,
    Theme,
    Language,
}

#[derive(Debug)]
pub struct UnverifiedRecord<'a> {
    section: &'static str,
    name: &'a str,
    variant: Option<Identifier>,
    properties: &'a Properties,
    qualified_name: &'a str,
}

#[derive(Debug)]
pub struct Record<'a>(UnverifiedRecord<'a>);

impl<'a> UnverifiedRecord<'a> {
    fn from_root(properties: &'a Properties) -> Self {
        let section = EntitySection::Root.into();
        Self {
            section,
            name: "",
            variant: None,
            properties,
            qualified_name: section,
        }
    }
    fn try_new(input: &'a str, properties: &'a Properties) -> Result<Self, error::Application> {
        let mut section_parts = input.splitn(2, ':');
        let section = section_parts
            .next()
            .ok_or_else(|| error::InvalidPropertyValue {
                etype: "Unknown".into(),
                value: input.into(),
                field: "Section Name".into(),
            })?
            .trim();
        let section =
            EntitySection::from_str(section).map_err(|_| error::InvalidPropertyValue {
                etype: "Not Found".into(),
                value: input.into(),
                field: "Section Name".into(),
            })?;
        let qualified_name = section_parts.next().unwrap_or_default().trim();
        let (name, variant) =
            Self::title_str_and_variant_from_qualified(section.into(), qualified_name)?;
        Ok(Self {
            section: section.into(),
            name,
            variant,
            properties,
            qualified_name,
        })
    }
    fn title_str_and_variant_from_qualified(
        section: &'static str,
        qualified_name: &'a str,
    ) -> Result<(&'a str, Option<Identifier>), error::Application> {
        let mut parts = qualified_name.splitn(2, '|');
        let name = parts
            .next()
            .ok_or_else(|| error::InvalidPropertyValue {
                etype: section.into(),
                value: qualified_name.into(),
                field: "qualified name".into(),
            })?
            .trim();
        let variant =
            match parts.next() {
                Some(v) => Some(v.trim().parse::<Identifier>().map_err(|source| {
                    error::ConversionFailed {
                        etype: section.into(),
                        property: "variant".into(),
                        source,
                    }
                })?),
                None => None,
            };
        Ok((name, variant))
    }
    pub fn contains_key(&self, key: &'static str) -> bool {
        self.properties.contains_key(key)
    }
    pub fn into_record(
        self,
        required_props: &[&'static str],
        optional_props: &[&'static str],
    ) -> Result<Record<'a>, error::Application> {
        let found_keys: HashSet<&str> = self.properties.iter().map(|(k, _)| k).collect();
        let required_keys: HashSet<&str> = required_props.iter().copied().collect();
        let optional_keys: HashSet<&str> = optional_props.iter().copied().collect();
        let allowed_keys: HashSet<&str> = required_keys.union(&optional_keys).copied().collect();
        let mut missing_keys = required_keys.difference(&found_keys).peekable();
        let mut unexpected_keys = found_keys.difference(&allowed_keys).peekable();

        match (
            missing_keys.peek().is_some(),
            unexpected_keys.peek().is_some(),
        ) {
            (false, false) => Ok(Record(self)),
            (true, _) => {
                let id = self.qualified_name.into();
                Err(error::MissingProperties {
                    missing: missing_keys.map(|s| From::<&str>::from(s)).collect(),
                    etype: self.section.into(),
                    id,
                })
            }
            (_, true) => {
                let id = self.qualified_name.into();
                Err(error::UnexpectedProperties {
                    unexpected: unexpected_keys.map(|s| From::<&str>::from(s)).collect(),
                    etype: self.section.into(),
                    id,
                })
            }
        }
    }
}

impl<'a> Record<'a> {
    pub fn from_root(
        properties: &'a Properties,
        required_props: &[&'static str],
        optional_props: &[&'static str],
    ) -> Result<Self, error::Application> {
        let unverified = UnverifiedRecord::from_root(properties);
        unverified.into_record(required_props, optional_props)
    }
    pub fn parse_name<T>(&self) -> Result<T, error::Application>
    where
        T: FromStr<Err = IllegalConversion>,
    {
        self.0
            .name
            .parse()
            .map_err(|source| error::ConversionFailed {
                etype: self.0.section.into(),
                property: "name".into(),
                source,
            })
    }
    pub const fn entity_type(&self) -> &'static str {
        self.0.section
    }
    pub const fn variant(&self) -> &Option<Identifier> {
        &self.0.variant
    }
    pub const fn qualified_name(&self) -> &str {
        self.0.qualified_name
    }
    pub fn get(&self, prop: &'static str) -> Option<&str> {
        self.0.properties.get(prop)
    }
    pub fn require(&self, prop: &'static str) -> Result<&str, error::Application> {
        self.get(prop).ok_or_else(|| error::PropertyNotFound {
            etype: self.0.section.into(),
            property: prop.into(),
            id: self.0.qualified_name.into(),
        })
    }
    pub fn get_parsed<T>(&self, prop: &'static str) -> Result<Option<T>, error::Application>
    where
        T: FromStr<Err = IllegalConversion>,
    {
        self.get(prop).map_or_else(
            || Ok(None),
            |s| {
                s.parse::<T>()
                    .map_err(|source| error::ConversionFailed {
                        etype: self.0.section.into(),
                        property: "name".into(),
                        source,
                    })
                    .map(|s| Some(s))
            },
        )
    }
    pub fn require_parsed<T>(&self, prop: &'static str) -> Result<T, error::Application>
    where
        T: FromStr<Err = IllegalConversion>,
    {
        let s = self.require(prop)?;
        s.parse::<T>().map_err(|source| error::ConversionFailed {
            etype: self.0.section.into(),
            property: "name".into(),
            source,
        })
    }
    pub fn get_list(&self, prop: &'static str) -> impl Iterator<Item = &str> {
        self.0
            .properties
            .get(prop)
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }
    pub fn get_list_parsed<T>(
        &self,
        prop: &'static str,
    ) -> impl Iterator<Item = Result<T, error::Application>>
    where
        T: FromStr<Err = IllegalConversion>,
    {
        self.get_list(prop).map(|s| {
            s.parse::<T>().map_err(|source| error::ConversionFailed {
                etype: self.0.section.into(),
                property: "name".into(),
                source,
            })
        })
    }
    #[allow(dead_code)]
    pub fn require_list(
        &self,
        prop: &'static str,
    ) -> Result<impl Iterator<Item = &str>, error::Application> {
        Ok(self
            .require(prop)?
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty()))
    }
    pub fn parse_qualified_name(
        &self,
        qualified_name: &'a str,
    ) -> Result<(Title, Option<Identifier>), error::Application> {
        let (name, variant) =
            UnverifiedRecord::title_str_and_variant_from_qualified(self.0.section, qualified_name)?;
        Ok((
            name.parse::<Title>()
                .map_err(|source| error::ConversionFailed {
                    etype: self.0.section.into(),
                    property: "name".into(),
                    source,
                })?,
            variant,
        ))
    }
}

pub struct SectionRecordIter<'a> {
    iter: SectionIter<'a>,
    section: &'static str,
}

impl<'a> SectionRecordIter<'a> {
    pub fn new(iter: SectionIter<'a>, section: EntitySection) -> Self {
        Self {
            iter,
            section: section.into(),
        }
    }
}

impl<'a> Iterator for SectionRecordIter<'a> {
    type Item = Result<UnverifiedRecord<'a>, error::Application>;

    fn next(&mut self) -> Option<Self::Item> {
        for (input_opt, properties) in &mut self.iter {
            let input = match input_opt {
                Some(i) => i,
                None => continue,
            };
            match UnverifiedRecord::try_new(input, properties) {
                Ok(r) if r.section.trim() == self.section => {
                    return Some(Ok(r));
                }
                Err(e) => return Some(Err(e)),
                _ => continue,
            }
        }
        None
    }
}

pub trait IterRequireWith {
    type Item;

    fn require_next(
        &mut self,
        record: &Record<'_>,
        property_description: impl Into<Box<str>>,
    ) -> Result<Self::Item, error::Application>;
}

impl<I> IterRequireWith for I
where
    I: Iterator,
{
    type Item = I::Item;

    fn require_next(
        &mut self,
        record: &Record<'_>,
        property_description: impl Into<Box<str>>,
    ) -> Result<Self::Item, error::Application> {
        self.next().ok_or_else(|| error::PropertyNotFound {
            etype: record.entity_type().into(),
            property: property_description.into(),
            id: record.qualified_name().into(),
        })
    }
}

pub trait ParseWith {
    fn parse_with<'b, T>(
        &self,
        record: &Record<'b>,
        property_description: impl Into<Box<str>>,
    ) -> Result<T, error::Application>
    where
        T: FromStr,
        <T as FromStr>::Err: Into<IllegalConversion>;
}

impl ParseWith for &str {
    fn parse_with<'b, T>(
        &self,
        record: &Record<'b>,
        property_description: impl Into<Box<str>>,
    ) -> Result<T, error::Application>
    where
        T: FromStr,
        <T as FromStr>::Err: Into<IllegalConversion>,
    {
        self.parse::<T>().map_err(|source| error::ConversionFailed {
            etype: record.entity_type().into(),
            property: property_description.into(),
            source: source.into(),
        })
    }
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use ini::Properties;

    fn make_record<'a>(
        name: &'static str,
        map: &'a [(&'static str, &'static str)],
        props: &'a mut Properties,
    ) -> Record<'a> {
        make_unverified_record(name, map, props)
            .into_record(
                map.iter().map(|(k, _)| *k).collect::<Vec<_>>().as_slice(),
                &[],
            )
            .unwrap()
    }
    fn make_unverified_record<'a>(
        name: &'static str,
        map: &'a [(&'static str, &'static str)],
        props: &'a mut Properties,
    ) -> UnverifiedRecord<'a> {
        for (k, v) in map {
            props.insert((*k).to_string(), (*v).to_string());
        }
        UnverifiedRecord::try_new(name, props).unwrap()
    }

    #[test]
    fn require_success() {
        let mut props = Properties::new();
        let record = make_record(
            "Item:golden_ring",
            &[("description", "A golden ring.")],
            &mut props,
        );

        let result = record.require("description");
        assert_eq!(result.unwrap(), "A golden ring.");
    }

    #[test]
    fn require_failure() {
        let mut props = Properties::new();
        let record = make_record("Item:ring", &[], &mut props);

        let err = record.require("description").unwrap_err();
        match err {
            error::Application::PropertyNotFound {
                property,
                id,
                etype: entity,
            } => {
                assert_eq!(property, "description".into());
                assert_eq!(id, "ring".into());
                assert_eq!(entity, "Item".into());
            }
            _ => panic!("Expected PropertyNotFound"),
        }
    }

    #[test]
    fn expect_keys_success_with_optional() {
        let mut props = Properties::new();
        let record = make_unverified_record(
            "Item:ring",
            &[("description", "A ring"), ("weight", "5")],
            &mut props,
        );
        let result = record.into_record(&["description"], &["weight"]);
        assert!(result.is_ok());
    }

    #[test]
    fn expect_keys_success_required_only() {
        let mut props = Properties::new();
        let record = make_unverified_record("Item:ring", &[("description", "A ring")], &mut props);
        let result = record.into_record(&["description"], &["weight"]);
        assert!(result.is_ok());
    }

    #[test]
    fn expect_keys_missing_required() {
        let mut props = Properties::new();
        let record = make_unverified_record("Item:ring", &[("weight", "5")], &mut props);
        let err = record
            .into_record(&["description"], &["weight"])
            .unwrap_err();
        match err {
            error::Application::MissingProperties {
                missing,
                etype: entity,
                id,
            } => {
                assert_eq!(missing, vec!["description".into()]);
                assert_eq!(entity, "Item".into());
                assert_eq!(id, "ring".into());
            }
            _ => panic!("Expected PropertyNamesDontMatch"),
        }
    }

    #[test]
    fn expect_keys_unexpected_extra_key() {
        let mut props = Properties::new();
        let record = make_unverified_record(
            "Item:ring",
            &[("description", "A ring"), ("weight", "5"), ("color", "red")],
            &mut props,
        );
        let err = record
            .into_record(&["description"], &["weight"])
            .unwrap_err();
        match err {
            error::Application::UnexpectedProperties {
                unexpected,
                etype: entity,
                id,
            } => {
                assert_eq!(unexpected, vec!["color".into()]);
                assert_eq!(entity, "Item".into());
                assert_eq!(id, "ring".into());
            }
            _ => panic!("Expected PropertyNamesDontMatch"),
        }
    }
}
