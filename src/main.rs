use iced::{
    Element, Length, Subscription,
    widget::{self},
};

mod card;
mod projective;
mod selection;
mod set;
mod timed;
use crate::{
    card::{ClassicDeck, ProjDeck},
    projective::ProjSet,
    set::ClassicSet,
    timed::TimedSet,
};

pub const BOARD_PADDING: f32 = 20.;
pub const GRID_SPACING: f32 = 20.;
pub const CARD_ASPECT: f32 = 2. / 3.;

#[derive(Default)]
enum State {
    #[default]
    Menu,
    ProjSet(ProjSet<ProjDeck>),
    ClassicSet(ClassicSet<ClassicDeck>),
    TimedSet(TimedSet<ClassicDeck>),
}

#[derive(Clone)]
enum Screen {
    Menu,
    ProjSet,
    ClassicSet,
    TimedSet,
}

#[derive(Clone)]
enum Message {
    ChangeScreen(Screen),
    ProjSet(projective::Message),
    ClassicSet(set::Message),
    TimedSet(timed::Message),
}

#[derive(Default)]
struct App {
    state: State,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::ChangeScreen(Screen::ProjSet) => {
                self.state = State::ProjSet(ProjSet::default());
            }
            Message::ChangeScreen(Screen::ClassicSet) => {
                self.state = State::ClassicSet(ClassicSet::default());
            }
            Message::ChangeScreen(Screen::TimedSet) => {
                self.state = State::TimedSet(TimedSet::default());
            }
            Message::ProjSet(projective::Message::Exit)
            | Message::ClassicSet(set::Message::Exit)
            | Message::TimedSet(timed::Message::Exit) => self.state = State::Menu,
            Message::ProjSet(message) if let State::ProjSet(projset) = &mut self.state => {
                projset.update(message);
            }
            Message::ClassicSet(message) if let State::ClassicSet(classicset) = &mut self.state => {
                classicset.update(message);
            }
            Message::TimedSet(message) if let State::TimedSet(timedset) = &mut self.state => {
                timedset.update(message);
            }
            _ => (),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Menu => widget::column![
                widget::text!["Select Game:"],
                widget::button("Projective Set")
                    .on_press(Message::ChangeScreen(Screen::ProjSet))
                    .width(Length::Fixed(160.)),
                widget::button("Classic Set")
                    .on_press(Message::ChangeScreen(Screen::ClassicSet))
                    .width(Length::Fixed(160.)),
                widget::button("Timed Set")
                    .on_press(Message::ChangeScreen(Screen::TimedSet))
                    .width(Length::Fixed(160.)),
            ]
            .spacing(5.)
            .padding(20.)
            .into(),
            State::ProjSet(projset) => projset.view().map(Message::ProjSet),
            State::ClassicSet(classicset) => classicset.view().map(Message::ClassicSet),
            State::TimedSet(timedset) => timedset.view().map(Message::TimedSet),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.state {
            State::Menu => Subscription::none(),
            State::ProjSet(projset) => projset.subscription().map(Message::ProjSet),
            State::ClassicSet(classicset) => classicset.subscription().map(Message::ClassicSet),
            State::TimedSet(timedset) => timedset.subscription().map(Message::TimedSet),
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
