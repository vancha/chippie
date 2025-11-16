use std::cell::RefCell;
use std::rc::Rc;

use iced::time;
use iced::widget::column;
use iced::{Element, Fill, Subscription};

use chippie_emulator::{Cpu, DISPLAY_HEIGHT, DISPLAY_WIDTH, RomBuffer};

mod widgets;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Tick,
}

pub struct Application {
    cpu: Cpu,
    display: widgets::Display,
}

impl Application {
    pub fn view(&self) -> Element<'_, Message> {
        column![self.display.view(),]
            .width(Fill)
            .height(Fill)
            .into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => self.cpu.cycle(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(time::Duration::new(1, 0)).map(|_| Message::Tick)
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
            display: widgets::Display::new(
                DISPLAY_HEIGHT.into(),
                DISPLAY_WIDTH.into(),
                framebuffer,
            ),
        }
    }
}

pub fn run() -> iced::Result {
    iced::application("Chippie", Application::update, Application::view)
        .subscription(Application::subscription)
        .run()
}
