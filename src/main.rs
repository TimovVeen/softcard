use iced::{
    Element, Subscription,
    widget::{self},
};

mod card;
mod projective;
mod selection;
use crate::projective::ProjSet;

#[derive(Debug, Default, Clone)]
enum Screen {
    #[default]
    Menu,
    ProjSet(ProjSet),
}

#[derive(Debug, Clone)]
enum Message {
    ChangeScreen(Screen),
    ProjSet(projective::Message),
}

#[derive(Default)]
struct App {
    screen: Screen,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::ChangeScreen(screen) => self.screen = screen,
            Message::ProjSet(projective::Message::Exit) => self.screen = Screen::Menu,
            Message::ProjSet(message) if let Screen::ProjSet(projset) = &mut self.screen => {
                projset.update(message)
            }
            _ => (),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Menu => widget::column![
                widget::button("Projective Set")
                    .on_press(Message::ChangeScreen(Screen::ProjSet(ProjSet::default()))),
                widget::button("Classic Set"),
            ]
            .spacing(5.)
            .into(),
            Screen::ProjSet(projset) => projset.view().map(Message::ProjSet),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.screen {
            Screen::Menu => Subscription::none(),
            Screen::ProjSet(projset) => projset.subscription().map(Message::ProjSet),
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
