use crate::selection::Sel;

#[derive(Clone, Debug, Default)]
pub struct Ordered {
    pub selection: Vec<usize>,
}

impl Sel for Ordered {
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
