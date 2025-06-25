# **Text Adventure Game**

Welcome to **Text Adventure Game,** a game engine for making TUI text adventure games, using the quaint magic of 90's era ini files!

Ini files were selected in hopes of keeping the syntax simple and kid-accessible. That's right, kids can make a game in minutes with this engine! You can describe rooms, characters, items, chat dialogues, responses, and actions. You can customize the theme and even swap out all of the game text to support non-English languages. If you want text to span multiple lines, you can do so with a simple triple quote `"""` at the beginning and end of your text. That is the one enhancement to ini files provided here.

## **Instructions**

To create a game, first install `text-adventure-game`, then create an `.ini` file describing your game world. Run your game using:

```sh
text-adventure-game -f path/to/your/game.ini
```

## Download

| Platform | Link |
|----------|------|
| 🐧 Linux (musl) | [Download](https://github.com/BernardIgiri/text-adventure-game/releases/latest/download/text-adventure-game-x86_64-unknown-linux-musl.tar.gz) |
| 🪟 Windows | [Download](https://github.com/BernardIgiri/text-adventure-game/releases/latest/download/text-adventure-game-x86_64-pc-windows-msvc.zip) |
| 🍎 macOS | [Download](https://github.com/BernardIgiri/text-adventure-game/releases/latest/download/text-adventure-game-x86_64-apple-darwin.tar.gz) |

For a demo, try the [example.ini](https://raw.githubusercontent.com/BernardIgiri/text-adventure-game/refs/heads/main/example.ini)

## **INI File Structure**

Your game is defined in a single `.ini` file. Each section defines a different part of the game world: the title screen, theme, language, rooms, characters, dialogues, items, and more.

### **Root Section: Game Title**

The root (unnamed) section must define the following fields:

```ini
title=The Game Title
greeting=Welcome to your next adventure!
credits=Thanks for playing!
start_room=StartingRoom
```

- `title`: Displayed at game launch.
- `greeting`: Displayed when the game begins.
- `credits`: Displayed when the game ends.
- `start_room`: The `Title` id of the room the player begins in.

### **[Theme] (Optional)**

```ini
[Theme]
title = Red
heading = Green
background = #FF3344
text = #FFFFFF
highlight = RGB(10, 50, 10)
highlight_text = Blue
subdued = Gray
```

Defines colors for the UI. Most valid CSS colors should work, including RGB, hex codes, and common color name strings.

### **[Language] (Optional)**

Allows you to override built-in strings (e.g., “Talk”, “Do”, “Go”) for localization or stylistic changes.

```ini
[Language]
characters_found = You look around and see:
exits_found = You can exit in these directions:
talk = Talk
interact = Interact
go_somewhere = Go someplace?
end_game = End the game?
choose_exit = Get out of here?
cancel_exit = Don't leave
choose_chat = Talk to:
cancel_chat = Nevermind
choose_response = You say:
cancel_response = ...
choose_action = You decide to:
cancel_action = Nevermind
action_failed = That didn't work
continue_game = Keep Going?
press_q_to_quit = The letter q is for quit!
```

------

## **Game Entities**

Each entity section starts with one of the following headers:

- `[Room:RoomName]`
- `[Room:RoomName|variant]` (see **Variants** below)
- `[Character:CharacterName]`
- `[Dialogue:dialogue_id]`
- `[Dialogue:dialogue_id|variant]` (see **Variants** below)
- `[Response:response_id]`
- `[Item:item_id]`
- `[Action:action_id]`

### **Naming Conventions**

| Type         | Syntax     | Example        |
| ------------ | ---------- | -------------- |
| `Title`      | CamelCase  | `StartingRoom` |
| `Identifier` | snake_case | `the_ring`     |

The engine will prettify names when shown to players:

- `StartingRoom` → **Starting Room**
- `the_ring` → **The Ring**

Only rooms and character names use `Title` syntax. Everything else is an `Identifier`.

### **Entity Variants**

Some entities can have variants to reflect dynamic changes (e.g., after an event):

```ini
[Room:Basement|rubble]
description=The basement is filled with rubble.
```

The default variant is the one with no `|variant` suffix. All entities that have variants must include a default. Variants use `Identifier` syntax.

------

## **Room**

Each room must define:

```ini
[Room:Basement]
description=A cold, damp basement.
exits=north:LivingRoom,east:SecretLab
characters=CuriousCalvin
actions=open_crate
```

- `description`: Text shown when entering the room.
- `exits`: (Optional) Comma-separated list of directions and destinations. Each direction is separated from the destination room name by a colon `:`.
- `characters`: (Optional) Comma-separated list of characters present.
- `actions`: (Optional) Comma-separated list of actions available.

**If a room has no exits, the game ends when the player enters it!**

## **Character**

```ini
[Character:NeighborFrank]
start_dialogue=hello
```

- `start_dialogue`: The ID of the dialogue shown when the player talks to this character.

------

## **Dialogue**

```ini
[Dialogue:hello]
text=Hi there!
responses=wave,goodbye
```

Dialogues show text and present a list of responses. They can have **variants** to show different text based on game state.

- `text`: Text spoken by the character.
- `responses`: (Optional) Comma-separated list of response IDs. If omitted, the chat ends immediately.

```ini
[Dialogue:hello|rude]
text=Go away!
responses=shrug
requires=has_item:the_ring
```

### **Requirements**

Use the optional `requires` attribute to conditionally show dialogue variants. Supported conditions:

- `has_item:item_id`
- `room_variant:RoomName|variant`

If no requirements match, the default variant (no `|variant`) is shown.

------

## **Response**

```ini
[Response:wave]
text=You wave back.
leads_to=goodbye
```

- `text`: This text appears as a selectable menu option.
- `leads_to`: (Optional) The next dialogue ID. If not specified, the chat ends.

`requires` works the same way as for dialogue, but responses **do not** have variants.

------

## **Item**

```ini
[Item:the_ring]
description=A mysterious golden ring.
```

Items can be given to or taken from the player via actions.

------

## **Action**

```ini
[Action:pull_lever]
change_room=WoodShed->closed
description=You pull the hefty lever and hear a satisfying clunk! Immediately, the lights go out, and the lever seizes in place.

[Action:pay_bribe]
give_item=silver_coin
description=You give away your last coin begrudgingly.

[Action:unlock_chest]
replace_item=key->ring
description=You unlock the chest and discover a golden ring!

[Action:pickup_key]
take_item=key
description=You pick up the dingy key on the floor.

[Action:beam_me_up]
teleport_to=Enterprise
required=silver_coin
description=Scotty teleports you aboard the ship!

[Action:push_the_red_button]
sequence=pickup_key,unlock_chest,beam_me_up
required=golden_ticket
description=You don't quite know what just happened, but you are now on a spaceship with a ring in your hand!
```

Actions perform in-game effects. Supported types:

- `ChangeRoom`: Changes a room to the specified variant.
- `GiveItem`: Adds items to the player's inventory.
- `TakeItem`: Removes items from the player's inventory.
- `ReplaceItem`: Swaps one item for another.
- `Teleport`: Instantly moves the player to a room.
- `Sequence`: Chains multiple actions.

The `Teleport`, `Sequence`, and `ChangeRoom` actions may include a `required` field. This specifies an item the player must have to perform the action. If used, the item is consumed when the action is executed.

------

## **Multiline Text**

Wrap multiline strings in triple quotes:

```ini
description="""
    A cold wind blows.
    You see your breath.
"""
```
