use std::{iter::Sum, ops::Add};

use iced::{
    Border, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme,
    border::Radius,
    color, mouse,
    widget::{
        canvas::{
            self, Cache, Path, Stroke,
            path::{
                Builder,
                lyon_path::geom::euclid::{Transform2D, Vector2D},
            },
        },
        container, mouse_area,
    },
};

const DOT_RADIUS_RATIO: f32 = 0.15;

const CARD_COLORS: [Color; 6] = [
    color!(0xFF0000),
    color!(0xFFA500),
    color!(0xFFD700),
    color!(0x008000),
    color!(0x0000FF),
    color!(0x800080),
];

#[derive(Debug, Clone)]
pub enum Message {
    Toggle,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProjCard {
    pub mask: u8,
}

impl Add for ProjCard {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.mask ^ rhs.mask)
    }
}

impl Sum for ProjCard {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Self::add)
    }
}

impl ProjCard {
    pub const fn new(mask: u8) -> Self {
        Self { mask }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClassicCard {
    pub mask: [u8; 4],
}

impl ClassicCard {
    pub fn new(mask: [u8; 4]) -> Self {
        Self { mask }
    }
}

impl Add for ClassicCard {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut res = [0; 4];
        res.iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = (self.mask[i] + rhs.mask[i]) % 3);
        Self::Output::new(res)
    }
}

impl Sum for ClassicCard {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Self::add)
    }
}

pub struct CardCanvas<Card: CardDraw + Clone + Copy> {
    card: Card,
    cache: Cache,
}

impl<Card: CardDraw + Clone + Copy> CardCanvas<Card> {
    pub fn new(card: &Card) -> Self {
        Self {
            card: *card,
            cache: Cache::new(),
        }
    }

    pub fn set_card(&mut self, card: Card) {
        self.card = card;
        self.cache.clear();
    }

    pub fn get_card(&self) -> Card {
        self.card
    }

    pub fn view(&self, selected: bool) -> Element<'_, Message> {
        let card = container(
            canvas::Canvas::new(self)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .style(move |_theme| container::Style {
            background: Some(
                if selected {
                    color!(0x71777F)
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

        mouse_area(card).on_press(Message::Toggle).into()
    }
}

impl<Message, Card: CardDraw + Clone + Copy> canvas::Program<Message> for CardCanvas<Card> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let card = self
            .cache
            .draw(renderer, bounds.size(), |frame| self.card.draw(frame));

        vec![card]
    }
}

pub trait CardDraw {
    fn draw(&self, frame: &mut canvas::Frame<Renderer>);
}

impl CardDraw for ProjCard {
    fn draw(&self, frame: &mut canvas::Frame<Renderer>) {
        let radius = frame.width() * DOT_RADIUS_RATIO;
        for (row, y) in [0.18333334, 0.48333335, 0.78333336].iter().enumerate() {
            for (col, x) in [0.275, 0.725].iter().enumerate() {
                let idx = row * 2 + col;
                if self.mask & (1 << idx) == 0 {
                    continue;
                }

                let center = Point::new(frame.width() * x, frame.height() * y);
                let dot = Path::circle(center, radius);
                frame.fill(&dot, CARD_COLORS[idx]);
            }
        }
    }
}

impl CardDraw for ClassicCard {
    fn draw(&self, frame: &mut canvas::Frame<Renderer>) {
        {
            let color = match self.mask[0] {
                0 => color!(0xFF0000),
                1 => color!(0x00B500),
                2 => color!(0x592693),
                _ => panic!(),
            };

            let ypositions: &[f32] = match self.mask[1] {
                0 => &[0.5],
                1 => &[0.35, 0.65],
                2 => &[0.2, 0.5, 0.8],
                _ => panic!(),
            };

            let shape = match self.mask[2] {
                0 => {
                    Path::rounded_rectangle(Point::ORIGIN, Size::new(200., 100.), Radius::new(50.))
                }
                1 => {
                    let mut diamond = Builder::new();
                    diamond.move_to(Point::new(0.0, 50.0));
                    diamond.line_to(Point::new(100.0, 0.0));
                    diamond.line_to(Point::new(200.0, 50.0));
                    diamond.line_to(Point::new(100.0, 100.0));
                    diamond.close();
                    diamond.build()
                }
                2 => {
                    let mut squiggle = Builder::new();

                    squiggle.move_to(Point::new(198.0, 11.0));
                    squiggle.bezier_curve_to(
                        Point::new(214.8, 54.8),
                        Point::new(169.4, 102.6),
                        Point::new(116.0, 89.0),
                    );
                    squiggle.bezier_curve_to(
                        Point::new(94.6, 83.6),
                        Point::new(74.4, 65.0),
                        Point::new(44.0, 87.0),
                    );
                    squiggle.bezier_curve_to(
                        Point::new(9.2, 112.2),
                        Point::new(0.8, 97.6),
                        Point::new(0.0, 61.0),
                    );
                    squiggle.bezier_curve_to(
                        Point::new(-0.8, 25.0),
                        Point::new(28.2, 0.4),
                        Point::new(62.0, 5.0),
                    );
                    squiggle.bezier_curve_to(
                        Point::new(108.4, 11.4),
                        Point::new(113.8, 44.0),
                        Point::new(168.0, 9.0),
                    );
                    squiggle.bezier_curve_to(
                        Point::new(180.6, 1.0),
                        Point::new(191.8, -5.2),
                        Point::new(198.0, 11.0),
                    );
                    squiggle.build()
                }
                _ => panic!(),
            };

            let basetransform = Transform2D::translation(-100., -50.)
                .then_scale(frame.width() / 300., frame.width() / 300.);
            for ypos in ypositions {
                let transformation = basetransform
                    .then_translate(Vector2D::new(frame.width() / 2., frame.height() * ypos));
                let transformed = shape.transform(&transformation);
                if self.mask[3] == 2 {
                    frame.fill(&transformed, color);
                } else if self.mask[3] == 1 {
                    frame.fill(
                        &transformed,
                        canvas::Fill {
                            style: canvas::Style::Gradient(canvas::Gradient::Linear(
                                canvas::gradient::Linear::new(
                                    Point::new(0., 0.),
                                    Point::new(frame.width(), 0.),
                                )
                                .add_stop(0.4, Color::WHITE)
                                .add_stop(1., color),
                            )),
                            ..Default::default()
                        },
                    );
                }
                frame.stroke(
                    &transformed,
                    Stroke {
                        style: canvas::Style::Solid(color),
                        width: frame.width() / 60.,
                        ..Stroke::default()
                    },
                );
            }
        }
    }
}
