use crate::{
    entity::{Character, Title},
    error,
};

use super::{staging::Staging, world::WorldData};

// TODO: Implement this!
#[allow(unused_variables)]
pub fn list_characters(
    staging: &Staging,
    world: &WorldData,
) -> Result<Vec<(Title, Character)>, error::Application> {
    todo!()
}
