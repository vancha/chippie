//! # chippie-gui
//!
//! A GUI wrapper for the chippie-emulator crate

use std::cell::RefCell;
use std::rc::Rc;

use iced::time;
use iced::widget::column;
use iced::{Element, Fill, Subscription};

use chippie_emulator::{Cpu, DISPLAY_HEIGHT, DISPLAY_WIDTH, RomBuffer};

mod constants;
mod widgets;

/// Messages that are used for communication between iced widgets.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    /// A messages that is used as a clock source's signal
    Tick,
}

/// The main application struct, which constructs GUI and reacts on messages
pub struct Application {
    cpu: Cpu,
    display: widgets::Display,
}

impl Application {
    /// Starts the emulator and creates a window with which a user can interact
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use chippie_gui::Application;
    ///
    /// let _ = Application::run();
    /// ```
    pub fn run() -> iced::Result {
        iced::application(constants::APP_NAME, Application::update, Application::view)
            .subscription(Application::subscription)
            .run()
    }

    /// Creates a full view of the main window
    pub fn view(&self) -> Element<'_, Message> {
        column![self.display.view(),]
            .width(Fill)
            .height(Fill)
            .into()
    }

    /// The function, called by iced when there is a message, queued for this application
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => self.cpu.cycle(),
        }
    }

    /// Creates a specific task, that is run asynchronously by iced
    pub fn subscription(&self) -> Subscription<Message> {
        time::every(constants::TICK_INTERVAL).map(|_| Message::Tick)
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
