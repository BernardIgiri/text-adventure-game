use convert_case::{Case, Casing};
use cursive::{
    align::HAlign,
    theme::{BaseColor, BorderStyle, Color, Effect, PaletteColor, Style, Theme},
    utils::{markup::StyledString, span::SpannedString},
    view::{IntoBoxedView, Resizable},
    views::{Button, DummyView, LinearLayout, SelectView, TextView},
    Cursive, CursiveExt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoomChoice {
    Chat,
    Interact,
    Leave,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartChatChoice {
    TalkTo(usize),
    NoOne,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractionChoice {
    Do(usize),
    Nothing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaveChoice {
    GoTo(usize),
    Stay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatChoice {
    RespondWith(usize),
    Leave,
}

enum UIChoice {
    None,
    InRoom(RoomChoice),
    StartChat(StartChatChoice),
    InChat(ChatChoice),
    Interact(InteractionChoice),
    Leave(LeaveChoice),
}

struct UIState {
    choice: UIChoice,
}

pub struct UI {
    siv: Cursive,
}

impl UI {
    pub fn new() -> Self {
        let mut siv = Cursive::default();
        let mut theme = Theme::default();
        theme.palette[PaletteColor::Background] = Color::Dark(BaseColor::Black);
        theme.palette[PaletteColor::View] = Color::Dark(BaseColor::Black);
        theme.palette[PaletteColor::Primary] = Color::Light(BaseColor::White);
        theme.palette[PaletteColor::TitlePrimary] = Color::Light(BaseColor::Yellow);
        theme.palette[PaletteColor::Highlight] = Color::Rgb(40, 40, 40);
        theme.palette[PaletteColor::HighlightText] = Color::Rgb(200, 200, 200);
        theme.borders = BorderStyle::None;
        siv.set_theme(theme);
        siv.add_global_callback('q', |s| s.quit());
        siv.set_user_data(UIState {
            choice: UIChoice::None,
        });
        Self { siv }
    }

    pub fn greet(&mut self, title: &str, greeting: &str) {
        let mut title_str = StyledString::new();
        title_str.append_styled(
            title,
            Style::from(Color::Light(BaseColor::Red)).combine(Effect::Bold),
        );
        let title_view = TextView::new(title_str).h_align(HAlign::Center);

        let mut greeting_str = StyledString::new();
        greeting_str.append_plain(greeting);
        let greeting_view = TextView::new(greeting_str).h_align(HAlign::Center);

        let pause = pause_for_any_key_view();

        let layout = LinearLayout::vertical()
            .child(DummyView.full_height())
            .weight(1)
            .child(title_view)
            .child(DummyView.fixed_height(2))
            .child(greeting_view)
            .child(DummyView.fixed_height(2))
            .child(pause)
            .child(DummyView.full_height())
            .weight(1)
            .full_width();
        self.siv.add_fullscreen_layer(layout);
        self.siv.run();
    }

    pub fn roll_credits(&mut self, title: &str, credits: &str) {
        let mut title_str = StyledString::new();
        title_str.append_styled(
            title,
            Style::from(Color::Light(BaseColor::Red)).combine(Effect::Bold),
        );
        let title_view = TextView::new(title_str).h_align(HAlign::Center);

        let mut credits_str = StyledString::new();
        credits_str.append_plain(credits);
        let credits_view = TextView::new(credits_str).h_align(HAlign::Center);

        let pause = pause_for_any_key_view();

        let layout = LinearLayout::vertical()
            .child(DummyView.full_height())
            .weight(1)
            .child(title_view)
            .child(DummyView.fixed_height(2))
            .child(credits_view)
            .child(DummyView.fixed_height(2))
            .child(pause)
            .child(DummyView.full_height())
            .weight(1)
            .full_width();
        self.siv.add_fullscreen_layer(layout);
        self.siv.run();
    }

    pub fn present_room(
        &mut self,
        room_name: &str,
        room_description: &str,
        items: &[String],
        characters: &[String],
        exits: &[String],
        has_actions: bool,
    ) -> RoomChoice {
        let (title, mut body) = prompt_header(room_name, room_description);
        let mut menu = SelectView::<RoomChoice>::new();

        if !items.is_empty() {
            body.append_styled("There are things here:\n", Effect::Bold);
            for item in items {
                body.append_plain(format!("- {}\n", item.to_case(Case::Sentence)));
            }
            body.append_plain("\n");
        }

        if !characters.is_empty() {
            body.append_styled("There are people here:\n", Effect::Bold);
            for name in characters {
                body.append_plain(format!("- {}\n", name.to_case(Case::Title)));
            }
            body.append_plain("\n");
            menu.add_item("Talk", RoomChoice::Chat);
        }

        if has_actions {
            menu.add_item("Interact", RoomChoice::Interact);
        }

        if !exits.is_empty() {
            body.append_styled("Your exits are:\n", Effect::Bold);
            for exit in exits {
                body.append_plain(format!("- {}\n", exit.to_case(Case::Title)));
            }
            body.append_plain("\n");
            menu.add_item("Go somewhere else", RoomChoice::Leave);
        }

        body.append_plain("What would you like to do?");

        let body_view = TextView::new(body).h_align(HAlign::Left);

        menu.add_item("End game", RoomChoice::GameOver);
        menu.set_on_submit(|siv, selected| {
            siv.with_user_data(|data: &mut UIState| {
                data.choice = UIChoice::InRoom(selected.clone());
            });
            siv.quit();
        });

        let layout = menu_layout(title, body_view, menu);

        self.siv.add_fullscreen_layer(layout);
        self.siv.run();
        if let Some(UIState {
            choice: UIChoice::InRoom(choice),
            ..
        }) = self.siv.user_data::<UIState>()
        {
            choice.clone()
        } else {
            panic!("Expected choice in room prompt!");
        }
    }

    pub fn present_chat_targets(
        &mut self,
        room_name: &str,
        room_description: &str,
        characters: &[String],
    ) -> StartChatChoice {
        let (title, mut body) = prompt_header(room_name, room_description);
        body.append_plain("Who will you talk to?");
        let mut menu = SelectView::<StartChatChoice>::new();
        for (i, choice) in characters.iter().enumerate() {
            menu.add_item(choice.to_case(Case::Title), StartChatChoice::TalkTo(i));
        }
        menu.set_on_submit(|siv, selected| {
            siv.with_user_data(|data: &mut UIState| {
                data.choice = UIChoice::StartChat(selected.clone());
            });
            siv.quit();
        });
        menu.add_item("No one", StartChatChoice::NoOne);

        let body_view = TextView::new(body).h_align(HAlign::Left);
        let layout = menu_layout(title, body_view, menu);

        self.siv.add_fullscreen_layer(layout);
        self.siv.run();
        if let Some(UIState {
            choice: UIChoice::StartChat(choice),
            ..
        }) = self.siv.user_data::<UIState>()
        {
            choice.clone()
        } else {
            panic!("Expected character in chat select!");
        }
    }

    pub fn present_chat(
        &mut self,
        character_name: &str,
        dialogue: &str,
        responses: &[String],
    ) -> ChatChoice {
        let title_view = TextView::new(character_name).h_align(HAlign::Center);
        let mut menu = SelectView::<ChatChoice>::new();
        for (i, choice) in responses.iter().enumerate() {
            menu.add_item(choice, ChatChoice::RespondWith(i));
        }
        menu.set_on_submit(|siv, selected| {
            siv.with_user_data(|data: &mut UIState| {
                data.choice = UIChoice::InChat(selected.clone());
            });
            siv.quit();
        });
        menu.add_item("Nothing", ChatChoice::Leave);

        let mut body = StyledString::new();
        body.append_plain(dialogue);
        body.append_plain("\nYou Say:");
        let body_view = TextView::new(body).h_align(HAlign::Left);

        let layout = menu_layout(title_view, body_view, menu);

        self.siv.add_fullscreen_layer(layout);
        self.siv.run();
        #[allow(clippy::unwrap_used)]
        if let Some(UIState {
            choice: UIChoice::InChat(choice),
            ..
        }) = self.siv.user_data::<UIState>()
        {
            choice.clone()
        } else {
            panic!("Expected response in chat prompt!");
        }
    }

    pub fn present_action_select(
        &mut self,
        room_name: &str,
        room_description: &str,
        actions: &[String],
    ) -> InteractionChoice {
        let (title, mut body) = prompt_header(room_name, room_description);
        body.append_plain("What will you do?");
        let mut menu = SelectView::<InteractionChoice>::new();
        for (i, choice) in actions.iter().enumerate() {
            menu.add_item(choice.to_case(Case::Sentence), InteractionChoice::Do(i));
        }
        menu.set_on_submit(|siv, selected| {
            siv.with_user_data(|data: &mut UIState| {
                data.choice = UIChoice::Interact(selected.clone())
            });
            siv.quit();
        });
        menu.add_item("Nothing", InteractionChoice::Nothing);

        let body_view = TextView::new(body).h_align(HAlign::Left);
        let layout = menu_layout(title, body_view, menu);

        self.siv.add_fullscreen_layer(layout);
        self.siv.run();
        if let Some(UIState {
            choice: UIChoice::Interact(choice),
            ..
        }) = self.siv.user_data::<UIState>()
        {
            choice.clone()
        } else {
            panic!("Expected action in action prompt!");
        }
    }

    pub fn present_action(&mut self, action_name: &str, action_description: &str) {
        let (title, body) = prompt_header(&action_name.to_case(Case::Title), action_description);
        let body_view = TextView::new(body).h_align(HAlign::Center);
        let pause = pause_for_any_key_view();

        let layout = LinearLayout::vertical()
            .child(DummyView.full_height())
            .weight(1)
            .child(title)
            .child(DummyView.fixed_height(1))
            .child(body_view)
            .child(DummyView.fixed_height(1))
            .child(pause)
            .child(DummyView.full_height())
            .weight(1)
            .full_width();
        let layout = LinearLayout::horizontal()
            .child(DummyView.full_width())
            .weight(1)
            .child(layout)
            .child(DummyView.full_width())
            .weight(1);
        self.siv.add_fullscreen_layer(layout);
        self.siv.run();
    }

    pub fn present_exit_select(
        &mut self,
        room_name: &str,
        room_description: &str,
        exits: &[String],
    ) -> LeaveChoice {
        let (title, mut body) = prompt_header(room_name, room_description);
        body.append_plain("Which way will you go?");
        let mut menu = SelectView::<LeaveChoice>::new();
        for (i, choice) in exits.iter().enumerate() {
            menu.add_item(choice.to_case(Case::Title), LeaveChoice::GoTo(i));
        }
        menu.set_on_submit(|siv, selected| {
            siv.with_user_data(|data: &mut UIState| {
                data.choice = UIChoice::Leave(selected.clone())
            });
            siv.quit();
        });
        menu.add_item("Stay", LeaveChoice::Stay);

        let body_view = TextView::new(body).h_align(HAlign::Left);
        let layout = menu_layout(title, body_view, menu);

        self.siv.add_fullscreen_layer(layout);
        self.siv.run();
        if let Some(UIState {
            choice: UIChoice::Leave(choice),
            ..
        }) = self.siv.user_data::<UIState>()
        {
            choice.clone()
        } else {
            panic!("Expected exit direction in exit room prompt!");
        }
    }
}

fn pause_for_any_key_view() -> Button {
    Button::new_raw("[ Continue... ]", |s| s.quit())
}

fn prompt_header(title_text: &str, description: &str) -> (TextView, SpannedString<Style>) {
    let mut title = StyledString::new();
    title.append_styled(
        title_text,
        Style::from(Color::Light(BaseColor::Blue)).combine(Effect::Bold),
    );
    let title_view = TextView::new(title).h_align(HAlign::Center);
    let mut body = StyledString::new();
    body.append_plain(format!("{}\n\n", description));
    (title_view, body)
}

fn menu_layout<V: IntoBoxedView + 'static>(
    title: TextView,
    body: TextView,
    menu: V,
) -> LinearLayout {
    let layout = LinearLayout::vertical()
        .child(DummyView.full_height())
        .weight(1)
        .child(title)
        .child(DummyView.fixed_height(1))
        .child(body)
        .child(menu)
        .child(DummyView.full_height())
        .weight(1);
    LinearLayout::horizontal()
        .child(DummyView.full_width())
        .weight(1)
        .child(layout)
        .child(DummyView.full_width())
        .weight(1)
}
