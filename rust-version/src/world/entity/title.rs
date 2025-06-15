use derive_getters::Getters;
use derive_new::new;

use super::Title;

#[derive(Getters, new, Debug, Clone, PartialEq, Eq)]
pub struct GameTitle {
    title: String,
    start_room: Title,
}
