use crate::selection::Sel;
use std::{array::from_fn, iter::Sum};

use iced::{
    Function, Subscription, Task, keyboard,
    time::{self, Duration, Instant, milliseconds},
    widget::{self, canvas::Cache, container, grid, responsive},
};
use log::info;

use crate::{
    BOARD_PADDING, CARD_ASPECT, GRID_SPACING,
    cards::card::{self, CardDraw},
    gui::Element,
};

#[derive(Debug, Clone)]
pub enum Message {
    Card(u8, card::Message),
    KeyboardEvent(keyboard::Event),
    Restart,
    Exit,
    Finished(Duration),
    Tick(Instant),
}

pub struct FixedSet<Deck, Card, Selection, const BOARD_SIZE: usize, const DECK_SIZE: usize>
where
    Deck: Iterator<Item = Card> + Default,
    Card: CardDraw + Copy + Sum + Default + Eq,
    Selection: Sel,
{
    cards: [Card; BOARD_SIZE],
    caches: [Cache; BOARD_SIZE],
    all_cards: Deck,
    selection: Selection,
    card_head: usize,
    finished: bool,
    start_time: Instant,
    current_time: Instant,
}

impl<Deck, Card, Selection, const BOARD_SIZE: usize, const DECK_SIZE: usize>
    FixedSet<Deck, Card, Selection, BOARD_SIZE, DECK_SIZE>
where
    Deck: Iterator<Item = Card> + Default,
    Card: CardDraw + Copy + Sum + Default + Eq,
    Selection: Sel,
{
    pub fn new() -> Self {
        let mut all_cards = Deck::default();

        Self {
            cards: from_fn(|_| all_cards.next().unwrap()),
            caches: from_fn(|_| Default::default()),
            all_cards,
            selection: Selection::empty(),
            card_head: BOARD_SIZE,
            finished: false,
            start_time: Instant::now(),
            current_time: Instant::now(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Card(card, card::Message::Toggle) => {
                self.toggle_card(card);
                if self.finished {
                    return Task::done(Message::Finished(self.current_time - self.start_time));
                }
            }
            Message::KeyboardEvent(event) => self.handle_keyboard_event(event),
            Message::Restart => *self = Self::new(),
            Message::Exit | Message::Finished(_) => (),
            Message::Tick(now) => self.current_time = now,
        }
        Task::none()
    }

    pub fn view(&self, columns: usize) -> Element<'_, Message> {
        let elapsed_time = (self.current_time - self.start_time).as_secs();
        let seconds = elapsed_time % 60;
        let minutes = elapsed_time / 60;
        let bar = widget::row![
            widget::button("Restart").on_press(Message::Restart),
            widget::button("Menu").on_press(Message::Exit),
            widget::text!("Remaining cards: {}", DECK_SIZE - self.card_head),
            widget::text!("Time: {:02}:{:02}", minutes, seconds),
        ]
        .spacing(5.)
        .padding(5.);

        let grid =
            container(responsive(move |size| {
                let rows = BOARD_SIZE.div_ceil(columns);
                let expected_width = (size.height - (rows - 1) as f32 * GRID_SPACING) / rows as f32
                    * CARD_ASPECT
                    * columns as f32
                    + (columns - 1) as f32 * GRID_SPACING;

                grid(self.cards.iter().zip(self.caches.iter()).enumerate().map(
                    |(i, (card, cache))| {
                        card.view(cache, self.selection.is_selected(i as u8))
                            .map(Message::Card.with(i as u8))
                    },
                ))
                .columns(columns)
                .spacing(GRID_SPACING)
                .width(size.width.min(expected_width))
                .height(grid::Sizing::AspectRatio(CARD_ASPECT))
            }))
            .padding(BOARD_PADDING);

        widget::column![bar, grid].into()
    }

    fn handle_keyboard_event(&mut self, event: keyboard::Event) {
        if let keyboard::Event::KeyPressed { key, repeat, .. } = event
            && !repeat
        {
            match key.as_ref() {
                keyboard::Key::Character("c") => self.selection.clear(),
                keyboard::Key::Character("x") => {
                    self.selection.invert();
                    self.resolve_selection();
                }
                keyboard::Key::Character(ch)
                    if let Ok(num) = ch.parse::<u8>()
                        && (1..=7).contains(&num) =>
                {
                    self.toggle_card(num - 1);
                }
                _ => {}
            }
        }
    }

    fn toggle_card(&mut self, card: u8) {
        if self.finished || card >= self.cards.len() as u8 {
            return;
        }
        self.selection.toggle(card);
        self.resolve_selection();
    }

    fn resolve_selection(&mut self) {
        if self.selection.is_empty() || !self.selection.check_set(&self.cards) {
            return;
        }

        info!("You got a set!");
        if self.selection.len() + self.card_head >= DECK_SIZE {
            self.finished = true;
            self.selection.clear();
            info!("You win!");
            return;
        }
        self.selection
            .iter()
            .zip(self.all_cards.by_ref().take(self.selection.len()))
            .for_each(|(card_idx, card)| {
                self.cards[card_idx] = card;
                self.caches[card_idx].clear();
                self.card_head += 1;
            });
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

impl<Deck, Card, Selection, const BOARD_SIZE: usize, const DECK_SIZE: usize> Default
    for FixedSet<Deck, Card, Selection, BOARD_SIZE, DECK_SIZE>
where
    Deck: Iterator<Item = Card> + Default,
    Card: CardDraw + Copy + Sum + Default + Eq,
    Selection: Sel,
{
    fn default() -> Self {
        Self::new()
    }
}
