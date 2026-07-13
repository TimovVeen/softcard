use std::{array::from_fn, iter::Sum, ops::Add};

use iced::{
    Color, Point, Renderer, color,
    widget::canvas::{self, Path},
};

use crate::cards::card::{CardDraw, CardGen};

const DOT_RADIUS_RATIO: f32 = 0.15;

const CARD_COLORS: [Color; 6] = [
    color!(0xFF0000),
    color!(0xFFA500),
    color!(0xFFD700),
    color!(0x008000),
    color!(0x0000FF),
    color!(0x800080),
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProjCard {
    pub mask: u8,
}

impl Add for ProjCard {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
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

impl CardGen for ProjCard {
    fn all() -> Vec<Self> {
        from_fn::<_, 63, _>(|i| ProjCard::new(i as u8 + 1)).to_vec()
    }
}
