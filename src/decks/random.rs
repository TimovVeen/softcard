use crate::cards::card::CardGen;

#[derive(Default)]
pub struct RandomDeck<Card: CardGen> {
    _phantom: std::marker::PhantomData<Card>,
}

impl<Card: CardGen> Iterator for RandomDeck<Card> {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Card::random())
    }
}
