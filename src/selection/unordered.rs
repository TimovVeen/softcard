use crate::selection::Sel;

#[derive(Clone, Debug)]
pub struct Unordered {
    mask: u32,
    pub size: u8,
}

impl Sel for Unordered {
    fn toggle(&mut self, index: u8) {
        self.mask ^= (1 << index) & ((1 << self.size) - 1);
    }

    fn clear(&mut self) {
        self.mask = 0;
    }

    fn is_empty(&self) -> bool {
        self.mask == 0
    }

    fn is_selected(&self, index: u8) -> bool {
        self.mask & (1 << index) != 0
    }

    fn invert(&mut self) {
        self.mask = !self.mask & ((1 << self.size) - 1);
    }

    fn empty() -> Self {
        // TODO: will regret this later, fix
        Self::new(7)
    }

    fn len(&self) -> usize {
        self.mask.count_ones() as usize
    }

    // TODO: change later
    fn iter(&self) -> impl Iterator<Item = usize> {
        self.clone().map(|x| x as usize)
    }
}

impl Unordered {
    pub fn new(size: u8) -> Self {
        Self { mask: 0, size }
    }
}

impl Iterator for Unordered {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.mask.highest_one().map(|x| {
            self.mask ^= 1 << x;
            x as u8
        })
    }
}

impl DoubleEndedIterator for Unordered {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.mask.lowest_one().map(|x| {
            self.mask ^= 1 << x;
            x as u8
        })
    }
}
