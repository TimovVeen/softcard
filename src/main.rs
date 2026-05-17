use iced::{
    Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, keyboard, mouse,
    widget::canvas::{self, Canvas, Path, Stroke, event::Status},
};
use log::info;

const RADIUS: f32 = 60.0;
const CARD_WIDTH: f32 = 400.0;
const CARD_HEIGHT: f32 = 600.0;

const WINDOW_MARGIN: f32 = 20.0;
const SPACING: f32 = 20.0;
const MARGIN: f32 = 50.0;
const PADDING: f32 = 60.0;
const OFFSET: f32 = 2.0 * RADIUS + PADDING;
const MARGIN_OFFSET: f32 = MARGIN + RADIUS;

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

    fn handle_key(&mut self, key: keyboard::Key) {
        match key.as_ref() {
            keyboard::Key::Character("q") | keyboard::Key::Character("Q") => {
                print_solution(&self.cards);
            }
            keyboard::Key::Character("c") | keyboard::Key::Character("C") => {
                self.selection = 0;
            }
            keyboard::Key::Character(ch) => {
                if let Ok(num) = ch.parse::<u8>()
                    && (1..=7).contains(&num)
                {
                    self.toggle_card((num - 1) as usize);
                }
            }
            _ => {}
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
        if self.selection == 0 {
            return;
        }

        if xor_selected(self.selection, &self.cards) != 0 {
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
}

impl Default for SetApp {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
enum Message {
    ToggleCard(usize),
    KeyPressed(keyboard::Key),
}

fn update(app: &mut SetApp, message: Message) {
    match message {
        Message::ToggleCard(card) => app.toggle_card(card),
        Message::KeyPressed(key) => app.handle_key(key),
    }
}

fn view(app: &SetApp) -> Element<'_, Message> {
    Canvas::new(Board {
        cards: app.cards,
        selection: app.selection,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[derive(Debug, Clone, Copy)]
struct Board {
    cards: [u8; 7],
    selection: u8,
}

impl canvas::Program<Message> for Board {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (Status, Option<Message>) {
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor_position(cursor)
                    && let Some(card) = card_at(position, bounds)
                {
                    return (Status::Captured, Some(Message::ToggleCard(card)));
                }
                (Status::Ignored, None)
            }
            canvas::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                (Status::Captured, Some(Message::KeyPressed(key)))
            }
            _ => (Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let background = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&background, Color::WHITE);

        let card_scale = card_scale(bounds.size());

        for (i, &card) in self.cards.iter().enumerate() {
            let selected = self.selection & (1 << i) != 0;
            draw_projcard(
                &mut frame,
                Point::new(
                    WINDOW_MARGIN + (CARD_WIDTH + SPACING) * (i % 4) as f32,
                    WINDOW_MARGIN + (CARD_HEIGHT + SPACING) * (i / 4) as f32,
                ),
                card_scale,
                card,
                selected,
            );
        }

        vec![frame.into_geometry()]
    }
}

fn xor_selected(selection: u8, cards: &[u8; 7]) -> u8 {
    let mut sels = selection;
    let mut res = 0;
    while sels != 0 {
        res ^= cards[sels.trailing_zeros() as usize];
        sels &= sels - 1;
    }
    res
}

fn card_scale(size: Size) -> f32 {
    let x_cardscale = size.width / ((CARD_WIDTH + SPACING) * 4.0 - SPACING + WINDOW_MARGIN * 2.0);
    let y_cardscale = size.height / ((CARD_HEIGHT + SPACING) * 2.0 - SPACING + WINDOW_MARGIN * 2.0);

    x_cardscale.min(y_cardscale)
}

fn cursor_position(cursor: mouse::Cursor) -> Option<Point> {
    match cursor {
        mouse::Cursor::Available(position) => Some(position),
        mouse::Cursor::Unavailable => None,
    }
}

fn card_at(position: Point, bounds: Rectangle) -> Option<usize> {
    let local_x = position.x - bounds.x;
    let local_y = position.y - bounds.y;
    if local_x < 0.0 || local_y < 0.0 {
        return None;
    }

    let cardscale = card_scale(bounds.size());
    if cardscale <= 0.0 {
        return None;
    }

    let horizontal =
        ((local_x / cardscale - WINDOW_MARGIN) / (CARD_WIDTH + SPACING)).floor() as i32;
    let vertical = ((local_y / cardscale - WINDOW_MARGIN) / (CARD_HEIGHT + SPACING)).floor() as i32;
    let index = vertical * 4 + horizontal;

    if !(0..4).contains(&horizontal) || !(0..7).contains(&index) {
        return None;
    }

    Some(index as usize)
}

fn draw_projcard(frame: &mut canvas::Frame, origin: Point, scale: f32, mask: u8, selected: bool) {
    let card_size = Size::new(CARD_WIDTH * scale, CARD_HEIGHT * scale);
    let card = Path::rectangle(origin, card_size);

    if selected {
        frame.fill(&card, Color::from_rgb8(0xE0, 0xE0, 0xE0));
    }

    frame.stroke(
        &card,
        Stroke {
            width: 3.0 * scale,
            style: Color::BLACK.into(),
            ..Stroke::default()
        },
    );

    for i in 0..3 {
        for j in 0..2 {
            let idx = i * 2 + j;
            if (1 << idx) & mask != 0 {
                let center = Point::new(
                    origin.x + (MARGIN_OFFSET + OFFSET * j as f32) * scale,
                    origin.y + (MARGIN_OFFSET + OFFSET * i as f32) * scale,
                );
                let dot = Path::circle(center, RADIUS * scale);
                frame.fill(&dot, color(idx));
            }
        }
    }
}

fn color(idx: usize) -> Color {
    match idx {
        0 => Color::from_rgb8(0xFF, 0x00, 0x00),
        1 => Color::from_rgb8(0xFF, 0xA5, 0x00),
        2 => Color::from_rgb8(0xFF, 0xD7, 0x00),
        3 => Color::from_rgb8(0x00, 0x80, 0x00),
        4 => Color::from_rgb8(0x00, 0x00, 0xFF),
        5 => Color::from_rgb8(0x80, 0x00, 0x80),
        _ => Color::BLACK,
    }
}

fn print_solution(cards: &[u8; 7]) {
    for i in 1..0b10000000_usize {
        if i.count_ones() < 3 {
            continue;
        }

        if xor_selected(i as u8, cards) == 0 {
            info!("{:b}", i);
            return;
        }
    }
}

fn main() -> iced::Result {
    #[cfg(not(target_family = "wasm"))]
    simple_logger::init_with_level(log::Level::Info).unwrap();
    #[cfg(target_family = "wasm")]
    console_log::init_with_level(log::Level::Info).unwrap();

    iced::application("Softcard", update, view)
        .window_size((800., 600.))
        .run()
}
