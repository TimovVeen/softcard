use iced::{
    Element, Subscription,
    widget::{self},
};

mod card;
mod projective;
mod selection;
mod set;
use crate::{projective::ProjSet, set::ClassicSet};

pub const BOARD_PADDING: f32 = 20.;
pub const GRID_SPACING: f32 = 20.;
pub const CARD_ASPECT: f32 = 2. / 3.;

#[derive(Debug, Default, Clone)]
enum Screen {
    #[default]
    Menu,
    ProjSet(ProjSet),
    ClassicSet(ClassicSet),
}

#[derive(Debug, Clone)]
enum Message {
    ChangeScreen(Screen),
    ProjSet(projective::Message),
    ClassicSet(set::Message),
}

#[derive(Default)]
struct App {
    screen: Screen,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::ChangeScreen(screen) => self.screen = screen,
            Message::ProjSet(projective::Message::Exit)
            | Message::ClassicSet(set::Message::Exit) => self.screen = Screen::Menu,
            Message::ProjSet(message) if let Screen::ProjSet(projset) = &mut self.screen => {
                projset.update(message)
            }
            Message::ClassicSet(message)
                if let Screen::ClassicSet(classicset) = &mut self.screen =>
            {
                classicset.update(message)
            }
            _ => (),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Menu => widget::column![
                widget::button("Projective Set")
                    .on_press(Message::ChangeScreen(Screen::ProjSet(ProjSet::default()))),
                widget::button("Classic Set").on_press(Message::ChangeScreen(Screen::ClassicSet(
                    ClassicSet::default()
                ))),
            ]
            .spacing(5.)
            .into(),
            Screen::ProjSet(projset) => projset.view().map(Message::ProjSet),
            Screen::ClassicSet(classicset) => classicset.view().map(Message::ClassicSet),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.screen {
            Screen::Menu => Subscription::none(),
            Screen::ProjSet(projset) => projset.subscription().map(Message::ProjSet),
            Screen::ClassicSet(classicset) => classicset.subscription().map(Message::ClassicSet),
        }
    }
}

fn main() -> iced::Result {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    iced::application(App::default, App::update, App::view)
        .title("Softcard")
        .subscription(App::subscription)
        .run()
}
