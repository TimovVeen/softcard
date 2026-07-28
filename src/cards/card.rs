use std::iter::Sum;

use iced::{
    Border, Color, Fill, Rectangle, Renderer, Theme, color, mouse,
    widget::{
        canvas::{self, Cache},
        container, mouse_area,
    },
};
use itertools::Itertools;

use crate::gui::Element;

#[derive(Debug, Clone)]
pub enum Message {
    Toggle,
}

struct CardCanvas<'a, Card: CardDraw> {
    card: Card,
    cache: &'a Cache,
}

impl<'a, Message, Card: CardDraw> canvas::Program<Message> for CardCanvas<'a, Card> {
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

pub trait CardDraw: Sized + Copy {
    fn draw(&self, frame: &mut canvas::Frame<Renderer>);

    fn view<'a>(&'a self, cache: &'a Cache, selected: bool) -> Element<'a, Message> {
        let card = container(
            canvas::Canvas::new(CardCanvas { card: *self, cache })
                .width(Fill)
                .height(Fill),
        )
        .style(move |_theme| container::Style {
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
        });

        mouse_area(card).on_press(Message::Toggle).into()
    }
}

pub fn check_if_has_set<Card: Copy + Sum + Default + Eq>(cards: &[Card]) -> bool {
    (0..cards.len())
        .array_combinations::<3>()
        .any(|idxs| Card::default() == idxs.map(|i| cards[i]).into_iter().sum())
}

pub trait CardGen: Sized {
    fn all() -> impl Iterator<Item = Self>;
}
