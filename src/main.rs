use iced::{Length, Subscription, Task, widget};
use log::error;
use serde::{Deserialize, Serialize};

mod cards;
mod decks;
mod games;
mod gui;
mod selection;
mod userdata;
use crate::{
    cards::{projective::ProjCard, set::ClassicCard, symmetric::SymCard, wreath::WreathCard},
    decks::{RandomDeck, ShuffleDeck},
    games::{
        fixed::{self, FixedSet},
        set::{self, ClassicSet},
        timed::{self, TimedSet},
    },
    gui::Element,
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
    ProjSet(FixedSet<ShuffleDeck<ProjCard>, selection::Unordered, 7, 63>),
    ClassicSet(ClassicSet<ShuffleDeck<ClassicCard>>),
    ClassicProj(ClassicSet<ShuffleDeck<ProjCard>>),
    TimedSet(TimedSet<RandomDeck<ClassicCard>>),
    TimedProj(TimedSet<RandomDeck<ProjCard>>),
    SymSet(FixedSet<ShuffleDeck<SymCard<4>>, selection::Ordered, 7, 23>),
    WreathSet(FixedSet<ShuffleDeck<WreathCard<3>>, selection::Ordered, 6, 47>),
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
enum Screen {
    Menu,
    ProjSet,
    ClassicSet,
    ClassicProj,
    TimedSet,
    TimedProj,
    SymSet,
    WreathSet,
}

impl From<Screen> for State {
    fn from(screen: Screen) -> Self {
        match screen {
            Screen::Menu => Self::Menu,
            Screen::ProjSet => Self::ProjSet(FixedSet::new()),
            Screen::ClassicSet => Self::ClassicSet(ClassicSet::new()),
            Screen::ClassicProj => Self::ClassicProj(ClassicSet::new()),
            Screen::TimedSet => Self::TimedSet(TimedSet::new()),
            Screen::TimedProj => Self::TimedProj(TimedSet::new()),
            Screen::SymSet => Self::SymSet(FixedSet::new()),
            Screen::WreathSet => Self::WreathSet(FixedSet::new()),
        }
    }
}

impl From<&State> for Screen {
    fn from(state: &State) -> Self {
        match state {
            State::Menu => Self::Menu,
            State::ProjSet(_) => Self::ProjSet,
            State::ClassicSet(_) => Self::ClassicSet,
            State::ClassicProj(_) => Self::ClassicProj,
            State::TimedSet(_) => Self::TimedSet,
            State::TimedProj(_) => Self::TimedProj,
            State::SymSet(_) => Self::SymSet,
            State::WreathSet(_) => Self::WreathSet,
        }
    }
}

#[derive(Clone)]
enum Message {
    ChangeScreen(Screen),
    Error(Result<(), String>),
    UserDataRead(Option<UserData>),
    ProjSet(fixed::Message),
    ClassicSet(set::Message),
    ClassicProj(set::Message),
    TimedSet(timed::Message),
    SymSet(fixed::Message),
    WreathSet(fixed::Message),
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
            Message::ProjSet(fixed::Message::Exit)
            | Message::ClassicSet(set::Message::Exit)
            | Message::ClassicProj(set::Message::Exit)
            | Message::TimedSet(timed::Message::Exit)
            | Message::SymSet(fixed::Message::Exit)
            | Message::WreathSet(fixed::Message::Exit) => self.state = State::Menu,
            Message::ProjSet(fixed::Message::Finished(time))
            | Message::ClassicSet(set::Message::Finished(time))
            | Message::ClassicProj(set::Message::Finished(time))
            | Message::SymSet(fixed::Message::Finished(time))
            | Message::WreathSet(fixed::Message::Finished(time)) => {
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
            Message::ClassicProj(message)
                if let State::ClassicProj(classicproj) = &mut self.state =>
            {
                return classicproj.update(message).map(Message::ClassicProj);
            }
            Message::TimedSet(message) if let State::TimedSet(timedset) = &mut self.state => {
                return timedset.update(message).map(Message::TimedSet);
            }
            Message::TimedSet(message) if let State::TimedProj(timedproj) = &mut self.state => {
                return timedproj.update(message).map(Message::TimedSet);
            }
            Message::SymSet(message) if let State::SymSet(symset) = &mut self.state => {
                return symset.update(message).map(Message::SymSet);
            }
            Message::WreathSet(message) if let State::WreathSet(wreathset) = &mut self.state => {
                return wreathset.update(message).map(Message::WreathSet);
            }
            _ => (),
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Menu => self.view_menu(),
            State::ProjSet(projset) => projset.view(4).map(Message::ProjSet),
            State::ClassicSet(classicset) => classicset.view().map(Message::ClassicSet),
            State::ClassicProj(classicproj) => classicproj.view().map(Message::ClassicProj),
            State::TimedSet(timedset) => timedset.view().map(Message::TimedSet),
            State::TimedProj(timedproj) => timedproj.view().map(Message::TimedSet),
            State::SymSet(symset) => symset.view(7).map(Message::SymSet),
            State::WreathSet(wreathset) => wreathset.view(6).map(Message::WreathSet),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.state {
            State::Menu => Subscription::none(),
            State::ProjSet(projset) => projset.subscription().map(Message::ProjSet),
            State::ClassicSet(classicset) => classicset.subscription().map(Message::ClassicSet),
            State::ClassicProj(classicproj) => classicproj.subscription().map(Message::ClassicProj),
            State::TimedSet(timedset) => timedset.subscription().map(Message::TimedSet),
            State::TimedProj(timedproj) => timedproj.subscription().map(Message::TimedSet),
            State::SymSet(symset) => symset.subscription().map(Message::SymSet),
            State::WreathSet(wreathset) => wreathset.subscription().map(Message::WreathSet),
        }
    }

    fn view_menu(&self) -> Element<'_, Message> {
        widget::column![
            widget::text!["Select Game:"],
            widget::button(widget::text!(
                "Projective Set\nBest time: {}",
                self.userdata
                    .best_times
                    .get(&Screen::ProjSet)
                    .map_or_else(|| "None".to_string(), |time| format!("{}s", time.as_secs()))
            ))
            .on_press(Message::ChangeScreen(Screen::ProjSet))
            .width(Length::Fixed(160.)),
            widget::button(widget::text!(
                "Classic Set\nBest time: {}",
                self.userdata
                    .best_times
                    .get(&Screen::ClassicSet)
                    .map_or_else(|| "None".to_string(), |time| format!("{}s", time.as_secs()))
            ))
            .on_press(Message::ChangeScreen(Screen::ClassicSet))
            .width(Length::Fixed(160.)),
            widget::button(widget::text!(
                "Classic Projective\nBest time: {}",
                self.userdata
                    .best_times
                    .get(&Screen::ClassicProj)
                    .map_or_else(|| "None".to_string(), |time| format!("{}s", time.as_secs()))
            ))
            .on_press(Message::ChangeScreen(Screen::ClassicProj))
            .width(Length::Fixed(160.)),
            widget::button(widget::text!(
                "Timed Set\nMost cards: {}",
                self.userdata
                    .best_cards
                    .get(&Screen::TimedSet)
                    .map_or_else(|| "None".to_string(), ToString::to_string)
            ))
            .on_press(Message::ChangeScreen(Screen::TimedSet))
            .width(Length::Fixed(160.)),
            widget::button(widget::text!(
                "Timed Projective\nMost cards: {}",
                self.userdata
                    .best_cards
                    .get(&Screen::TimedProj)
                    .map_or_else(|| "None".to_string(), ToString::to_string)
            ))
            .on_press(Message::ChangeScreen(Screen::TimedProj))
            .width(Length::Fixed(160.)),
            widget::button(widget::text!(
                "Permutation Set\nBest time: {}",
                self.userdata
                    .best_times
                    .get(&Screen::SymSet)
                    .map_or_else(|| "None".to_string(), |time| format!("{}s", time.as_secs()))
            ))
            .on_press(Message::ChangeScreen(Screen::SymSet))
            .width(Length::Fixed(160.)),
            widget::button(widget::text!(
                "Wreath Set\nBest time: {}",
                self.userdata
                    .best_times
                    .get(&Screen::WreathSet)
                    .map_or_else(|| "None".to_string(), |time| format!("{}s", time.as_secs()))
            ))
            .on_press(Message::ChangeScreen(Screen::WreathSet))
            .width(Length::Fixed(160.)),
        ]
        .spacing(5.)
        .padding(20.)
        .into()
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
