use std::{
    array::from_fn,
    iter::{Cycle, Sum},
    ops::Add,
    time::Duration,
};

use iced::{
    Element, Function, Subscription,
    time::{self, Instant, milliseconds},
    widget::{self, container, grid, responsive},
};
use log::info;

use crate::{
    BOARD_PADDING, CARD_ASPECT, GRID_SPACING,
    card::{self, CardCanvas, CardDraw, check_if_has_set},
    selection::Selection,
};

#[derive(Debug, Clone)]
pub enum Message {
    Card(u8, card::Message),
    Restart,
    Exit,
    Tick(Instant),
}

pub struct TimedSet<
    Card: CardDraw + Copy + Sum + Default + Eq + Add,
    Deck: Iterator<Item = Card> + Default + Clone,
> {
    cards: [CardCanvas<Card>; 12],
    all_cards: Cycle<Deck>,
    selection: Selection,
    pub finished: bool,
    start_time: Instant,
    remaining_time: Duration,
    pub sets: usize,
}

impl<
    Card: CardDraw + Copy + Sum + Default + Eq + Add,
    Deck: Iterator<Item = Card> + Default + Clone,
> TimedSet<Card, Deck>
{
    pub fn new() -> Self {
        let mut all_cards = Deck::default().cycle();
        let mut cards = from_fn(|_| CardCanvas::new(all_cards.next().unwrap()));
        while !check_if_has_set(&cards) {
            cards[0] = CardCanvas::new(all_cards.find(|&x| !has_card(&cards, x)).unwrap());
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
            let new_cards: Vec<Card> = self
                .all_cards
                .by_ref()
                .filter(|&x| !has_card(&self.cards, x))
                .take(3)
                .collect();
            self.selection
                .zip(new_cards)
                .for_each(|(card_idx, card)| self.cards[card_idx as usize].set_card(card));

            while !check_if_has_set(&self.cards) {
                self.cards[self
                    .selection
                    .into_iter()
                    .nth(fastrand::usize(0..3))
                    .unwrap() as usize] =
                    CardCanvas::new(self.all_cards.find(|&x| !has_card(&self.cards, x)).unwrap());
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

fn has_card<Card: CardDraw + Copy + Eq>(cards: &[CardCanvas<Card>], card: Card) -> bool {
    cards.iter().any(|cardcanvas| cardcanvas.get_card() == card)
}

impl<
    Card: CardDraw + Copy + Sum + Default + Eq + Add,
    Deck: Iterator<Item = Card> + Default + Clone,
> Default for TimedSet<Card, Deck>
{
    fn default() -> Self {
        Self::new()
    }
}
