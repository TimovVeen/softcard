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
    cards: Vec<CardCanvas<ClassicCard>>,
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

        let mut initial_count = 12;
        let mut cards: Vec<_> = all_cards
            .iter()
            .take(initial_count)
            .map(CardCanvas::new)
            .collect();
        while !check_if_has_set(&cards) {
            info!("no set");
            initial_count += 3;
            cards = all_cards
                .iter()
                .take(initial_count)
                .map(CardCanvas::new)
                .collect();
        }

        fastrand::shuffle(&mut cards);

        Self {
            cards,
            all_cards,
            selection: Selection::new(initial_count as u8),
            card_head: initial_count,
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
        .spacing(5.)
        .padding(5.);

        let grid = container(responsive(|size| {
            let columns = self.cards.len() / 3;
            let expected_width =
                (size.height - GRID_SPACING * 2.) / 3. * CARD_ASPECT * columns as f32
                    + (columns - 1) as f32 * GRID_SPACING;

            grid(self.cards.iter().enumerate().map(|(i, card)| {
                card.view(self.selection.is_selected(i as u8))
                    .map(Message::Card.with(i as u8))
            }))
            .columns(columns)
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
        if self.is_selected_set() {
            info!("You got a set!");
            if self.cards.len() == 12 && self.card_head < self.all_cards.len() {
                for card_idx in self.selection {
                    self.cards[card_idx as usize].set_card(self.all_cards[self.card_head]);
                    self.card_head += 1;
                }
            } else {
                for &card_idx in self.selection.into_iter().collect::<Vec<_>>().iter().rev() {
                    self.cards.remove(card_idx as usize);
                }
            }

            while !check_if_has_set(&self.cards) && self.card_head < self.all_cards.len() {
                self.cards
                    .push(CardCanvas::new(&self.all_cards[self.card_head]));
                self.cards
                    .push(CardCanvas::new(&self.all_cards[self.card_head + 1]));
                self.cards
                    .push(CardCanvas::new(&self.all_cards[self.card_head + 2]));
                self.card_head += 3;
            }

            if !check_if_has_set(&self.cards) {
                self.finished = true;
                info!("You win!");
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

fn check_if_has_set(cards: &[CardCanvas<ClassicCard>]) -> bool {
    let len = cards.len();
    for i in 0..len {
        for j in (i + 1)..len {
            for k in (j + 1)..len {
                if cards[i].get_card() + cards[j].get_card() + cards[k].get_card()
                    == ClassicCard::default()
                {
                    return true;
                }
            }
        }
    }
    false
}

impl Default for ClassicSet {
    fn default() -> Self {
        Self::new()
    }
}
