use std::iter::Sum;

use iced::{
    Border, Color, Element, Fill, Rectangle, Renderer, Theme, color, mouse,
    widget::{
        canvas::{self, Cache},
        container, mouse_area,
    },
};
use itertools::Itertools;

#[derive(Debug, Clone)]
pub enum Message {
    Toggle,
}

pub struct CardCanvas<Card: CardDraw> {
    card: Card,
    cache: Cache,
}

impl<Card: CardDraw> CardCanvas<Card> {
    pub fn new(card: Card) -> Self {
        Self {
            card,
            cache: Cache::new(),
        }
    }

    pub fn set_card(&mut self, card: Card) {
        self.card = card;
        self.cache.clear();
    }

    pub fn get_card(&self) -> &Card {
        &self.card
    }

    pub fn view(&self, selected: bool) -> Element<'_, Message> {
        let card =
            container(canvas::Canvas::new(self).width(Fill).height(Fill)).style(move |_theme| {
                container::Style {
                    background: Some(
                        if selected {
                            color!(0x71777F)
                        } else {
                            Color::WHITE
                        }
                        .into(),
                    ),
                    border: Border {
                        color: Color::BLACK,
                        width: 1.5,
                        radius: 10.0.into(),
                    },
                    ..Default::default()
                }
            });

        mouse_area(card).on_press(Message::Toggle).into()
    }
}

impl<Message, Card: CardDraw> canvas::Program<Message> for CardCanvas<Card> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let card = self
            .cache
            .draw(renderer, bounds.size(), |frame| self.card.draw(frame));

        vec![card]
    }
}

pub trait CardDraw {
    fn draw(&self, frame: &mut canvas::Frame<Renderer>);
}

pub fn check_if_has_set<Card: CardDraw + Copy + Sum + Default + Eq>(
    cards: &[CardCanvas<Card>],
) -> bool {
    (0..cards.len())
        .array_combinations::<3>()
        .any(|idxs| Card::default() == idxs.map(|i| *cards[i].get_card()).into_iter().sum())
}

pub trait CardGen: Sized {
    fn all() -> Vec<Self>;
}
