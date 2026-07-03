use std::{array::from_fn, time::Duration};

use iced::{
    Element, Function, Subscription,
    time::{self, Instant, milliseconds},
    widget::{self, container, grid, responsive},
};
use log::info;

use crate::{
    BOARD_PADDING, CARD_ASPECT, GRID_SPACING,
    card::{self, CardCanvas, ClassicCard, check_if_has_set},
    selection::Selection,
};

#[derive(Debug, Clone)]
pub enum Message {
    Card(u8, card::Message),
    Restart,
    Exit,
    Tick(Instant),
}

pub struct TimedSet<Deck: Iterator<Item = ClassicCard> + Default> {
    cards: [CardCanvas<ClassicCard>; 12],
    all_cards: Deck,
    selection: Selection,
    finished: bool,
    start_time: Instant,
    remaining_time: Duration,
    sets: usize,
}

impl<Deck: Iterator<Item = ClassicCard> + Default> TimedSet<Deck> {
    pub fn new() -> Self {
        let mut all_cards = Deck::default();
        let mut cards = from_fn(|_| CardCanvas::new(all_cards.next().unwrap()));
        while !check_if_has_set(&cards) {
            cards[0] = CardCanvas::new(all_cards.next().unwrap());
        }

        fastrand::shuffle(&mut cards);

        Self {
            cards,
            all_cards,
            selection: Selection::new(12),
            finished: false,
            start_time: Instant::now(),
            remaining_time: Duration::from_secs(60),
            sets: 0,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Card(card, card::Message::Toggle) => self.toggle_card(card),
            Message::Restart => *self = Self::new(),
            Message::Exit => (),
            Message::Tick(now) => {
                let passed_time = now - self.start_time;
                if passed_time >= Duration::from_secs(60) {
                    self.remaining_time = Duration::ZERO;
                    self.finished = true;
                } else {
                    self.remaining_time = Duration::from_secs(60) - passed_time;
                }
            }
        }
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

            grid(self.cards.iter().enumerate().map(|(i, card)| {
                card.view(self.selection.is_selected(i as u8))
                    .map(Message::Card.with(i as u8))
            }))
            .columns(4)
            .spacing(GRID_SPACING)
            .width(size.width.min(expected_width))
            .height(grid::Sizing::AspectRatio(CARD_ASPECT))
        }))
        .padding(BOARD_PADDING);

        widget::column![bar, grid].into()
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
            self.selection
                .zip(self.all_cards.by_ref().take(3))
                .for_each(|(card_idx, card)| self.cards[card_idx as usize].set_card(card));

            while !check_if_has_set(&self.cards) {
                self.cards[self.selection.nth(fastrand::usize(0..3)).unwrap() as usize] =
                    CardCanvas::new(self.all_cards.next().unwrap());
            }
        }

        self.selection.clear();
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.finished {
            Subscription::none()
        } else {
            time::every(milliseconds(100)).map(Message::Tick)
        }
    }
}

impl<Deck: Iterator<Item = ClassicCard> + Default> Default for TimedSet<Deck> {
    fn default() -> Self {
        Self::new()
    }
}
