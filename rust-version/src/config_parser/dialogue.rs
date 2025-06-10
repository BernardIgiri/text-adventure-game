use crate::{
    entity::{Dialogue, Identifier},
    error,
};

use super::{staging::Staging, world::WorldData};

// TODO: Implement this!
#[allow(unused_variables)]
pub fn list_dialogues(
    staging: &Staging,
    world: &WorldData,
) -> Result<Vec<(Identifier, Option<Identifier>, Dialogue)>, error::Application> {
    todo!()
}
