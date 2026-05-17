use iced::{
    Border, Color, Element, Length, Point, Rectangle, Renderer, Subscription, Theme, keyboard,
    mouse,
    widget::{
        self,
        canvas::{self, Path},
        container, grid, mouse_area,
    },
};
use log::info;

const BOARD_PADDING: f32 = 20.;
const GRID_SPACING: f32 = 20.;
const CARD_ASPECT: f32 = 2. / 3.;
const CARD_INNER_PADDING: f32 = 0.;
const DOT_RADIUS_RATIO: f32 = 0.15;

const CARD_COLORS: [Color; 6] = [
    Color::from_rgb8(0xFF, 0x00, 0x00),
    Color::from_rgb8(0xFF, 0xA5, 0x00),
    Color::from_rgb8(0xFF, 0xD7, 0x00),
    Color::from_rgb8(0x00, 0x80, 0x00),
    Color::from_rgb8(0x00, 0x00, 0xFF),
    Color::from_rgb8(0x80, 0x00, 0x80),
];

const CARDS: [u8; 63] = {
    let mut res = [0_u8; _];
    let mut i = 0;
    while i < res.len() {
        res[i] = i as u8 + 1;
        i += 1;
    }
    res
};

#[derive(Debug)]
struct SetApp {
    cards: [u8; 7],
    all_cards: [u8; 63],
    selection: u8,
    card_head: usize,
    finished: bool,
}

impl SetApp {
    fn new() -> Self {
        let mut all_cards = CARDS;
        fastrand::shuffle(&mut all_cards);

        Self {
            cards: all_cards[..7].try_into().unwrap(),
            all_cards,
            selection: 0,
            card_head: 7,
            finished: false,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleCard(card) => self.toggle_card(card),
            Message::KeyboardEvent(event) => self.handle_keyboard_event(event),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let cards = widget::grid![
            self.card_widget(0),
            self.card_widget(1),
            self.card_widget(2),
            self.card_widget(3),
            self.card_widget(4),
            self.card_widget(5),
            self.card_widget(6),
        ]
        .columns(4)
        .spacing(GRID_SPACING)
        .height(grid::Sizing::AspectRatio(CARD_ASPECT));

        container(cards).padding(BOARD_PADDING).into()
    }

    fn card_widget(&self, index: usize) -> Element<'_, Message> {
        let selected = self.is_selected(index);
        let card = container(
            canvas::Canvas::new(CardCanvas {
                mask: self.cards[index],
            })
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .padding(CARD_INNER_PADDING)
        .style(move |_theme| container::Style {
            background: Some(
                if selected {
                    Color::from_rgb8(0x71, 0x77, 0x7F)
                } else {
                    Color::WHITE
                }
                .into(),
            ),
            border: Border {
                color: Color::BLACK,
                width: 1.5,
                radius: 10.0.into(),
            },
            ..Default::default()
        });

        if self.finished {
            card.into()
        } else {
            mouse_area(card).on_press(Message::ToggleCard(index)).into()
        }
    }

    fn handle_keyboard_event(&mut self, event: keyboard::Event) {
        if let keyboard::Event::KeyPressed { key, repeat, .. } = event
            && !repeat
        {
            match key.as_ref() {
                keyboard::Key::Character("q") | keyboard::Key::Character("Q") => {
                    self.print_solution();
                }
                keyboard::Key::Character("c") | keyboard::Key::Character("C") => {
                    self.selection = 0;
                }
                keyboard::Key::Character(ch)
                    if let Ok(num) = ch.parse::<u8>()
                        && (1..=7).contains(&num) =>
                {
                    self.toggle_card((num - 1) as usize);
                }

                _ => {}
            }
        }
    }

    fn toggle_card(&mut self, card: usize) {
        if self.finished || card >= self.cards.len() {
            return;
        }

        self.selection ^= 1 << card;
        self.resolve_selection();
    }

    fn resolve_selection(&mut self) {
        if self.selection == 0 || self.xor_selected(self.selection) != 0 {
            return;
        }

        info!("You got a set!");
        let mut sels = self.selection;
        while sels != 0 {
            if self.card_head >= self.all_cards.len() {
                self.finished = true;
                self.selection = 0;
                info!("You win!");
                return;
            }

            let index = sels.trailing_zeros() as usize;
            self.cards[index] = self.all_cards[self.card_head];
            self.card_head += 1;
            sels &= sels - 1;
        }

        self.selection = 0;
    }

    fn xor_selected(&self, selection: u8) -> u8 {
        let mut sels = selection;
        let mut res = 0;
        while sels != 0 {
            res ^= self.cards[sels.trailing_zeros() as usize];
            sels &= sels - 1;
        }
        res
    }

    fn print_solution(&self) {
        for selection in 1..0b10000000_u8 {
            if selection.count_ones() < 3 {
                continue;
            }

            if self.xor_selected(selection) == 0 {
                info!("{selection:b}");
                return;
            }
        }
    }

    fn is_selected(&self, index: usize) -> bool {
        self.selection & (1 << index) != 0
    }
}

impl Default for SetApp {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
enum Message {
    ToggleCard(usize),
    KeyboardEvent(keyboard::Event),
}

#[derive(Debug, Clone, Copy)]
struct CardCanvas {
    mask: u8,
}

impl<Message> canvas::Program<Message> for CardCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let radius = bounds.width * DOT_RADIUS_RATIO;
        for (row, y) in [0.18333334, 0.48333335, 0.78333336].iter().enumerate() {
            for (col, x) in [0.275, 0.725].iter().enumerate() {
                let idx = row * 2 + col;
                if self.mask & (1 << idx) == 0 {
                    continue;
                }

                let center = Point::new(bounds.width * x, bounds.height * y);
                let dot = Path::circle(center, radius);
                frame.fill(&dot, CARD_COLORS[idx]);
            }
        }

        vec![frame.into_geometry()]
    }
}

fn subscription(_app: &SetApp) -> Subscription<Message> {
    keyboard::listen().map(Message::KeyboardEvent)
}

fn main() -> iced::Result {
    #[cfg(not(target_family = "wasm"))]
    simple_logger::init_with_level(log::Level::Info).unwrap();
    #[cfg(target_family = "wasm")]
    console_log::init_with_level(log::Level::Info).unwrap();

    iced::application(SetApp::default, SetApp::update, SetApp::view)
        .title("Softcard")
        .subscription(subscription)
        .window_size((800., 600.))
        .run()
}
