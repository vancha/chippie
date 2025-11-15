use std::cell::RefCell;
use std::rc::Rc;

use iced::mouse::Cursor;
use iced::widget::{canvas, column};
use iced::{Color, Element, Fill, Point, Rectangle, Renderer, Theme};

use chippie_emulator::{Cpu, DISPLAY_HEIGHT, DISPLAY_WIDTH, Framebuffer, RomBuffer};

pub struct Application {
    cpu: Cpu,
    display: Display,
}

impl Application {
    pub fn view(&self) -> Element<'_, Message> {
        column![self.display.view(),]
            .width(Fill)
            .height(Fill)
            .into()
    }
}

impl Default for Application {
    fn default() -> Self {
        let framebuffer = Rc::new(RefCell::new(
            [[false; DISPLAY_WIDTH as usize]; DISPLAY_HEIGHT as usize],
        ));
        let rom = RomBuffer::default();
        Self {
            cpu: Cpu::new(&rom, Rc::clone(&framebuffer)),
            display: Display::new(framebuffer),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {}

struct Display {
    framebuffer: Rc<RefCell<Framebuffer>>,
}

impl Display {
    pub fn new(framebuffer: Rc<RefCell<Framebuffer>>) -> Self {
        Self { framebuffer }
    }

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
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // The display has to be black by default
        let background = canvas::Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&background, Color::BLACK);

        vec![frame.into_geometry()]
    }
}
