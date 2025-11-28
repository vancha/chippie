//! # chippie-gui
//!
//! A GUI wrapper for the chippie-emulator crate

use std::cell::RefCell;
use std::rc::Rc;

use iced::keyboard;
use iced::time;
use iced::widget::{button, column};
use iced::{Element, Fill, Subscription, Task};
use rfd::{AsyncFileDialog, FileDialog};

use chippie_emulator::{Cpu, RomBuffer, DISPLAY_HEIGHT, DISPLAY_WIDTH, NUM_KEYS};

mod constants;
mod widgets;

/// Messages that are used for communication between iced widgets.
#[derive(Debug, Clone)]
pub enum Message {
    /// A message that is used as a clock source's signal
    Tick,
    KeyPressed(keyboard::Key),
    KeyReleased(keyboard::Key),
    SelectRomButtonPressed,
    FileSelected(Option<rfd::FileHandle>),
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
        column![
            self.display.view(),
            button("Select Rom").on_press(Message::SelectRomButtonPressed)
        ]
        .width(Fill)
        .height(Fill)
        .into()
    }

    /// The function, called by iced when there is a message, queued for this application
    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Tick => self.cpu.cycle(),
            Message::KeyPressed(k) => {
                if let Some(i) = Self::to_index(k) {
                    self.cpu.set_key_state(i, true)
                }
            }
            Message::KeyReleased(k) => {
                if let Some(i) = Self::to_index(k) {
                    self.cpu.set_key_state(i, false)
                }
            }

            Message::SelectRomButtonPressed => {
                return Task::perform(AsyncFileDialog::new().pick_file(), Message::FileSelected)
            }

            Message::FileSelected(Some(path)) => {
                println!("selected the file: {:?}", path);
            }

            Message::FileSelected(None) => {
                println!("no file selected");
            }
        }

        Task::none()
    }

    /// Creates a specific task, that is run asynchronously by iced
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            keyboard::on_key_press(|key, _| Some(Message::KeyPressed(key))),
            keyboard::on_key_release(|key, _| Some(Message::KeyReleased(key))),
            time::every(constants::TICK_INTERVAL).map(|_| Message::Tick),
        ])
    }

    /// The function is used to convert iced::keyboard::Key values to key indexes, used inside the
    /// emulator
    fn to_index(key: keyboard::Key) -> Option<u8> {
        match key {
            keyboard::Key::Character(ch) => {
                if let Ok(index) = u8::from_str_radix(ch.as_str(), 16) {
                    if index < NUM_KEYS {
                        Some(index)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
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
