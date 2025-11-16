use std::cell::RefCell;
use std::rc::Rc;

use iced::mouse::Cursor;
use iced::widget::canvas;
use iced::{Color, Element, Fill, Point, Rectangle, Renderer, Size, Theme};

use chippie_emulator::Framebuffer;

use crate::Message;

/// A custom widget based on Canvas, which draws *pixels* over a black screen in the natice CHIP-8
/// resolution.
pub struct Display {
    rows: usize,
    columns: usize,
    framebuffer: Rc<RefCell<Framebuffer>>,
}

impl Display {
    pub fn new(rows: usize, columns: usize, framebuffer: Rc<RefCell<Framebuffer>>) -> Self {
        assert!(rows == framebuffer.borrow_mut().len());
        // TODO: add checks for column sizes

        Self {
            rows,
            columns,
            framebuffer,
        }
    }

    /// Construct a canvas based on custom drawing logic
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
        let cell_size = Size::new(
            bounds.width / self.columns as f32,
            bounds.height / self.rows as f32,
        );
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Fill frames background with black color
        let background = canvas::Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&background, Color::BLACK);

        // Find all the "dark" pixels and draw black rectangles at the right places
        let framebuffer = self.framebuffer.borrow();
        for column in 0..self.columns {
            for row in 0..self.rows {
                if !framebuffer[row][column] {
                    continue;
                }

                let x = column as f32 * cell_size.width;
                let y = row as f32 * cell_size.height;
                let cell = canvas::Path::rectangle(Point::new(x, y), cell_size);
                frame.fill(&cell, Color::WHITE);
            }
        }

        vec![frame.into_geometry()]
    }
}
