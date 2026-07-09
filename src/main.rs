use iced::{
    Element, Length, Subscription, Task,
    widget::{self},
};
use log::error;
use serde::{Deserialize, Serialize};

mod card;
mod projective;
mod selection;
mod set;
mod timed;
mod userdata;
use crate::{
    card::{ClassicCard, ProjCard, ShuffleDeck},
    projective::ProjSet,
    set::ClassicSet,
    timed::TimedSet,
    userdata::UserData,
};

pub const BOARD_PADDING: f32 = 20.;
pub const GRID_SPACING: f32 = 20.;
pub const CARD_ASPECT: f32 = 2. / 3.;

#[derive(Default)]
#[allow(clippy::large_enum_variant)]
enum State {
    #[default]
    Menu,
    ProjSet(ProjSet<ShuffleDeck<ProjCard>>),
    ClassicSet(ClassicSet<ShuffleDeck<ClassicCard>>),
    TimedSet(TimedSet<ClassicCard, ShuffleDeck<ClassicCard>>),
    TimedProj(TimedSet<ProjCard, ShuffleDeck<ProjCard>>),
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
enum Screen {
    Menu,
    ProjSet,
    ClassicSet,
    TimedSet,
    TimedProj,
}

impl From<Screen> for State {
    fn from(screen: Screen) -> Self {
        match screen {
            Screen::Menu => State::Menu,
            Screen::ProjSet => State::ProjSet(ProjSet::default()),
            Screen::ClassicSet => State::ClassicSet(ClassicSet::default()),
            Screen::TimedSet => State::TimedSet(TimedSet::default()),
            Screen::TimedProj => State::TimedProj(TimedSet::default()),
        }
    }
}

impl From<&State> for Screen {
    fn from(state: &State) -> Self {
        match state {
            State::Menu => Screen::Menu,
            State::ProjSet(_) => Screen::ProjSet,
            State::ClassicSet(_) => Screen::ClassicSet,
            State::TimedSet(_) => Screen::TimedSet,
            State::TimedProj(_) => Screen::TimedProj,
        }
    }
}

#[derive(Clone)]
enum Message {
    ChangeScreen(Screen),
    Error(Result<(), String>),
    UserDataRead(Option<UserData>),
    ProjSet(projective::Message),
    ClassicSet(set::Message),
    TimedSet(timed::Message),
}

#[derive(Default)]
struct App {
    state: State,
    userdata: UserData,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self::default(),
            Task::perform(UserData::try_load(), Message::UserDataRead),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChangeScreen(screen) => self.state = screen.into(),
            Message::UserDataRead(Some(userdata)) => self.userdata = userdata,
            Message::Error(Err(e)) => error!("{e}"),
            Message::ProjSet(projective::Message::Exit)
            | Message::ClassicSet(set::Message::Exit)
            | Message::TimedSet(timed::Message::Exit) => self.state = State::Menu,
            Message::ProjSet(projective::Message::Finished(time))
            | Message::ClassicSet(set::Message::Finished(time)) => {
                return self
                    .userdata
                    .add_time(Screen::from(&self.state), time)
                    .map_err(|e| format!("User data write error: {e}"))
                    .map(Message::Error);
            }
            Message::TimedSet(timed::Message::Finished(cards)) => {
                return self
                    .userdata
                    .add_cards(Screen::from(&self.state), cards)
                    .map_err(|e| format!("User data write error: {e}"))
                    .map(Message::Error);
            }
            Message::ProjSet(message) if let State::ProjSet(projset) = &mut self.state => {
                return projset.update(message).map(Message::ProjSet);
            }
            Message::ClassicSet(message) if let State::ClassicSet(classicset) = &mut self.state => {
                return classicset.update(message).map(Message::ClassicSet);
            }
            Message::TimedSet(message) if let State::TimedSet(timedset) = &mut self.state => {
                return timedset.update(message).map(Message::TimedSet);
            }
            Message::TimedSet(message) if let State::TimedProj(timedproj) = &mut self.state => {
                return timedproj.update(message).map(Message::TimedSet);
            }
            _ => (),
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Menu => widget::column![
                widget::text!["Select Game:"],
                widget::button(widget::text!(
                    "Projective Set\nBest time: {}",
                    self.userdata
                        .best_times
                        .get(&Screen::ProjSet)
                        .map_or("None".to_string(), |time| format!("{}s", time.as_secs()))
                ))
                .on_press(Message::ChangeScreen(Screen::ProjSet))
                .width(Length::Fixed(160.)),
                widget::button(widget::text!(
                    "Classic Set\nBest time: {}",
                    self.userdata
                        .best_times
                        .get(&Screen::ClassicSet)
                        .map_or("None".to_string(), |time| format!("{}s", time.as_secs()))
                ))
                .on_press(Message::ChangeScreen(Screen::ClassicSet))
                .width(Length::Fixed(160.)),
                widget::button(widget::text!(
                    "Timed Set\nMost cards: {}",
                    self.userdata
                        .best_cards
                        .get(&Screen::TimedSet)
                        .map_or("None".to_string(), |cards| cards.to_string())
                ))
                .on_press(Message::ChangeScreen(Screen::TimedSet))
                .width(Length::Fixed(160.)),
                widget::button(widget::text!(
                    "Timed Projective\nMost cards: {}",
                    self.userdata
                        .best_cards
                        .get(&Screen::TimedProj)
                        .map_or("None".to_string(), |cards| cards.to_string())
                ))
                .on_press(Message::ChangeScreen(Screen::TimedProj))
                .width(Length::Fixed(160.)),
            ]
            .spacing(5.)
            .padding(20.)
            .into(),
            State::ProjSet(projset) => projset.view().map(Message::ProjSet),
            State::ClassicSet(classicset) => classicset.view().map(Message::ClassicSet),
            State::TimedSet(timedset) => timedset.view().map(Message::TimedSet),
            State::TimedProj(timedproj) => timedproj.view().map(Message::TimedSet),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.state {
            State::Menu => Subscription::none(),
            State::ProjSet(projset) => projset.subscription().map(Message::ProjSet),
            State::ClassicSet(classicset) => classicset.subscription().map(Message::ClassicSet),
            State::TimedSet(timedset) => timedset.subscription().map(Message::TimedSet),
            State::TimedProj(timedproj) => timedproj.subscription().map(Message::TimedSet),
        }
    }
}

fn main() -> iced::Result {
    #[cfg(debug_assertions)]
    let log_level = log::Level::Debug;
    #[cfg(not(debug_assertions))]
    let log_level = log::Level::Warn;
    simple_logger::init_with_level(log_level).unwrap();

    iced::application(App::new, App::update, App::view)
        .title("Softcard")
        .subscription(App::subscription)
        .run()
}
