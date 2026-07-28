pub mod ordered;
pub mod unordered;

pub use ordered::Ordered;
use std::iter::Sum;
pub use unordered::Unordered;

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
