use std::{array::from_fn, iter::Sum, ops::Add};

use iced::{
    Color, Point, Renderer,
    widget::canvas::{self, Path, Stroke},
};
use itertools::{Itertools, iproduct, repeat_n};

use crate::cards::card::{CardDraw, CardGen};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WreathCard<const N: usize> {
    perm: [u8; N],
    mask: [bool; N],
}

impl<const N: usize> Default for WreathCard<N> {
    fn default() -> Self {
        Self {
            perm: from_fn(|i| i as u8),
            mask: [false; _],
        }
    }
}

impl<const N: usize> Add for WreathCard<N> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        let mut res = [false; _];
        for i in 0..N {
            res[rhs.perm[i] as usize] = rhs.mask[i] != self.mask[self.perm[i] as usize];
        }
        Self::Output::new(self.perm.map(|x| rhs.perm[x as usize]), res)
    }
}

impl<const N: usize> Sum for WreathCard<N> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Self::add)
    }
}

impl<const N: usize> WreathCard<N> {
    pub const fn new(perm: [u8; N], mask: [bool; N]) -> Self {
        Self { perm, mask }
    }
}

impl<const N: usize> CardDraw for WreathCard<N> {
    fn draw(&self, frame: &mut canvas::Frame<Renderer>) {
        let step = 0.7 / (N as f32 - 1.);
        for (start, &end) in self.perm.iter().enumerate() {
            frame.stroke(
                &Path::line(
                    Point::new(0., (0.1 + step * start as f32) * frame.height()),
                    Point::new(frame.width(), (0.1 + step * end as f32) * frame.height()),
                ),
                Stroke::default(),
            );

            if self.mask[start] {
                frame.fill(
                    &Path::circle(
                        Point::new(
                            0.1 * frame.width(),
                            (0.1 + step * start as f32) * frame.height(),
                        ),
                        0.07 * frame.width(),
                    ),
                    Color::BLACK,
                );
            }
        }

        let mut inversions = 0;
        for i in 0..N {
            for j in (i + 1)..N {
                if self.perm[i] > self.perm[j] {
                    inversions += 1;
                }
            }
        }
        if inversions % 2 != 0 {
            let dot = Path::circle(
                Point::new(frame.width() * 0.5, frame.height()),
                frame.width() * 0.1,
            );
            frame.fill(&dot, Color::BLACK);
        }
    }
}

impl<const N: usize> CardGen for WreathCard<N> {
    // TODO: find a better way to do this
    fn all() -> Vec<Self> {
        iproduct!(
            (0..N as u8)
                .permutations(N)
                .map(|x| x.iter().copied().collect_array::<N>().unwrap()),
            repeat_n([false, true], N)
                .multi_cartesian_product()
                .map(|x| x.iter().copied().collect_array::<N>().unwrap())
        )
        .map(|(perm, mask)| Self::new(perm, mask))
        .skip(1)
        .collect()
    }
}
