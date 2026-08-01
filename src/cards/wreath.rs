use std::{array::from_fn, iter::Sum, ops::Add};

use iced::{
    Color, Point, Renderer,
    widget::canvas::{self, Path},
};
use itertools::{Itertools, iproduct, repeat_n};

use crate::cards::{
    card::{CardDraw, CardGen},
    symmetric::SymCard,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WreathCard<const N: usize> {
    perm: SymCard<N>,
    dots: [bool; N],
}

impl<const N: usize> Default for WreathCard<N> {
    fn default() -> Self {
        Self {
            perm: Default::default(),
            dots: [false; _],
        }
    }
}

impl<const N: usize> Add for WreathCard<N> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        let mut res = [false; _];
        for i in 0..N {
            res[rhs.perm.0[i] as usize] = rhs.dots[i] != self.dots[self.perm.0[i] as usize];
        }
        Self::Output {
            perm: self.perm + rhs.perm,
            dots: res,
        }
    }
}

impl<const N: usize> Sum for WreathCard<N> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Self::add)
    }
}

impl<const N: usize> WreathCard<N> {
    pub const fn new(perm: [u8; N], dots: [bool; N]) -> Self {
        Self {
            perm: SymCard::new(perm),
            dots,
        }
    }
}

impl<const N: usize> CardDraw for WreathCard<N> {
    fn draw(&self, frame: &mut canvas::Frame<Renderer>) {
        self.perm.draw(frame);

        let step = 0.7 / (N as f32 - 1.);
        for start in 0..N {
            if self.dots[start] {
                frame.fill(
                    &Path::circle(
                        Point::new(
                            0.075 * frame.width(),
                            (0.1 + step * start as f32) * frame.height(),
                        ),
                        0.05 * frame.width(),
                    ),
                    Color::BLACK,
                );
            }
        }
    }
}

impl<const N: usize> CardGen for WreathCard<N> {
    // TODO: find a better way to do this
    fn all() -> impl Iterator<Item = Self> {
        iproduct!(
            (0..N as u8)
                .permutations(N)
                .map(|x| x.iter().copied().collect_array().unwrap()),
            repeat_n([false, true], N)
                .multi_cartesian_product()
                .map(|x| x.iter().copied().collect_array().unwrap())
        )
        .map(|(perm, mask)| Self::new(perm, mask))
        .skip(1)
    }

    fn random() -> Self {
        Self::new(
            from_fn(|_| fastrand::u8(0..N as u8)),
            from_fn(|_| fastrand::bool()),
        )
    }
}
