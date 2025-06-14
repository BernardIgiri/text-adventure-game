 - [ ] Remap depedencies. If all of the potential circular depedencies are broken up, then parsing could be made much simpler.


Item

Room
	- Room (w)
	- Item
	- Action (w)

Action
	- Room
	- Item

Response
	- Dialog (w)
	- Action
	- Requirement
		- Item
		- Room

Dialogue
	- Response
	- Requirement
		- Item
		- Room

Character
	- Room
	- Dialogue

// In absense of Responses there needs to be a generic Goodby response
// In absense of Dialogue there needs to be a generic "..." dialogue
// Add a GameOver action
    rooms
        .room_refs()
        .map(|room| {
            let missing_action_ids: Vec<String> = room
                .actions()
                .iter()
                .filter(|id| !actions.contains_key(id))
                .map(ToString::to_string)
                .collect();
            if !missing_action_ids.is_empty() {
                return Err(error::EntityNotFound {
                    etype: "Action(s)",
                    id: missing_action_ids.join(", "),
                });
            }
            let missing_room_ids: Vec<String> = room
                .exits()
                .values()
                .filter(|id| !rooms.contains_key(id))
                .map(ToString::to_string)
                .collect();
            if !missing_room_ids.is_empty() {
                return Err(error::EntityNotFound {
                    etype: "Room(s)",
                    id: missing_room_ids.join(", "),
                });
            }
            Ok(())
        })
        .collect::<Result<Vec<_>, _>>()?;

xpub trait RoomRefsIter<'a> {
    type Iter: Iterator<Item = RoomRefs<'a>>;
    fn room_refs(&'a self) -> Self::Iter;
}

impl<'a> RoomRefsIter<'a> for RoomMap {
    type Iter = Map<
        FlatMap<
            hash_map::Values<'a, Title, HashMap<Option<Identifier>, Rc<Room>>>,
            hash_map::Values<'a, Option<Identifier>, Rc<Room>>,
            fn(
                &'a HashMap<Option<Identifier>, Rc<Room>>,
            ) -> hash_map::Values<'a, Option<Identifier>, Rc<Room>>,
        >,
        fn(&'a Rc<Room>) -> RoomRefs<'a>,
    >;

    fn room_refs(&'a self) -> Self::Iter {
        self.values()
            .flat_map(HashMap::values as _)
            .map(|room| RoomRefs::new(room))
    }
}

pub trait CharacterRefsIter<'a> {
    type Iter: Iterator<Item = CharacterRefs<'a>>;
    fn character_refs(&'a self) -> Self::Iter;
}

impl<'a> CharacterRefsIter<'a> for CharacterMap {

}
