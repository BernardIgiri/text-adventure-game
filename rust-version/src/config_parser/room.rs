use crate::{
    entity::{Identifier, Room, Title},
    error,
};

use super::{staging::Staging, world::WorldData};

// TODO: Implement this!
#[allow(unused_variables)]
pub fn list_rooms(
    staging: &Staging,
    world: &WorldData,
) -> Result<Vec<(Title, Option<Identifier>, Room)>, error::Application> {
    todo!()
}
