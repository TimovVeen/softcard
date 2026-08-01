use iced::{
    Function, Subscription, Task,
    keyboard::{self, key::Code, key::Physical},
    time::{self, Duration, Instant, milliseconds},
    widget::{self, canvas::Cache, container, grid, responsive},
};
use log::info;

use crate::{
    BOARD_PADDING, CARD_ASPECT, GRID_SPACING,
    cards::card::{self, CardDraw, check_if_has_set},
    selection::Sel,
};
use crate::{ClassicCard, gui::Element, selection};

#[derive(Debug, Clone)]
pub enum Message {
    Card(u8, card::Message),
    KeyboardEvent(keyboard::Event),
    Restart,
    Exit,
    Finished(Duration),
    Tick(Instant),
}

pub struct ClassicSet<Deck: Iterator<Item = ClassicCard> + Default> {
    cards: Vec<ClassicCard>,
    caches: [Cache; 21], // forgot what was the actual max
    all_cards: Deck,
    selection: selection::Unordered,
    card_head: usize,
    finished: bool,
    start_time: Instant,
    current_time: Instant,
}

impl<Deck: Iterator<Item = ClassicCard> + Default> ClassicSet<Deck> {
    pub fn new() -> Self {
        let mut all_cards = Deck::default();
        let mut initial_count = 12;
        let mut cards: Vec<_> = all_cards.by_ref().take(initial_count).collect();
        while !check_if_has_set(&cards) {
            info!("no set");
            initial_count += 3;
            cards.extend(all_cards.by_ref().take(3));
        }

        fastrand::shuffle(&mut cards);

        Self {
            cards,
            caches: Default::default(),
            all_cards,
            selection: selection::Unordered::new(initial_count as u8),
            card_head: initial_count,
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

    pub fn view(&self) -> Element<'_, Message> {
        let elapsed_time = (self.current_time - self.start_time).as_secs();
        let seconds = elapsed_time % 60;
        let minutes = elapsed_time / 60;
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
                card.view(&self.caches[i], self.selection.is_selected(i as u8))
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

    fn handle_keyboard_event(&mut self, event: keyboard::Event) {
        if let keyboard::Event::KeyPressed {
            physical_key: Physical::Code(code),
            repeat,
            ..
        } = event
            && !repeat
        {
            let cols = self.cards.len() as u8 / 3;
            if let Some((row, col)) = code_to_grid(code)
                && col < cols
                && row < 3
            {
                let idx = row * cols + col;
                self.toggle_card(idx);
            }
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
            if self.cards.len() == 12 && self.card_head < 81 {
                self.selection
                    .iter()
                    .zip(self.all_cards.by_ref())
                    .for_each(|(card_idx, card)| {
                        self.cards[card_idx] = card;
                        self.card_head += 1;
                    });
            } else {
                for card_idx in self.selection.iter() {
                    self.cards.remove(card_idx);
                }
            }
            // TODO: smarter cache clearing
            for cache in &self.caches {
                cache.clear();
            }

            while !check_if_has_set(&self.cards) && self.card_head < 81 {
                self.cards.extend(self.all_cards.by_ref().take(3));
                self.card_head += 3;
            }
            self.selection.size = self.cards.len() as u8;

            if !check_if_has_set(&self.cards) {
                self.finished = true;
                info!("You win!");
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

impl<Deck: Iterator<Item = ClassicCard> + Default> Default for ClassicSet<Deck> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn code_to_grid(code: Code) -> Option<(u8, u8)> {
    match code {
        Code::KeyQ => Some((0, 0)),
        Code::KeyW => Some((0, 1)),
        Code::KeyE => Some((0, 2)),
        Code::KeyR => Some((0, 3)),
        Code::KeyT => Some((0, 4)),
        Code::KeyY => Some((0, 5)),
        Code::KeyU => Some((0, 6)),

        Code::KeyA => Some((1, 0)),
        Code::KeyS => Some((1, 1)),
        Code::KeyD => Some((1, 2)),
        Code::KeyF => Some((1, 3)),
        Code::KeyG => Some((1, 4)),
        Code::KeyH => Some((1, 5)),
        Code::KeyJ => Some((1, 6)),

        Code::KeyZ => Some((2, 0)),
        Code::KeyX => Some((2, 1)),
        Code::KeyC => Some((2, 2)),
        Code::KeyV => Some((2, 3)),
        Code::KeyB => Some((2, 4)),
        Code::KeyN => Some((2, 5)),
        Code::KeyM => Some((2, 6)),

        _ => None,
    }
}
