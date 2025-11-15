use iced::mouse::Cursor;
use iced::widget::canvas;
use iced::{Element, Fill, Rectangle, Renderer, Theme};

#[derive(Debug, Clone, Copy)]
enum Message {}

pub struct Display {}

impl Display {
    pub fn view(&self) -> Element<'_, Message> {
        canvas::Canvas::new(self).width(Fill).height(Fill).into()
    }
}

impl canvas::Program<Message> for Display {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry> {
        let frame = canvas::Frame::new(renderer, bounds.size());
        vec![frame.into_geometry()]
    }
}
