use iced::{Renderer, Theme};

pub mod shape;

pub type Element<'a, Message> = iced::Element<'a, Message, Theme, Renderer>;
