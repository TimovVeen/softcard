use std::{array::from_fn, iter::Sum, ops::Add};

use iced::{
    Color, Point, Renderer,
    widget::canvas::{self, Path, Stroke, path::Builder},
};
use itertools::Itertools;

use crate::cards::card::{CardDraw, CardGen};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymCard<const N: usize>(pub [u8; N]);

impl<const N: usize> Default for SymCard<N> {
    fn default() -> Self {
        Self(from_fn(|i| i as u8))
    }
}

impl<const N: usize> Add for SymCard<N> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0.map(|x| rhs.0[x as usize]))
    }
}

impl<const N: usize> Sum for SymCard<N> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Self::add)
    }
}

impl<const N: usize> SymCard<N> {
    pub const fn new(mask: [u8; N]) -> Self {
        Self(mask)
    }

    pub fn odd_parity(&self) -> bool {
        let mut inversions = 0;
        for i in 0..N {
            for j in (i + 1)..N {
                if self.0[i] > self.0[j] {
                    inversions += 1;
                }
            }
        }
        inversions % 2 != 0
    }
}

impl<const N: usize> CardDraw for SymCard<N> {
    fn draw(&self, frame: &mut canvas::Frame<Renderer>) {
        let step = 0.7 / (N as f32 - 1.);
        let mut lines = Builder::new();
        for (start, &end) in self.0.iter().enumerate() {
            let start_y = (0.1 + step * start as f32) * frame.height();
            let end_y = (0.1 + step * end as f32) * frame.height();
            lines.move_to(Point::new(0., start_y));
            lines.line_to(Point::new(0.1 * frame.width(), start_y));
            lines.bezier_curve_to(
                Point::new(0.5 * frame.width(), start_y),
                Point::new(0.5 * frame.width(), end_y),
                Point::new(0.9 * frame.width(), end_y),
            );
            lines.line_to(Point::new(frame.width(), end_y));
        }

        frame.stroke(&lines.build(), Stroke::default());

        if self.odd_parity() {
            frame.fill(
                &Path::circle(
                    Point::new(frame.width() * 0.5, frame.height()),
                    frame.width() * 0.1,
                ),
                Color::BLACK,
            );
        }
    }
}

impl<const N: usize> CardGen for SymCard<N> {
    fn all() -> impl Iterator<Item = Self> {
        (0..N as u8)
            .permutations(N)
            .map(|x| Self::new(x.iter().copied().collect_array().unwrap()))
            .skip(1)
    }

    fn random() -> Self {
        Self::new(from_fn(|_| fastrand::u8(0..N as u8)))
    }
}
