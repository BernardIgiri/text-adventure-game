use crate::{
    entity::{Identifier, Response},
    error,
};

use super::{staging::Staging, world::WorldData};

// TODO: Implement this!
#[allow(unused_variables)]
pub fn list_responses(
    staging: &Staging,
    world: &WorldData,
) -> Result<Vec<(Identifier, Response)>, error::Application> {
    todo!()
}
