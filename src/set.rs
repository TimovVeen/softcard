use std::array::from_fn;

use iced::{
    Element, Function, Subscription,
    time::{self, Instant, milliseconds},
    widget::{self, container, grid, responsive},
};
use log::info;

use crate::{
    BOARD_PADDING, CARD_ASPECT, GRID_SPACING,
    card::{self, CardCanvas, ClassicCard},
    selection::Selection,
};

#[derive(Debug, Clone)]
pub enum Message {
    Card(u8, card::Message),
    Restart,
    Exit,
    Tick(Instant),
}

pub struct ClassicSet {
    cards: [CardCanvas<ClassicCard>; 12],
    all_cards: [ClassicCard; 81],
    selection: Selection,
    card_head: usize,
    finished: bool,
    start_time: Instant,
    current_time: Instant,
}

impl ClassicSet {
    pub fn new() -> Self {
        let mut all_cards = {
            let mut res = [ClassicCard::default(); 81];
            let mut i = 0;
            // find a better way to do this
            for j in 0..=2 {
                for k in 0..=2 {
                    for l in 0..=2 {
                        for m in 0..=2 {
                            res[i] = ClassicCard::new([j, k, l, m]);
                            i += 1;
                        }
                    }
                }
            }
            res
        };
        fastrand::shuffle(&mut all_cards);

        Self {
            cards: from_fn(|i| CardCanvas::new(all_cards[i])),
            all_cards,
            selection: Selection::new(12),
            card_head: 12,
            finished: false,
            start_time: Instant::now(),
            current_time: Instant::now(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Card(card, card::Message::Toggle) => self.toggle_card(card),
            Message::Restart => *self = Self::new(),
            Message::Exit => (),
            Message::Tick(now) => self.current_time = now,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let elapsed_time = (self.current_time - self.start_time).as_millis();
        let millis = elapsed_time % 1000;
        let seconds = (elapsed_time / 1000) % 60;
        let minutes = elapsed_time / 60000;
        let bar = widget::row![
            widget::button("Restart").on_press(Message::Restart),
            widget::button("Menu").on_press(Message::Exit),
            widget::text!("Remaining cards: {}", 81 - self.card_head),
            widget::text!("Time: {:02}:{:02}", minutes, seconds),
        ]
        .spacing(5.);

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
        if self.selection.card_count() == 3 {
            self.resolve_selection();
        }
    }

    fn resolve_selection(&mut self) {
        if !self.selection.is_empty() && self.is_selected_set() {
            info!("You got a set!");
            for card in self.selection {
                if self.card_head >= self.all_cards.len() {
                    self.finished = true;
                    self.selection.clear();
                    info!("You win!");
                    return;
                }

                self.cards[card as usize].set_card(self.all_cards[self.card_head]);
                self.card_head += 1;
            }
        }

        self.selection.clear();
    }

    fn is_selected_set(&self) -> bool {
        self.selection
            .into_iter()
            .map(|i| self.cards[i as usize].get_card())
            .sum::<ClassicCard>()
            == ClassicCard::default()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if !self.finished {
            time::every(milliseconds(100)).map(Message::Tick)
        } else {
            Subscription::none()
        }
    }
}

impl Default for ClassicSet {
    fn default() -> Self {
        Self::new()
    }
}
