use std::{iter::Sum, ops::Add};

use iced::{
    Color, Point, Renderer, color,
    widget::canvas::{
        self, Stroke,
        path::lyon_path::geom::euclid::{Transform2D, Vector2D},
    },
};
use itertools::iproduct;

use crate::{
    cards::card::{CardDraw, CardGen},
    gui::shape::standard_shape,
};

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

            let shape = standard_shape(self.mask[2]);

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

impl CardGen for ClassicCard {
    fn all() -> Vec<Self> {
        iproduct!(0..3, 0..3, 0..3, 0..3)
            .map(|idxs| ClassicCard::new(idxs.into()))
            .collect()
    }
}
