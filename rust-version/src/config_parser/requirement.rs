use crate::{
    entity::{Identifier, Requirement},
    error,
};

use super::{staging::Staging, world::WorldData};

// TODO: Implement this!
#[allow(unused_variables)]
pub fn list_requirements(
    staging: &Staging,
    world: &WorldData,
) -> Result<Vec<(Identifier, Requirement)>, error::Application> {
    todo!()
}
