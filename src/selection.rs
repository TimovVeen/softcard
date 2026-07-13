use std::iter::Sum;

use crate::cards::card::{CardCanvas, CardDraw};

#[derive(Clone, Copy, Debug)]
pub struct Selection {
    mask: u32,
    pub size: u8,
}

impl Selection {
    pub fn new(size: u8) -> Self {
        Self { mask: 0, size }
    }

    pub fn toggle(&mut self, index: u8) {
        self.mask ^= (1 << index) & ((1 << self.size) - 1);
    }

    pub fn invert(&mut self) {
        self.mask = !self.mask & ((1 << self.size) - 1);
    }

    pub fn clear(&mut self) {
        self.mask = 0
    }

    pub fn is_empty(&self) -> bool {
        self.mask == 0
    }

    pub fn is_selected(&self, index: u8) -> bool {
        self.mask & (1 << index) != 0
    }

    pub fn check_set<Card: CardDraw + Copy + Default + Sum + Eq>(
        &self,
        cards: &[CardCanvas<Card>],
    ) -> bool {
        self.into_iter()
            .map(|i| *cards[i as usize].get_card())
            .sum::<Card>()
            == Card::default()
    }
}

impl Iterator for Selection {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.mask.highest_one().map(|x| {
            self.mask ^= 1 << x;
            x as u8
        })
    }
}

impl ExactSizeIterator for Selection {
    fn len(&self) -> usize {
        self.mask.count_ones() as usize
    }
}

impl DoubleEndedIterator for Selection {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.mask.lowest_one().map(|x| {
            self.mask ^= 1 << x;
            x as u8
        })
    }
}
