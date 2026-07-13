use crate::cards::card::CardGen;

#[derive(Clone)]
pub struct ShuffleDeck<Card: CardGen> {
    deck: std::vec::IntoIter<Card>,
}

impl<Card: CardGen> Default for ShuffleDeck<Card> {
    fn default() -> Self {
        let mut all_cards = Card::all();
        fastrand::shuffle(&mut all_cards);
        Self {
            deck: all_cards.into_iter(),
        }
    }
}

impl<Card: CardGen> Iterator for ShuffleDeck<Card> {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        self.deck.next()
    }
}
