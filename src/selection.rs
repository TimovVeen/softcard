use std::iter::Sum;

pub trait Sel {
    fn toggle(&mut self, index: u8);
    fn clear(&mut self);
    fn is_empty(&self) -> bool;
    fn is_selected(&self, index: u8) -> bool;
    fn len(&self) -> usize;
    fn empty() -> Self;
    fn invert(&mut self);

    fn iter(&self) -> impl Iterator<Item = usize>;

    fn check_set<Card: Copy + Default + Sum + Eq>(&self, cards: &[Card]) -> bool {
        self.iter().map(|i| cards[i]).sum::<Card>() == Card::default()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Selection {
    mask: u32,
    pub size: u8,
}

impl Sel for Selection {
    fn toggle(&mut self, index: u8) {
        self.mask ^= (1 << index) & ((1 << self.size) - 1);
    }

    fn clear(&mut self) {
        self.mask = 0
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
        self.map(|x| x as usize)
    }
}

impl Selection {
    pub fn new(size: u8) -> Self {
        Self { mask: 0, size }
    }

    pub fn check_set<Card: Copy + Default + Sum + Eq>(&self, cards: &[Card]) -> bool {
        self.into_iter().map(|i| cards[i as usize]).sum::<Card>() == Card::default()
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

impl DoubleEndedIterator for Selection {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.mask.lowest_one().map(|x| {
            self.mask ^= 1 << x;
            x as u8
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct OrderedSelection {
    pub selection: Vec<usize>,
}

impl Sel for OrderedSelection {
    fn toggle(&mut self, index: u8) {
        if let Some(i) = self.selection.iter().position(|x| *x == index as usize) {
            self.selection.remove(i);
        } else {
            self.selection.push(index as usize);
        }
    }

    fn clear(&mut self) {
        self.selection.clear();
    }

    fn is_empty(&self) -> bool {
        self.selection.is_empty()
    }

    fn is_selected(&self, index: u8) -> bool {
        self.selection.contains(&(index as usize))
    }
    fn len(&self) -> usize {
        self.selection.len()
    }

    fn iter(&self) -> impl Iterator<Item = usize> {
        self.selection.iter().copied()
    }

    fn empty() -> Self {
        Self::default()
    }

    // TODO: not very modular
    fn invert(&mut self) {}
}
