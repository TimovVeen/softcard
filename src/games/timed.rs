use std::{array::from_fn, iter::Sum, ops::Add, time::Duration};

use iced::{
    Function, Subscription, Task,
    keyboard::{self, key::Physical},
    time::{self, Instant, milliseconds},
    widget::{self, canvas::Cache, container, grid, responsive},
};
use log::info;

use crate::{
    BOARD_PADDING, CARD_ASPECT, GRID_SPACING,
    cards::card::{self, CardDraw, check_if_has_set},
    games::set::code_to_grid,
    gui::Element,
    selection::{self, Sel},
};

#[derive(Debug, Clone)]
pub enum Message {
    Card(u8, card::Message),
    KeyboardEvent(keyboard::Event),
    Restart,
    Exit,
    Finished(u32),
    Tick(Instant),
}

pub struct TimedSet<Card, Deck> {
    cards: [Card; 12],
    caches: [Cache; 12],
    all_cards: Deck,
    selection: selection::Unordered,
    finished: bool,
    start_time: Instant,
    remaining_time: Duration,
    sets: u32,
}

impl<Card: CardDraw + Copy + Sum + Default + Eq + Add, Deck: Iterator<Item = Card> + Default>
    TimedSet<Card, Deck>
{
    pub fn new() -> Self {
        let mut all_cards = Deck::default();
        let mut cards = from_fn(|_| all_cards.next().unwrap());
        while !check_if_has_set(&cards) {
            cards[0] = all_cards.find(|x| !cards.contains(x)).unwrap();
        }

        fastrand::shuffle(&mut cards);

        Self {
            cards,
            caches: Default::default(),
            all_cards,
            selection: selection::Unordered::new(12),
            finished: false,
            start_time: Instant::now(),
            remaining_time: Duration::from_secs(60),
            sets: 0,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Card(card, card::Message::Toggle) => self.toggle_card(card),
            Message::KeyboardEvent(event) => self.handle_keyboard_event(event),
            Message::Restart => *self = Self::new(),
            Message::Exit | Message::Finished(_) => (),
            Message::Tick(now) => {
                let passed_time = now - self.start_time;
                if passed_time >= Duration::from_secs(60) {
                    self.remaining_time = Duration::ZERO;
                    self.finished = true;
                    return Task::done(Message::Finished(self.sets));
                } else {
                    self.remaining_time = Duration::from_secs(60) - passed_time;
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let bar = widget::row![
            widget::button("Restart").on_press(Message::Restart),
            widget::button("Menu").on_press(Message::Exit),
            widget::text!("Time: {:02}", self.remaining_time.as_secs()),
            widget::text!("Sets: {}", self.sets),
        ]
        .spacing(5.)
        .padding(5.);

        let grid = container(responsive(|size| {
            let expected_width =
                (size.height - GRID_SPACING * 2.) / 3. * CARD_ASPECT * 4. + 3. * GRID_SPACING;

            grid(
                self.cards
                    .iter()
                    .zip(&self.caches)
                    .enumerate()
                    .map(|(i, (card, cache))| {
                        card.view(cache, self.selection.is_selected(i as u8))
                            .map(Message::Card.with(i as u8))
                    }),
            )
            .columns(4)
            .spacing(GRID_SPACING)
            .width(size.width.min(expected_width))
            .height(grid::Sizing::AspectRatio(CARD_ASPECT))
        }))
        .padding(BOARD_PADDING);

        widget::column![bar, grid].into()
    }

    fn handle_keyboard_event(&mut self, event: keyboard::Event) {
        if let keyboard::Event::KeyPressed {
            physical_key: Physical::Code(code),
            repeat,
            ..
        } = event
            && !repeat
            && let Some((row, col)) = code_to_grid(code)
            && col < 4
            && row < 3
        {
            let idx = row * 4 + col;
            self.toggle_card(idx);
        }
    }

    fn toggle_card(&mut self, card: u8) {
        if self.finished || card >= self.cards.len() as u8 {
            return;
        }
        self.selection.toggle(card);
        if self.selection.len() == 3 {
            self.resolve_selection();
        }
    }

    fn resolve_selection(&mut self) {
        if self.selection.check_set(&self.cards) {
            info!("You got a set!");
            self.sets += 1;
            let new_cards: Vec<Card> = self
                .all_cards
                .by_ref()
                .filter(|x| !self.cards.contains(x))
                .take(3)
                .collect();
            self.selection
                .iter()
                .zip(new_cards)
                .for_each(|(card_idx, card)| {
                    self.caches[card_idx].clear();
                    self.cards[card_idx] = card;
                });

            while !check_if_has_set(&self.cards) {
                self.cards[self.selection.iter().nth(fastrand::usize(0..3)).unwrap() as usize] =
                    self.all_cards.find(|x| !self.cards.contains(x)).unwrap();
            }
        }

        self.selection.clear();
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard = keyboard::listen().map(Message::KeyboardEvent);
        if self.finished {
            keyboard
        } else {
            Subscription::batch(vec![
                keyboard,
                time::every(milliseconds(100)).map(Message::Tick),
            ])
        }
    }
}
