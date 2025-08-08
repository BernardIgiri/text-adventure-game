mod action;
mod character;
mod dialogue;
mod item;
mod iter;
mod preprocessor;
mod requirement;
mod response;
mod room;
mod title;

#[cfg(test)]
pub mod test_utils;

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use action::parse_actions;
use character::parse_characters;
use dialogue::parse_dialogues;
use indexmap::IndexMap;
use ini::Ini;
use item::parse_items;
use iter::EntitySection;
use response::parse_responses;
use room::parse_rooms;
use strum::IntoEnumIterator;
use title::{parse_language, parse_theme, parse_title};

use crate::{
    core::{
        ActionEntity, ActionId, ActionRaw, ChangeRoom, CharacterEntity, CharacterId,
        DialogueEntity, DialogueId, DialogueRaw, DialogueVariantEntity, DialogueVariantId,
        GameTitle, GiveItem, Identifier, ItemId, ReplaceItem, Requirement, RequirementRaw,
        ResponseEntity, ResponseId, RoomId, RoomRaw, RoomVariantEntity, RoomVariantId, Sequence,
        TakeItem, Teleport, Title, World,
    },
    error,
};

pub use preprocessor::*;

pub fn parse(ini: Ini) -> Result<World, error::Application> {
    validate_section_types(&ini)?;
    let title = parse_title(&ini)?;
    let theme = parse_theme(ini.iter())?;
    let language = parse_language(ini.iter())?;

    // load raw data
    let characters = parse_characters(ini.iter())?;
    let items = parse_items(ini.iter())?;
    let actions = parse_actions(ini.iter())?;
    let responses = parse_responses(ini.iter())?;
    let raw_rooms = parse_rooms(ini.iter())?; // has variants
    let raw_dialogues = parse_dialogues(ini.iter())?; // has variants

    // map ids
    let character_ids = characters
        .iter()
        .enumerate()
        .map(|(id, v)| (v.name.clone(), id.into()))
        .collect::<HashMap<Title, CharacterId>>();
    let item_ids = items
        .iter()
        .enumerate()
        .map(|(id, v)| (v.name.clone(), id.into()))
        .collect::<HashMap<Identifier, ItemId>>();
    let action_ids = actions
        .iter()
        .enumerate()
        .map(|(id, v)| (v.name().clone(), id.into()))
        .collect::<HashMap<Identifier, ActionId>>();
    let response_ids = responses
        .iter()
        .enumerate()
        .map(|(id, v)| (v.name.clone(), id.into()))
        .collect::<HashMap<Identifier, ResponseId>>();
    let room_ids = raw_rooms.map_ids();
    let dialouge_ids = raw_dialogues.map_ids();

    // build entities
    let title = GameTitle::builder()
        .title(title.title)
        .greeting(title.greeting)
        .credits(title.credits)
        .start_room(room_ids.get_id(&title.start_room)?)
        .build();
    let characters = characters
        .into_iter()
        .map(|raw| {
            Ok(CharacterEntity {
                name: raw.name.to_string(),
                start_dialogue: dialouge_ids.get_id(&raw.start_dialogue)?,
            })
        })
        .collect::<Result<Vec<_>, error::Application>>()?;
    let actions = actions
        .into_iter()
        .map(|raw| {
            Ok(match raw {
                ActionRaw::ChangeRoom(r) => ActionEntity::ChangeRoom(ChangeRoom {
                    name: r.name.to_string(),
                    description: r.description,
                    required: r.required.map(|v| item_ids.require(&v)).transpose()?,
                    room: room_ids.get_id(&r.room)?,
                    variant: room_ids.get_variant_id(&r.room, &r.variant)?,
                }),
                ActionRaw::GiveItem(r) => ActionEntity::GiveItem(GiveItem {
                    name: r.name.to_string(),
                    description: r.description,
                    required: r.required.map(|v| item_ids.require(&v)).transpose()?,
                    items: r
                        .items
                        .iter()
                        .map(|v| item_ids.require(v))
                        .collect::<Result<Vec<_>, error::Application>>()?,
                }),
                ActionRaw::ReplaceItem(r) => ActionEntity::ReplaceItem(ReplaceItem {
                    name: r.name.to_string(),
                    description: r.description,
                    original: item_ids.require(&r.original)?,
                    replacement: item_ids.require(&r.replacement)?,
                }),
                ActionRaw::TakeItem(r) => ActionEntity::TakeItem(TakeItem {
                    name: r.name.to_string(),
                    description: r.description,
                    items: r
                        .items
                        .iter()
                        .map(|v| item_ids.require(v))
                        .collect::<Result<Vec<_>, error::Application>>()?,
                }),
                ActionRaw::Teleport(r) => ActionEntity::Teleport(Teleport {
                    name: r.name.to_string(),
                    description: r.description,
                    required: r.required.map(|v| item_ids.require(&v)).transpose()?,
                    room: room_ids.get_id(&r.room)?,
                }),
                ActionRaw::Sequence(r) => ActionEntity::Sequence(Sequence {
                    name: r.name.to_string(),
                    description: r.description,
                    required: r.required.map(|v| item_ids.require(&v)).transpose()?,
                    actions: r
                        .actions
                        .iter()
                        .map(|v| action_ids.require(v))
                        .collect::<Result<Vec<_>, error::Application>>()?,
                }),
            })
        })
        .collect::<Result<Vec<_>, error::Application>>()?;
    let responses = responses
        .into_iter()
        .map(|raw| {
            Ok(ResponseEntity {
                text: raw.text.clone(),
                leads_to: raw.leads_to.map(|v| dialouge_ids.get_id(&v)).transpose()?,
                triggers: raw.triggers.map(|v| action_ids.require(&v)).transpose()?,
                requires: raw
                    .requires
                    .iter()
                    .map(|r| requirement_from_raw(r, &item_ids, &room_ids))
                    .collect::<Result<Vec<_>, error::Application>>()?,
            })
        })
        .collect::<Result<Vec<_>, error::Application>>()?;
    let mut rooms = Vec::new();
    for raw in raw_rooms {
        let id: RoomId = room_ids.get_id(&raw.name)?;
        if usize::from(id) + 1 > rooms.len() {
            rooms.push(Vec::new());
        }
        #[allow(clippy::expect_used)]
        rooms
            .last_mut()
            .expect("populated vec shouldn't be empty")
            .push(RoomVariantEntity {
                name: raw.name.to_string(),
                description: raw.description.clone(),
                characters: raw
                    .characters
                    .iter()
                    .map(|v| character_ids.require(v))
                    .collect::<Result<Vec<_>, error::Application>>()?,
                exits: raw
                    .exits
                    .iter()
                    .map(|(direction, name)| Ok((direction.clone(), room_ids.get_id(name)?)))
                    .collect::<Result<IndexMap<Identifier, RoomId>, error::Application>>()?,
                actions: raw
                    .actions
                    .iter()
                    .map(|v| action_ids.require(v))
                    .collect::<Result<Vec<_>, error::Application>>()?,
            });
    }
    let mut dialogues = Vec::new();
    for raw in raw_dialogues {
        let id: DialogueId = dialouge_ids.get_id(&raw.name)?;
        if usize::from(id) + 1 > dialogues.len() {
            dialogues.push(DialogueEntity::new());
        }
        #[allow(clippy::expect_used)]
        dialogues
            .last_mut()
            .expect("populated vec shouldn't be empty")
            .push(DialogueVariantEntity {
                text: raw.text.clone(),
                responses: raw
                    .responses
                    .iter()
                    .map(|v| response_ids.require(v))
                    .collect::<Result<Vec<_>, error::Application>>()?,
                requires: raw
                    .requires
                    .iter()
                    .map(|r| requirement_from_raw(r, &item_ids, &room_ids))
                    .collect::<Result<Vec<_>, error::Application>>()?,
            });
    }

    // Circular reference check
    for outer in actions.iter() {
        if let ActionEntity::Sequence(s) = &outer {
            for id in s.actions.iter() {
                if let ActionEntity::Sequence(inner) = &actions[usize::from(id)] {
                    return Err(error::CircularReferenceFound {
                        etype: "ActionSequence".into(),
                        parent_id: s.name.clone().into(),
                        child_id: inner.name.clone().into(),
                    });
                }
            }
        }
    }

    Ok(World::builder()
        .title(title)
        .theme(theme)
        .language(language)
        .items(items)
        .actions(actions)
        .rooms(rooms)
        .dialogues(dialogues)
        .characters(characters)
        .responses(responses)
        .build())
}

fn requirement_from_raw(
    raw: &RequirementRaw,
    item_ids: &HashMap<Identifier, ItemId>,
    room_ids: &IdMap<RoomRaw>,
) -> Result<Requirement, error::Application> {
    Ok(match raw {
        RequirementRaw::HasItem(n) => Requirement::HasItem(item_ids.require(n)?),
        RequirementRaw::RoomVariant(n, v) => {
            Requirement::RoomVariant(room_ids.get_id(n)?, room_ids.get_variant_id(n, v)?)
        }
        RequirementRaw::DoesNotHave(n) => Requirement::DoesNotHave(item_ids.require(n)?),
    })
}

trait HasEntityType {
    fn entity_type() -> &'static str;
}
impl HasEntityType for CharacterId {
    fn entity_type() -> &'static str {
        "Character"
    }
}
impl HasEntityType for ItemId {
    fn entity_type() -> &'static str {
        "Item"
    }
}
impl HasEntityType for ActionId {
    fn entity_type() -> &'static str {
        "Action"
    }
}
impl HasEntityType for ResponseId {
    fn entity_type() -> &'static str {
        "Response"
    }
}
impl HasEntityType for DialogueId {
    fn entity_type() -> &'static str {
        "Dialogue"
    }
}
impl HasEntityType for DialogueVariantId {
    fn entity_type() -> &'static str {
        "DialogueVariant"
    }
}
impl HasEntityType for RoomId {
    fn entity_type() -> &'static str {
        "Room"
    }
}
impl HasEntityType for RoomVariantId {
    fn entity_type() -> &'static str {
        "RoomVariant"
    }
}
trait RequireClone<K, V> {
    fn require(&self, key: &K) -> Result<V, error::Application>;
}

impl<K, V> RequireClone<K, V> for HashMap<K, V>
where
    K: ToString + Eq + std::hash::Hash,
    V: Clone + HasEntityType,
{
    fn require(&self, key: &K) -> Result<V, error::Application> {
        self.get(key).cloned().ok_or_else(|| error::EntityNotFound {
            etype: <V as HasEntityType>::entity_type().into(),
            id: key.to_string().into(),
        })
    }
}

#[allow(type_alias_bounds)]
type IdMap<T: HasNameVariant> =
    HashMap<T::Name, HashMap<Option<Identifier>, (T::Id, T::VariantId)>>;
trait HasNameVariant: Sized {
    type Name: PartialEq + Eq + Hash + Clone;
    type Id: From<usize> + Copy + Clone + HasEntityType;
    type VariantId: From<usize> + Copy + Clone + HasEntityType;
    fn name(&self) -> &Self::Name;
    fn variant(&self) -> &Option<Identifier>;
}
trait HasMapIds<T: HasNameVariant> {
    fn map_ids(&self) -> IdMap<T>;
}
impl<T: HasNameVariant> HasMapIds<T> for Vec<T> {
    fn map_ids(&self) -> IdMap<T> {
        let mut map: IdMap<T> = HashMap::new();
        let mut entity_index_map: HashMap<T::Name, usize> = HashMap::new();
        let mut next_entity_id = 0;
        let mut variant_counters: HashMap<T::Name, usize> = HashMap::new();
        for item in self.iter() {
            let entity_name = item.name().clone();
            let variant_name = item.variant().clone();
            let entity_id_usize =
                *entity_index_map
                    .entry(entity_name.clone())
                    .or_insert_with(|| {
                        let id = next_entity_id;
                        next_entity_id += 1;
                        id
                    });
            let entity_id = T::Id::from(entity_id_usize);

            let variant_id = if variant_name.is_none() {
                T::VariantId::from(0)
            } else {
                let counter = variant_counters.entry(entity_name.clone()).or_insert(1);
                let id = *counter;
                *counter += 1;
                T::VariantId::from(id)
            };

            let inner = map.entry(entity_name.clone()).or_default();
            inner.insert(variant_name, (entity_id, variant_id));
        }
        map
    }
}
trait HasIdAndVariantId<T: HasNameVariant> {
    fn get_id(&self, name: &T::Name) -> Result<T::Id, error::Application>;
    fn get_variant_id(
        &self,
        name: &T::Name,
        variant: &Option<Identifier>,
    ) -> Result<Option<T::VariantId>, error::Application>;
}
impl HasIdAndVariantId<DialogueRaw> for IdMap<DialogueRaw> {
    fn get_id(
        &self,
        name: &<DialogueRaw as HasNameVariant>::Name,
    ) -> Result<<DialogueRaw as HasNameVariant>::Id, error::Application> {
        Ok(self
            .get(name)
            .ok_or_else(|| error::EntityNotFound {
                etype: "Dialogue".into(),
                id: name.to_string().into(),
            })?
            .get(&None)
            .ok_or_else(|| error::DefaultEntityNotFound {
                etype: "Dialogue default".into(),
                id: name.to_string().into(),
            })?
            .0)
    }

    fn get_variant_id(
        &self,
        name: &<DialogueRaw as HasNameVariant>::Name,
        variant: &Option<Identifier>,
    ) -> Result<Option<<DialogueRaw as HasNameVariant>::VariantId>, error::Application> {
        Ok(if variant.is_none() {
            None
        } else {
            Some(
                self.get(name)
                    .ok_or_else(|| error::EntityNotFound {
                        etype: "Dialogue".into(),
                        id: name.to_string().into(),
                    })?
                    .get(variant)
                    .ok_or_else(|| error::EntityVariantNotFound {
                        etype: "Dialogue".into(),
                        id: name.to_string().into(),
                        variant: variant
                            .clone()
                            .map_or_else(|| "None".into(), |v| v.to_string().into()),
                    })?
                    .1,
            )
        })
    }
}
impl HasIdAndVariantId<RoomRaw> for IdMap<RoomRaw> {
    fn get_id(
        &self,
        name: &<RoomRaw as HasNameVariant>::Name,
    ) -> Result<<RoomRaw as HasNameVariant>::Id, error::Application> {
        Ok(self
            .get(name)
            .ok_or_else(|| error::EntityNotFound {
                etype: "Room".into(),
                id: name.to_string().into(),
            })?
            .get(&None)
            .ok_or_else(|| error::DefaultEntityNotFound {
                etype: "Room default".into(),
                id: name.to_string().into(),
            })?
            .0)
    }

    fn get_variant_id(
        &self,
        name: &<RoomRaw as HasNameVariant>::Name,
        variant: &Option<Identifier>,
    ) -> Result<Option<<RoomRaw as HasNameVariant>::VariantId>, error::Application> {
        Ok(if variant.is_none() {
            None
        } else {
            Some(
                self.get(name)
                    .ok_or_else(|| error::EntityNotFound {
                        etype: "Room".into(),
                        id: name.to_string().into(),
                    })?
                    .get(variant)
                    .ok_or_else(|| error::EntityVariantNotFound {
                        etype: "Room".into(),
                        id: name.to_string().into(),
                        variant: variant
                            .clone()
                            .map_or_else(|| "None".into(), |v| v.to_string().into()),
                    })?
                    .1,
            )
        })
    }
}

impl HasNameVariant for DialogueRaw {
    type Name = Identifier;
    type Id = DialogueId;
    type VariantId = DialogueVariantId;
    fn name(&self) -> &Self::Name {
        &self.name
    }
    fn variant(&self) -> &Option<Identifier> {
        &self.variant
    }
}
impl HasNameVariant for RoomRaw {
    type Name = Title;
    type Id = RoomId;
    type VariantId = RoomVariantId;
    fn name(&self) -> &Self::Name {
        &self.name
    }
    fn variant(&self) -> &Option<Identifier> {
        &self.variant
    }
}
fn validate_section_types(ini: &Ini) -> Result<(), error::Application> {
    let allowed_set: HashSet<&'static str> = EntitySection::iter().map(|s| s.into()).collect();
    for section in ini.sections().flatten() {
        let section = section.split(':').next().unwrap_or("");
        if !allowed_set.contains(section) {
            return Err(error::UnknownSectionFound(section.into()));
        }
    }
    Ok(())
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use super::*;
    use assert_matches::assert_matches;
    use asserting::prelude::*;
    use ini::Ini;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use std::collections::{BTreeSet, HashMap};

    #[test]
    fn validate_section_types_accepts_known_section_type() {
        let mut ini = Ini::new();
        ini.set_to(Some("Action:name"), "test".into(), "value".into());
        let r = validate_section_types(&ini);
        assert_that!(r).is_ok();
    }

    #[test]
    fn validate_section_types_rejects_unknown_section_type() {
        let mut ini = Ini::new();
        ini.set_to(Some("BadSection:name"), "test".into(), "value".into());
        let r = validate_section_types(&ini);
        assert_that!(r)
            .is_err()
            .extracting(|r| r.unwrap_err().to_string())
            .contains("BadSection");
    }

    #[derive(Clone, Debug, PartialEq)]
    struct FakeId;
    impl HasEntityType for FakeId {
        fn entity_type() -> &'static str {
            "FakeEntity"
        }
    }

    #[rstest]
    #[case::exists("present", true)]
    #[case::missing("missing", false)]
    fn require_clone_returns_expected(#[case] key: &str, #[case] exists: bool) {
        let mut map: HashMap<String, FakeId> = HashMap::new();
        map.insert("present".into(), FakeId);

        let result = map.require(&key.to_string());
        if exists {
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), FakeId);
        } else {
            assert!(result.is_err());
            let err = result.unwrap_err().to_string();
            assert!(err.contains("FakeEntity"));
            assert!(err.contains(key));
        }
    }

    #[derive(Clone)]
    struct DummyRaw {
        name: Title,
        variant: Option<Identifier>,
    }
    impl HasNameVariant for DummyRaw {
        type Name = Title;
        type Id = RoomId;
        type VariantId = RoomVariantId;
        fn name(&self) -> &Self::Name {
            &self.name
        }
        fn variant(&self) -> &Option<Identifier> {
            &self.variant
        }
    }

    #[test]
    fn map_ids_assigns_ids_and_variants() {
        fn raw(name: &str, variant: Option<&str>) -> DummyRaw {
            DummyRaw {
                name: name.parse().unwrap(),
                variant: variant.map(|v| v.parse().unwrap()),
            }
        }
        let raw_data = vec![
            raw("Room", None),
            raw("OtherRoom", Some("v1")),
            raw("Room", Some("v1")),
            raw("OtherRoom", None),
            raw("Room", Some("v2")),
        ];
        let map = raw_data.map_ids();
        let expectations = vec![
            (
                "Room",
                RoomId::from(0),
                vec![
                    RoomVariantId::from(0),
                    RoomVariantId::from(1),
                    RoomVariantId::from(2),
                ],
                vec![None, Some("v1"), Some("v2")],
            ),
            (
                "OtherRoom",
                RoomId::from(1),
                vec![RoomVariantId::from(0), RoomVariantId::from(1)],
                vec![None, Some("v1")],
            ),
        ];
        for (name, expected_id, expected_variant_ids, expected_variant_names) in expectations {
            let key = name.parse().unwrap();
            let room_map = map.get(&key).unwrap();
            assert_matches!(
                room_map.get(&None),
                Some((room, variant)) if *room == expected_id && *variant == RoomVariantId::from(0),
                "Default variant id should be zero"
            );
            assert_eq!(
                room_map.len(),
                expected_variant_names.len(),
                "Number of variants"
            );
            for k in expected_variant_names {
                let variant = k.map(|v| v.parse().unwrap());
                assert!(room_map.contains_key(&variant), "variant names match");
            }
            let variant_ids = room_map.values().map(|(_, v)| v).collect::<BTreeSet<_>>();
            assert_that!(variant_ids)
                .described_as(format!("{name} variant ids"))
                .contains_exactly(expected_variant_ids.iter().collect::<Vec<_>>());
        }
    }

    fn make_room_id_map() -> IdMap<RoomRaw> {
        let mut map: IdMap<RoomRaw> = HashMap::new();
        let mut variants = HashMap::new();
        variants.insert(None, (RoomId::from(0), RoomVariantId::from(0)));
        variants.insert(
            Some("alt".parse().unwrap()),
            (RoomId::from(0), RoomVariantId::from(1)),
        );
        map.insert("LivingRoom".parse().unwrap(), variants);
        map
    }

    #[rstest]
    #[case::valid_variant("LivingRoom", Some("alt"), true)]
    #[case::missing_variant("LivingRoom", Some("missing"), false)]
    #[case::missing_entity("Kitchen", Some("alt"), false)]
    fn get_variant_id_behavior(
        #[case] name: &str,
        #[case] variant: Option<&str>,
        #[case] is_ok: bool,
    ) {
        let map = make_room_id_map();
        let variant_opt = variant.map(|s| s.parse().unwrap());
        let result = map.get_variant_id(&name.parse().unwrap(), &variant_opt);
        assert_eq!(result.is_ok(), is_ok);
    }

    #[rstest]
    #[case::valid_id("LivingRoom", true)]
    #[case::missing_id("Kitchen", false)]
    fn get_id_behavior(#[case] name: &str, #[case] is_ok: bool) {
        let map = make_room_id_map();
        let result = map.get_id(&name.parse().unwrap());
        assert_eq!(result.is_ok(), is_ok);
    }

    #[rstest]
    #[case::valid(vec!["Action:name"], true)]
    #[case::invalid(vec!["BadSection:name"], false)]
    #[case::mixed(vec!["Action:test", "Item:test", "BadSection:test"], false)]
    fn validate_section_types_behavior(#[case] sections: Vec<&str>, #[case] expected_ok: bool) {
        let mut ini = Ini::new();
        for s in sections {
            ini.set_to(Some(s), "k".into(), "v".into());
        }
        let r = validate_section_types(&ini);
        assert_eq!(r.is_ok(), expected_ok);
    }

    fn title_section() -> &'static str {
        r#"
title = Some Title
greeting = Some greeting
credits = Some credits
start_room = RoomA
"#
    }

    fn room_a() -> &'static str {
        r#"
[Room:RoomA]
description=Room A description
exits=east:RoomB
characters=CharacterA
"#
    }

    fn room_a_alt() -> &'static str {
        r#"
[Room:RoomA|alt]
description=Room A alt description
"#
    }

    fn room_b() -> &'static str {
        r#"
[Room:RoomB]
description=Room B description
exits=east:RoomA
"#
    }

    fn item_a() -> &'static str {
        r#"
[Item:item_a]
description=Item a
"#
    }

    fn item_b() -> &'static str {
        r#"
[Item:item_b]
description=Item b
"#
    }

    fn action_change_room() -> &'static str {
        r#"
[Action:change_room_action]
change_room=RoomA->alt
description=Change room a to alt.
"#
    }

    fn action_give_item() -> &'static str {
        r#"
[Action:give_item_action]
give_item=item_a
description=Give item a
"#
    }

    fn action_take_item() -> &'static str {
        r#"
[Action:take_item_action]
take_item=item_b
description=Take item b
"#
    }

    fn action_sequence() -> &'static str {
        r#"
[Action:action_seq]
sequence=change_room_action,give_item_action,take_item_action
description=Sequence action
"#
    }

    fn character_a() -> &'static str {
        r#"
[Character:CharacterA]
start_dialogue=dialogue_a
"#
    }

    fn dialogue_a() -> &'static str {
        r#"
[Dialogue:dialogue_a]
text=Dialogue a
response=response_a,response_b
"#
    }

    fn dialogue_a_alt() -> &'static str {
        r#"
[Dialogue:dialogue_a|alt]
text=Dialogue a alt
requires=has_item:item_a,room_variant:RoomA
"#
    }

    fn dialogue_b() -> &'static str {
        r#"
[Dialogue:dialogue_b]
text=Dialogue b
"#
    }

    fn response_a() -> &'static str {
        r#"
[Response:response_a]
text=Response A
requires=has_item:item_b,does_not_have:item_a,room_variant:RoomA|alt
triggers=give_item_action
leads_to=dialogue_b
"#
    }

    fn response_b() -> &'static str {
        r#"
[Response:response_b]
text=Response B
"#
    }

    fn make_ini(sections: &[&str]) -> Ini {
        Ini::load_from_str(&sections.join("\n")).unwrap()
    }

    #[test]
    fn parse_minimal_valid_world() {
        let ini = make_ini(&[
            title_section(),
            room_a(),
            room_a_alt(),
            room_b(),
            item_a(),
            item_b(),
            action_give_item(),
            character_a(),
            dialogue_a(),
            dialogue_b(),
            response_a(),
            response_b(),
        ]);

        let world = parse(ini);
        assert_that!(world).is_ok();
    }

    #[rstest]
    #[case::missing_start_room(
        vec![r#"
            title = Some Title
            greeting = Hello
            credits = Credits
            start_room = MissingRoom
            "#, room_a()],
        |e: &error::Application| {
            assert_matches!(e, error::EntityNotFound { etype, id } if *etype == "Room".into() && *id == "Missing Room".into());
            true
        }
    )]
    #[case::unknown_section(
        vec![title_section(), "[BadSection:foo]\nkey=value"],
        |e: &error::Application| {
            assert_matches!(e, error::UnknownSectionFound(s) if *s == "BadSection".into());
            true
        }
    )]
    fn parse_fails_with_expected_error(
        #[case] sections: Vec<&str>,
        #[case] matcher: fn(&error::Application) -> bool,
    ) {
        let ini = make_ini(&sections);
        let result = parse(ini);
        assert_that!(result)
            .is_err()
            .extracting(|e| e.err().unwrap())
            .satisfies(matcher);
    }

    #[test]
    fn parse_detects_circular_sequences() {
        let ini = make_ini(&[
            title_section(),
            character_a(),
            room_a(),
            room_a_alt(),
            room_b(),
            item_a(),
            item_b(),
            action_change_room(),
            action_give_item(),
            action_take_item(),
            action_sequence(),
            dialogue_a(),
            dialogue_b(),
            response_a(),
            response_b(),
            r#"
            [Action:loop_action]
            sequence=action_seq
            description=Loop action
            "#,
        ]);

        let result = parse(ini);
        assert_that!(result)
            .is_err()
            .extracting(|e| e.err().unwrap())
            .satisfies(|e| {
                assert_matches!(e, error::CircularReferenceFound { etype, .. } if *etype == "ActionSequence".into());
                true
            });
    }

    #[test]
    fn parse_fails_on_invalid_requirement_item() {
        let ini = make_ini(&[
            title_section(),
            room_a(),
            room_a_alt(),
            room_b(),
            item_a(),
            item_b(),
            character_a(),
            action_give_item(),
            dialogue_a(),
            r#"
            [Dialogue:dialogue_a|alt]
            text=Dialogue a alt
            requires=has_item:missing_item
            "#,
            dialogue_b(),
            response_a(),
            response_b(),
        ]);

        let result = parse(ini);
        assert_that!(result)
            .is_err()
            .extracting(|e| e.err().unwrap())
            .satisfies(|e| {
                assert_matches!(e, error::EntityNotFound { etype, id } if *etype == "Item".into() && *id == "missing_item".into());
                true
            });
    }

    #[test]
    fn parse_fails_on_invalid_requirement_type() {
        let ini = make_ini(&[
            title_section(),
            room_a(),
            room_a_alt(),
            room_b(),
            item_a(),
            item_b(),
            character_a(),
            action_give_item(),
            dialogue_a(),
            r#"
            [Dialogue:dialogue_a|alt]
            text=Dialogue a alt
            requires=fake:item_a
            "#,
            dialogue_b(),
            response_a(),
            response_b(),
        ]);

        let result = parse(ini);
        assert_that!(result)
            .is_err()
            .extracting(|e| e.err().unwrap())
            .satisfies(|e| {
                assert_matches!(e,
                    error::InvalidPropertyValue { etype, value, field }
                    if *etype == "Dialogue".into() &&
                        *value == "fake:item_a".into()
                        && *field == "requirement".into()
                );
                true
            });
    }

    #[test]
    fn parse_valid_with_variants_and_sequence() {
        let ini = make_ini(&[
            title_section(),
            room_a(),
            room_a_alt(),
            room_b(),
            item_a(),
            item_b(),
            action_change_room(),
            action_give_item(),
            action_take_item(),
            action_sequence(),
            character_a(),
            dialogue_a(),
            dialogue_a_alt(),
            dialogue_b(),
            response_a(),
            response_b(),
        ]);

        let world = parse(ini);
        assert_that!(world).is_ok();
    }

    #[rstest]
    #[case::missing_default_dialogue(
    vec![
        title_section(),
        room_a(),
        room_b(),
        item_a(),
        item_b(),
        action_give_item(),
        character_a(), // references dialogue_a
        // Missing default variant for dialogue_a
        r#"
        [Dialogue:dialogue_a|alt]
        text=Dialogue a alt
        "#,
        response_a(),
        response_b(),
    ],
    |e: &error::Application| {
        assert_matches!(
            e,
            error::DefaultEntityNotFound { etype, id } if *etype == "Dialogue default".into() && *id == "dialogue_a".into()
        );
        true
    }
    )]
    #[case::invalid_room_variant(
    vec![
        title_section(),
        room_a(),
        room_b(),
        item_a(),
        item_b(),
        r#"
        [Action:invalid_variant_action]
        change_room=RoomA->nonexistent_variant
        description=Invalid variant change
        "#,
        character_a(),
        dialogue_a(),
        dialogue_b(),
        response_a(),
        response_b(),
    ],
    |e: &error::Application| {
        assert_matches!(
            e,
            error::EntityVariantNotFound { etype, id, variant }
            if *etype == "Room".into() && *id == "Room A".into() && *variant == "nonexistent_variant".into()
        );
        true
    }
    )]
    #[case::missing_dialogue_referenced_by_response(
    vec![
        title_section(),
        room_a(),
        room_b(),
        item_a(),
        item_b(),
        action_give_item(),
        character_a(),
        dialogue_a(),
        response_a(), // leads_to=dialogue_b, but dialogue_b missing
        response_b(),
    ],
    |e: &error::Application| {
        assert_matches!(
            e,
            error::EntityNotFound { etype, id } if *etype == "Dialogue".into() && *id == "dialogue_b".into()
        );
        true
    }
    )]
    fn parse_additional_invalid_cases(
        #[case] sections: Vec<&str>,
        #[case] matcher: fn(&error::Application) -> bool,
    ) {
        let ini = make_ini(&sections);
        let result = parse(ini);
        assert_that!(result)
            .is_err()
            .extracting(|e| e.err().unwrap())
            .satisfies(matcher);
    }
}
