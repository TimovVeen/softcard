use iced::{
    Border, Color, Element, Function, Length, Subscription, keyboard,
    time::{self, Instant, milliseconds},
    widget::{self, container, grid, responsive},
};
use log::info;

use crate::{
    BOARD_PADDING, CARD_ASPECT, GRID_SPACING,
    card::{self, CardCanvas, ProjCard},
    selection::Selection,
};

#[derive(Debug, Clone)]
pub enum Message {
    Card(u8, card::Message),
    KeyboardEvent(keyboard::Event),
    Restart,
    Exit,
    Tick(Instant),
}

#[derive(Debug, Clone)]
pub struct ProjSet {
    cards: [ProjCard; 7],
    all_cards: [ProjCard; 63],
    selection: Selection,
    card_head: usize,
    finished: bool,
    start_time: Instant,
    current_time: Instant,
}

impl ProjSet {
    pub fn new() -> Self {
        let mut all_cards = {
            let mut res = [ProjCard::default(); 63];
            let mut i = 0;
            while i < res.len() {
                res[i] = ProjCard::new(i as u8 + 1);
                i += 1;
            }
            res
        };
        fastrand::shuffle(&mut all_cards);

        Self {
            cards: all_cards[..7].try_into().unwrap(),
            all_cards,
            selection: Selection::new(7),
            card_head: 7,
            finished: false,
            start_time: Instant::now(),
            current_time: Instant::now(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Card(card, card::Message::Toggle) => self.toggle_card(card),
            Message::KeyboardEvent(event) => self.handle_keyboard_event(event),
            Message::Restart => *self = Self::new(),
            Message::Exit => (),
            Message::Tick(now) => self.current_time = now,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(responsive(|size| {
            let expected_width =
                (size.height - GRID_SPACING) * CARD_ASPECT * 2. + 3. * GRID_SPACING;

            let buttons = widget::row![
                widget::button("Restart")
                    .on_press(Message::Restart)
                    .width(Length::Fill),
                widget::button("Menu")
                    .on_press(Message::Exit)
                    .width(Length::Fill),
            ]
            .spacing(5.);

            let elapsed_time = (self.current_time - self.start_time).as_millis();
            let millis = elapsed_time % 1000;
            let seconds = (elapsed_time / 1000) % 60;
            let minutes = elapsed_time / 60000;
            let stats = container(
                if !self.finished {
                    widget::column![
                        widget::text!("Remaining cards: {}", 63 - self.card_head),
                        widget::text!("Time: {:02}:{:02}", minutes, seconds),
                    ]
                } else {
                    widget::column![
                        widget::text!("Finished!"),
                        widget::text!("Time: {:02}:{:02}:{:03}", minutes, seconds, millis),
                    ]
                }
                .push(buttons)
                .spacing(5.),
            )
            .padding(10.)
            .style(move |_theme| container::Style {
                background: Some(Color::WHITE.into()),
                border: Border {
                    color: Color::BLACK,
                    width: 1.5,
                    radius: 10.0.into(),
                },
                ..Default::default()
            });

            grid(self.cards.iter().enumerate().map(|(i, card)| {
                card.view(self.selection.is_selected(i as u8))
                    .map(Message::Card.with(i as u8))
            }))
            .push(stats)
            .columns(4)
            .spacing(GRID_SPACING)
            .width(size.width.min(expected_width))
            .height(grid::Sizing::AspectRatio(CARD_ASPECT))
        }))
        .padding(BOARD_PADDING)
        .into()
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
        if self.selection.is_empty() || !self.is_selected_set() {
            return;
        }

        info!("You got a set!");
        for card in self.selection {
            if self.card_head >= self.all_cards.len() - 1 {
                self.finished = true;
                self.selection.clear();
                info!("You win!");
                return;
            }

            self.cards[card as usize] = self.all_cards[self.card_head];
            self.card_head += 1;
        }
        self.selection.clear();
    }

    fn is_selected_set(&self) -> bool {
        self.selection
            .into_iter()
            .map(|i| self.cards[i as usize])
            .sum::<ProjCard>()
            == ProjCard::default()
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

impl Default for ProjSet {
    fn default() -> Self {
        Self::new()
    }
}
