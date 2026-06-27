use iced::{
    Border, Color, Element, Length, Point, Rectangle, Renderer, Theme, mouse,
    widget::{
        Container,
        canvas::{self, Path},
        container,
    },
};

use crate::Message;

const DOT_RADIUS_RATIO: f32 = 0.15;

const CARD_COLORS: [Color; 6] = [
    Color::from_rgb8(0xFF, 0x00, 0x00),
    Color::from_rgb8(0xFF, 0xA5, 0x00),
    Color::from_rgb8(0xFF, 0xD7, 0x00),
    Color::from_rgb8(0x00, 0x80, 0x00),
    Color::from_rgb8(0x00, 0x00, 0xFF),
    Color::from_rgb8(0x80, 0x00, 0x80),
];

#[derive(Debug, Clone, Copy)]
pub struct ProjectiveCard {
    pub mask: u8,
}

impl ProjectiveCard {
    pub fn view<'a>(self, selected: bool) -> Element<'a, Message> {
        container(
            canvas::Canvas::new(self)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .style(move |_theme| container::Style {
            background: Some(
                if selected {
                    Color::from_rgb8(0x71, 0x77, 0x7F)
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
        })
        .into()
    }
}

impl<Message> canvas::Program<Message> for ProjectiveCard {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let radius = bounds.width * DOT_RADIUS_RATIO;
        for (row, y) in [0.18333334, 0.48333335, 0.78333336].iter().enumerate() {
            for (col, x) in [0.275, 0.725].iter().enumerate() {
                let idx = row * 2 + col;
                if self.mask & (1 << idx) == 0 {
                    continue;
                }

                let center = Point::new(bounds.width * x, bounds.height * y);
                let dot = Path::circle(center, radius);
                frame.fill(&dot, CARD_COLORS[idx]);
            }
        }

        vec![frame.into_geometry()]
    }
}
