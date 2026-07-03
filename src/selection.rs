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
}

impl Iterator for Selection {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.mask == 0 {
            return None;
        }

        let pos = self.mask.trailing_zeros();
        self.mask &= self.mask - 1;
        Some(pos as u8)
    }
}

impl ExactSizeIterator for Selection {
    fn len(&self) -> usize {
        self.mask.count_ones() as usize
    }
}
