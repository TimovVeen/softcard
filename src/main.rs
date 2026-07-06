use std::{collections::HashMap, fs, io, time::Duration};

use directories::ProjectDirs;
use iced::{
    Element, Length, Subscription,
    widget::{self},
};
use serde::{Deserialize, Serialize};

mod card;
mod projective;
mod selection;
mod set;
mod timed;
use crate::{
    card::{ClassicCard, ClassicDeck, ProjCard, ProjDeck},
    projective::ProjSet,
    set::ClassicSet,
    timed::TimedSet,
};

pub const BOARD_PADDING: f32 = 20.;
pub const GRID_SPACING: f32 = 20.;
pub const CARD_ASPECT: f32 = 2. / 3.;

#[derive(Serialize, Deserialize, Debug, Default)]
struct UserData {
    best_times: HashMap<String, u64>,
    best_cards: HashMap<String, u32>,
}

#[derive(Default)]
#[allow(clippy::large_enum_variant)]
enum State {
    #[default]
    Menu,
    ProjSet(ProjSet<ProjDeck>),
    ClassicSet(ClassicSet<ClassicDeck>),
    TimedSet(TimedSet<ClassicCard, ClassicDeck>),
    TimedProj(TimedSet<ProjCard, ProjDeck>),
}

#[derive(Clone)]
enum Screen {
    Menu,
    ProjSet,
    ClassicSet,
    TimedSet,
    TimedProj,
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
    userdata: UserData,
}

impl App {
    fn new() -> Self {
        let proj_dirs = ProjectDirs::from("com", "ItsAPixel", "Softcard").unwrap();
        let data_file = proj_dirs.data_dir().join("score.ron");
        let userdata = if let Ok(data) = fs::read_to_string(data_file) {
            ron::from_str(&data).unwrap_or_default()
        } else {
            UserData::default()
        };
        Self {
            userdata,
            ..Default::default()
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ChangeScreen(Screen::ProjSet) => {
                self.state = State::ProjSet(ProjSet::default());
            }
            Message::ChangeScreen(Screen::ClassicSet) => {
                self.state = State::ClassicSet(ClassicSet::default());
            }
            Message::ChangeScreen(Screen::TimedSet) => {
                self.state = State::TimedSet(TimedSet::<ClassicCard, ClassicDeck>::default());
            }
            Message::ChangeScreen(Screen::TimedProj) => {
                self.state = State::TimedProj(TimedSet::<ProjCard, ProjDeck>::default());
            }
            Message::ProjSet(projective::Message::Exit)
            | Message::ClassicSet(set::Message::Exit)
            | Message::TimedSet(timed::Message::Exit) => self.state = State::Menu,
            Message::ProjSet(message) if let State::ProjSet(projset) = &mut self.state => {
                projset.update(message);
                let final_time = (projset.current_time - projset.start_time).as_millis() as u64;
                if projset.finished
                    && *self.userdata.best_times.get("projset").unwrap_or(&u64::MAX) > final_time
                {
                    self.userdata
                        .best_times
                        .insert("projset".to_string(), final_time);
                    self.update_score().unwrap();
                }
            }
            Message::ClassicSet(message) if let State::ClassicSet(classicset) = &mut self.state => {
                classicset.update(message);
                let final_time =
                    (classicset.current_time - classicset.start_time).as_millis() as u64;
                if classicset.finished
                    && *self
                        .userdata
                        .best_times
                        .get("classicset")
                        .unwrap_or(&u64::MAX)
                        > final_time
                {
                    self.userdata
                        .best_times
                        .insert("classicset".to_string(), final_time);
                    self.update_score().unwrap();
                }
            }
            Message::TimedSet(message) if let State::TimedSet(timedset) = &mut self.state => {
                timedset.update(message);
                if timedset.finished
                    && *self.userdata.best_cards.get("timedset").unwrap_or(&0_u32)
                        < timedset.sets as u32
                {
                    self.userdata
                        .best_cards
                        .insert("timedset".to_string(), timedset.sets as u32);
                    self.update_score().unwrap();
                }
            }
            Message::TimedSet(message) if let State::TimedProj(timedproj) = &mut self.state => {
                timedproj.update(message);
                if timedproj.finished
                    && *self.userdata.best_cards.get("timedproj").unwrap_or(&0_u32)
                        < timedproj.sets as u32
                {
                    self.userdata
                        .best_cards
                        .insert("timedproj".to_string(), timedproj.sets as u32);
                    self.update_score().unwrap();
                }
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
                widget::button("Timed Proj Set")
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

    fn update_score(&self) -> io::Result<()> {
        let proj_dirs = ProjectDirs::from("com", "ItsAPixel", "Softcard").unwrap();
        let data_dir = proj_dirs.data_dir();
        if !data_dir.exists() {
            fs::create_dir_all(data_dir)?;
        }
        let data_file = data_dir.join("score.ron");
        fs::write(
            data_file,
            ron::ser::to_string_pretty(&self.userdata, ron::ser::PrettyConfig::default()).unwrap(),
        )
    }
}

fn main() -> iced::Result {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    iced::application(App::new, App::update, App::view)
        .title("Softcard")
        .subscription(App::subscription)
        .run()
}
