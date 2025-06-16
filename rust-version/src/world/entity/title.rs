use derive_getters::Getters;
use derive_new::new;

use super::Title;

#[derive(Getters, new, Debug, PartialEq, Eq)]
pub struct GameTitle {
    title: String,
    greeting: String,
    credits: String,
    start_room: Title,
}
