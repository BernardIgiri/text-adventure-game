use std::rc::Rc;

use convert_case::Casing;
use cursive::{
    align::HAlign,
    theme::{BorderStyle, Color, Effect, PaletteColor, Style, Theme as SivTheme},
    utils::markup::StyledString,
    view::{IntoBoxedView, Nameable, Resizable},
    views::{
        self, Button, DummyView, LayerPosition, LinearLayout, ScrollView, SelectView, TextView,
    },
    Cursive, CursiveExt,
};

use crate::core::{Language, Theme, ThemeColor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoomChoice {
    Chat,
    Interact,
    ViewInventory,
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum UIChoice {
    None,
    InRoom(RoomChoice),
    StartChat(StartChatChoice),
    InChat(ChatChoice),
    Interact(InteractionChoice),
    Leave(LeaveChoice),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MenuItem(String, UIChoice);

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct MenuScreen {
    title: String,
    body: String,
    prompt: String,
    menu: Vec<MenuItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UIState {
    choice: UIChoice,
}

pub struct UI {
    siv: Cursive,
    theme: Rc<Theme>,
    language: Rc<Language>,
    screen: MenuScreen,
}

impl From<&ThemeColor> for Color {
    fn from(color: &ThemeColor) -> Self {
        Self::Rgb(*color.r(), *color.g(), *color.b())
    }
}

impl UI {
    pub fn new(theme: Rc<Theme>, language: Rc<Language>) -> Self {
        let mut siv = Cursive::default();
        let mut siv_theme = SivTheme::default();
        siv_theme.palette[PaletteColor::Background] = theme.background().into();
        siv_theme.palette[PaletteColor::View] = theme.background().into();
        siv_theme.palette[PaletteColor::Primary] = theme.text().into();
        siv_theme.palette[PaletteColor::TitlePrimary] = theme.title().into();
        siv_theme.palette[PaletteColor::Highlight] = theme.highlight().into();
        siv_theme.palette[PaletteColor::HighlightText] = theme.highlight_text().into();
        siv_theme.borders = BorderStyle::None;
        siv.set_theme(siv_theme);
        siv.add_global_callback('q', |s| s.quit());
        siv.set_user_data(UIState {
            choice: UIChoice::None,
        });
        Self {
            siv,
            theme,
            language,
            screen: MenuScreen::default(),
        }
    }
    pub fn greet(&mut self, title: &str, greeting: &str) {
        let mut title_str = StyledString::new();
        title_str.append_styled(
            title,
            Style::from(Color::from(self.theme.title())).combine(Effect::Bold),
        );
        let title_view = TextView::new(title_str).h_align(HAlign::Center);

        let mut greeting_str = StyledString::new();
        greeting_str.append_plain(greeting);
        let greeting_view = TextView::new(greeting_str).h_align(HAlign::Center);
        let greeting_view = ScrollView::new(greeting_view);

        let pause = self.pause_for_any_key_view();

        let layout = LinearLayout::vertical()
            .child(DummyView.full_height())
            .weight(1)
            .child(title_view)
            .child(DummyView.fixed_height(2))
            .child(greeting_view)
            .child(DummyView.fixed_height(2))
            .child(pause)
            .child(DummyView.full_height())
            .weight(1);
        let layout = LinearLayout::horizontal()
            .child(DummyView.full_width())
            .weight(1)
            .child(layout)
            .child(DummyView.full_width())
            .weight(1);
        self.swap_layer(layout);
        self.siv.run();
        self.switch_to_menu_screen();
    }
    pub fn roll_credits(&mut self, title: &str, credits: &str) {
        let mut title_str = StyledString::new();
        title_str.append_styled(
            title,
            Style::from(Color::from(self.theme.title())).combine(Effect::Bold),
        );
        let title_view = TextView::new(title_str).h_align(HAlign::Center);

        let mut credits_str = StyledString::new();
        credits_str.append_plain(credits);
        let credits_view = TextView::new(credits_str).h_align(HAlign::Center);
        let credits_view = ScrollView::new(credits_view);

        let pause = self.pause_for_any_key_view();

        let layout = LinearLayout::vertical()
            .child(DummyView.full_height())
            .weight(1)
            .child(title_view)
            .child(DummyView.fixed_height(2))
            .child(credits_view)
            .child(DummyView.fixed_height(2))
            .child(pause)
            .child(DummyView.full_height())
            .weight(1);
        let layout = LinearLayout::horizontal()
            .child(DummyView.full_width())
            .weight(1)
            .child(layout)
            .child(DummyView.full_width())
            .weight(1);
        self.swap_layer(layout);
        self.siv.run();
    }
    pub fn present_room(
        &mut self,
        room_name: &str,
        room_description: &str,
        characters: &[String],
        exits: &[String],
        has_actions: bool,
        has_inventory: bool,
    ) -> RoomChoice {
        let mut menu = Vec::new();
        let mut body = String::new();
        body.push_str(room_description);
        body.push_str("\n\n");
        if !characters.is_empty() {
            body.push_str(self.language.characters_found());
            body.push(' ');
            body.push_str(&characters.join(", "));
            body.push('\n');
            menu.push(MenuItem(
                self.language.talk().into(),
                UIChoice::InRoom(RoomChoice::Chat),
            ));
        }
        if has_actions {
            menu.push(MenuItem(
                self.language.interact().into(),
                UIChoice::InRoom(RoomChoice::Interact),
            ));
        }
        if !exits.is_empty() {
            body.push_str(self.language.exits_found());
            body.push(' ');
            body.push_str(
                &exits
                    .iter()
                    .map(|s| s.to_string().to_case(convert_case::Case::Title))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            body.push('\n');
            menu.push(MenuItem(
                self.language.go_somewhere().into(),
                UIChoice::InRoom(RoomChoice::Leave),
            ));
            if has_inventory {
                menu.push(MenuItem(
                    self.language.view_inventory().into(),
                    UIChoice::InRoom(RoomChoice::ViewInventory),
                ));
            }
        } else {
            menu.push(MenuItem(
                self.language.end_game().into(),
                UIChoice::InRoom(RoomChoice::GameOver),
            ));
        }
        self.show_menu(MenuScreen {
            title: room_name.into(),
            prompt: self.language.choose_action().into(),
            body,
            menu,
        });
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
    pub fn present_inventory(&mut self, items: &[String]) {
        let menu = vec![MenuItem(
            self.language.continue_game().into(),
            UIChoice::None,
        )];
        let body = format!("- {}", items.join("\n- "));
        self.show_menu(MenuScreen {
            title: self.language.inventory().clone(),
            body,
            prompt: "".into(),
            menu,
        });
    }
    pub fn present_chat_targets(
        &mut self,
        room_name: &str,
        room_description: &str,
        characters: &[String],
    ) -> StartChatChoice {
        let mut menu = characters
            .iter()
            .enumerate()
            .map(|(i, c)| MenuItem(c.into(), UIChoice::StartChat(StartChatChoice::TalkTo(i))))
            .collect::<Vec<_>>();
        menu.push(MenuItem(
            self.language.cancel_chat().into(),
            UIChoice::StartChat(StartChatChoice::NoOne),
        ));
        self.show_menu(MenuScreen {
            title: room_name.into(),
            prompt: self.language.choose_chat().into(),
            body: room_description.into(),
            menu,
        });
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
        let mut menu = responses
            .iter()
            .enumerate()
            .map(|(i, c)| MenuItem(c.into(), UIChoice::InChat(ChatChoice::RespondWith(i))))
            .collect::<Vec<_>>();
        if responses.is_empty() {
            menu.push(MenuItem(
                self.language.cancel_response().into(),
                UIChoice::InChat(ChatChoice::Leave),
            ));
        }
        self.show_menu(MenuScreen {
            title: character_name.into(),
            prompt: self.language.choose_response().into(),
            body: dialogue.into(),
            menu,
        });
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
        let mut menu = actions
            .iter()
            .enumerate()
            .map(|(i, c)| {
                MenuItem(
                    c.to_string().to_case(convert_case::Case::Title),
                    UIChoice::Interact(InteractionChoice::Do(i)),
                )
            })
            .collect::<Vec<_>>();
        menu.push(MenuItem(
            self.language.cancel_action().into(),
            UIChoice::Interact(InteractionChoice::Nothing),
        ));
        self.show_menu(MenuScreen {
            title: room_name.into(),
            prompt: self.language.choose_action().into(),
            body: room_description.into(),
            menu,
        });
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

    pub fn present_action(&mut self, action_name: &str, action_description: &str, success: bool) {
        let description = if success {
            action_description
        } else {
            "It didn't work."
        };
        self.show_menu(MenuScreen {
            title: action_name.to_string().to_case(convert_case::Case::Title),
            prompt: "".into(),
            body: description.into(),
            menu: vec![MenuItem(
                self.language.continue_game().into(),
                UIChoice::None,
            )],
        });
    }

    pub fn present_exit_select(
        &mut self,
        room_name: &str,
        room_description: &str,
        exits: &[String],
    ) -> LeaveChoice {
        let mut menu = exits
            .iter()
            .enumerate()
            .map(|(i, c)| {
                MenuItem(
                    c.to_string().to_case(convert_case::Case::Title),
                    UIChoice::Leave(LeaveChoice::GoTo(i)),
                )
            })
            .collect::<Vec<_>>();
        menu.push(MenuItem(
            self.language.cancel_exit().into(),
            UIChoice::Leave(LeaveChoice::Stay),
        ));
        self.show_menu(MenuScreen {
            title: room_name.into(),
            prompt: self.language.choose_exit().into(),
            body: room_description.into(),
            menu,
        });
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
    fn swap_layer<T>(&mut self, layer: T)
    where
        T: IntoBoxedView,
    {
        self.siv.add_fullscreen_layer(layer);
        if self.siv.screen().len() > 1 {
            self.siv
                .screen_mut()
                .remove_layer(LayerPosition::FromBack(0));
        }
    }
    fn show_menu(&mut self, screen: MenuScreen) {
        if self.screen.title != screen.title {
            self.siv.call_on_name("title", |v: &mut views::TextView| {
                let mut styled = StyledString::new();
                styled.append_styled(
                    screen.title.as_str(),
                    Style::from(Color::from(self.theme.heading())).combine(Effect::Bold),
                );
                v.set_content(styled);
            });
        }
        if self.screen.body != screen.body {
            self.siv.call_on_name("body", |v: &mut views::TextView| {
                let mut styled = StyledString::new();
                styled.append_plain(screen.body.as_str());
                v.set_content(styled);
            });
        }
        if self.screen.prompt != screen.prompt {
            self.siv.call_on_name("prompt", |v: &mut views::TextView| {
                let mut styled = StyledString::new();
                styled.append_plain(screen.prompt.as_str());
                v.set_content(styled);
            });
        }
        self.siv
            .call_on_name("menu", |v: &mut views::SelectView<UIChoice>| {
                v.clear();
                for MenuItem(text, value) in &screen.menu {
                    v.add_item(text.as_str(), value.clone());
                }
            });
        self.screen = screen;
        self.siv.run();
    }
    fn switch_to_menu_screen(&mut self) {
        let title = TextView::new(StyledString::new())
            .h_align(HAlign::Center)
            .with_name("title");
        let body = TextView::new(StyledString::new())
            .h_align(HAlign::Left)
            .with_name("body");
        let body = ScrollView::new(body);
        let prompt = TextView::new(StyledString::new())
            .h_align(HAlign::Left)
            .with_name("prompt");
        let mut menu = SelectView::<UIChoice>::new();
        menu.set_on_submit(|siv, selected| {
            siv.with_user_data(|data: &mut UIState| {
                data.choice = selected.clone();
            });
            siv.quit();
        });
        let menu = menu.with_name("menu");
        let mut notice = StyledString::new();
        notice.append_styled(
            self.language.press_q_to_quit(),
            Style::from(Color::from(self.theme.subdued())),
        );
        let notice = TextView::new(notice).h_align(HAlign::Center);
        let layout = LinearLayout::vertical()
            .child(DummyView.full_height())
            .weight(1)
            .child(title)
            .child(DummyView.fixed_height(1))
            .child(body)
            .child(DummyView.fixed_height(1))
            .child(prompt)
            .child(menu)
            .child(DummyView.full_height())
            .weight(1)
            .child(notice)
            .child(DummyView.fixed_height(1));
        let layout = LinearLayout::horizontal()
            .child(DummyView.full_width())
            .weight(1)
            .child(layout)
            .child(DummyView.full_width())
            .weight(1);
        self.swap_layer(layout);
    }
    fn pause_for_any_key_view(&self) -> Button {
        let text = format!("[ {} ]", self.language.continue_game());
        Button::new_raw(text, |s| s.quit())
    }
}
