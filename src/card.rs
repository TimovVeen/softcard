use std::{iter::Sum, ops::Add};

use iced::{
    Border, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme,
    border::Radius,
    mouse,
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
    Color::from_rgb8(0xFF, 0x00, 0x00),
    Color::from_rgb8(0xFF, 0xA5, 0x00),
    Color::from_rgb8(0xFF, 0xD7, 0x00),
    Color::from_rgb8(0x00, 0x80, 0x00),
    Color::from_rgb8(0x00, 0x00, 0xFF),
    Color::from_rgb8(0x80, 0x00, 0x80),
];

#[derive(Debug, Clone)]
pub enum Message {
    Toggle,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ProjCard {
    pub mask: u8,
}

impl Add for ProjCard {
    type Output = ProjCard;

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

#[derive(Default)]
pub struct ProjCanvas {
    card: ProjCard,
    cache: Cache,
}

impl ProjCanvas {
    pub fn new(card: ProjCard) -> Self {
        Self {
            card,
            cache: Cache::new(),
        }
    }

    pub fn set_card(&mut self, card: ProjCard) {
        self.card = card;
        self.cache.clear();
    }

    pub fn get_card(&self) -> ProjCard {
        self.card
    }
}

impl<Message> canvas::Program<Message> for ProjCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let card = self.cache.draw(renderer, bounds.size(), |frame| {
            let radius = bounds.width * DOT_RADIUS_RATIO;
            for (row, y) in [0.18333334, 0.48333335, 0.78333336].iter().enumerate() {
                for (col, x) in [0.275, 0.725].iter().enumerate() {
                    let idx = row * 2 + col;
                    if self.card.mask & (1 << idx) == 0 {
                        continue;
                    }

                    let center = Point::new(bounds.width * x, bounds.height * y);
                    let dot = Path::circle(center, radius);
                    frame.fill(&dot, CARD_COLORS[idx]);
                }
            }
        });

        vec![card]
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ClassicCard {
    pub mask: [u8; 4],
}

impl ClassicCard {
    pub fn new(mask: [u8; 4]) -> Self {
        Self { mask }
    }
}

#[derive(Default)]
pub struct ClassicCanvas {
    card: ClassicCard,
    cache: Cache,
}

impl ClassicCanvas {
    pub fn new(card: ClassicCard) -> Self {
        Self {
            card,
            cache: Cache::new(),
        }
    }

    pub fn set_card(&mut self, card: ClassicCard) {
        self.card = card;
        self.cache.clear();
    }

    pub fn get_card(&self) -> ClassicCard {
        self.card
    }
}

impl Add for ClassicCard {
    type Output = ClassicCard;

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

impl<Message> canvas::Program<Message> for ClassicCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let card = self.cache.draw(renderer, bounds.size(), |frame| {
            let color = match self.card.mask[0] {
                0 => Color::from_rgb8(0xFF, 0x00, 0x00),
                1 => Color::from_rgb8(0x00, 0xB5, 0x00),
                2 => Color::from_rgb8(0xEF, 0x00, 0xDF),
                _ => panic!(),
            };

            let ypositions: &[f32] = match self.card.mask[1] {
                0 => &[0.5],
                1 => &[0.35, 0.65],
                2 => &[0.2, 0.5, 0.8],
                _ => panic!(),
            };

            let shape = match self.card.mask[2] {
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
                .then_scale(bounds.width / 300., bounds.width / 300.);
            for ypos in ypositions {
                let transformation = basetransform
                    .then_translate(Vector2D::new(bounds.width / 2., bounds.height * ypos));
                let transformed = shape.transform(&transformation);
                if self.card.mask[3] == 2 {
                    frame.fill(&transformed, color);
                } else if self.card.mask[3] == 1 {
                    frame.fill(
                        &transformed,
                        canvas::Fill {
                            style: canvas::Style::Gradient(canvas::Gradient::Linear(
                                canvas::gradient::Linear::new(
                                    Point::new(0., 0.),
                                    Point::new(bounds.width, 0.),
                                )
                                .add_stop(0.2, Color::from_rgb8(0xFF, 0xFF, 0xFF))
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
                        width: 5.,
                        ..Stroke::default()
                    },
                );
            }
        });

        vec![card]
    }
}

pub trait CardCanvas {
    fn view(&self, selected: bool) -> Element<'_, Message>;
}

impl<P: canvas::Program<Message>> CardCanvas for P {
    fn view(&self, selected: bool) -> Element<'_, Message> {
        let card = container(
            canvas::Canvas::new(self)
                .width(Length::Fill)
                .height(Length::Fill),
        )
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

        mouse_area(card).on_press(Message::Toggle).into()
    }
}
