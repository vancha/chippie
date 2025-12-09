//! # chippie-gui
//!
//! A GUI wrapper for the chippie-emulator crate

use std::cell::RefCell;
use std::rc::Rc;

use iced::keyboard;
use iced::time;
use iced::widget::{button, column};
use iced::{Element, Fill, Subscription, Task};
use iced_aw::menu::{Item, Menu, MenuBar};
use rfd::{AsyncFileDialog, FileHandle};

use chippie_emulator::{Cpu, DISPLAY_HEIGHT, DISPLAY_WIDTH, NUM_KEYS, RomBuffer};

mod constants;
use constants::CYCLES_PER_FRAME;
mod widgets;

/// Messages that are used for communication between iced widgets.
#[derive(Debug, Clone)]
pub enum Message {
    /// A message that is used as a clock source's signal
    Tick,
    KeyboardEvent(iced::keyboard::Event),
    FileSelectButtonClicked,
    FileSelected(Option<FileHandle>),
    PauseRequested,
    ResumeRequested,
}

/// The main application struct, which constructs GUI and reacts on messages
pub struct Application {
    cpu: Cpu,
    display: widgets::Display,
    initialized: bool,
    running: bool,
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
        // Create a menu bar, used to control the state of the emulator
        let bar = MenuBar::new(vec![
            Item::with_menu(
                button("File"),
                Menu::new(vec![Item::new(
                    button("Select Rom")
                        .on_press(Message::FileSelectButtonClicked)
                        .width(Fill),
                )])
                .width(180.0),
            ),
            Item::with_menu(
                button("Emulation"),
                Menu::new(vec![
                    Item::new(
                        button("Resume")
                            .on_press_maybe(if self.initialized && !self.running {
                                Some(Message::ResumeRequested)
                            } else {
                                None
                            })
                            .width(Fill),
                    ),
                    Item::new(
                        button("Pause")
                            .on_press_maybe(if self.running {
                                Some(Message::PauseRequested)
                            } else {
                                None
                            })
                            .width(Fill),
                    ),
                ]),
            ),
        ]);

        column![bar, self.display.view()]
            .width(Fill)
            .height(Fill)
            .into()
    }

    /// The function, called by iced when there is a message, queued for this application
    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Tick => {
                if self.running {
                    for _ in 0..CYCLES_PER_FRAME {
                        self.cpu.cycle();
                    }
                    self.cpu.decrement_timers();
                }
            }
            Message::KeyboardEvent(event) => match event {
                iced::keyboard::Event::KeyPressed { key, .. } => {
                    if self.running
                        && let Some(i) = Self::to_index(key)
                    {
                        self.cpu.set_key_state(i, true)
                    }
                }
                iced::keyboard::Event::KeyReleased { key, .. } => {
                    if self.running
                        && let Some(i) = Self::to_index(key)
                    {
                        self.cpu.set_key_state(i, false)
                    }
                }
                _ => {}
            }
            Message::FileSelectButtonClicked => {
                // Pause the execution
                self.pause();

                return Task::perform(
                    AsyncFileDialog::new()
                        .add_filter("Chip8 ROM files".to_string(), &["ch8", "8o"])
                        .pick_file(),
                    Message::FileSelected,
                );
            }
            Message::FileSelected(Some(file)) => {
                let rom = RomBuffer::new(file.path().to_str().unwrap());
                self.cpu.load(&rom);
                self.cpu.reset();

                self.initialized = true;
                self.resume();
            }
            Message::FileSelected(None) => {
                self.resume();
            }
            Message::PauseRequested => self.pause(),
            Message::ResumeRequested => self.resume(),
        }

        Task::none()
    }

    /// Creates a specific task, that is run asynchronously by iced
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            keyboard::listen().map(Message::KeyboardEvent),
            time::every(constants::TICK_INTERVAL).map(|_| Message::Tick),
        ])
    }

    /// This function pauses the execution of the program
    fn pause(&mut self) {
        self.running = false;
    }

    /// This function resumes the execution of the program
    fn resume(&mut self) {
        if self.initialized {
            self.running = true;
        }
    }

    /// The function is used to convert iced::keyboard::Key values to key indexes, used inside the
    /// emulator
    fn to_index(key: keyboard::Key) -> Option<u8> {
        match key {
            keyboard::Key::Character(ch) => {
                if let Ok(index) = u8::from_str_radix(ch.as_str(), 16) {
                    if index < NUM_KEYS { Some(index) } else { None }
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

        Self {
            cpu: Cpu::new(Rc::clone(&framebuffer)),
            display: widgets::Display::new(
                DISPLAY_HEIGHT.into(),
                DISPLAY_WIDTH.into(),
                framebuffer,
            ),
            initialized: false,
            running: false,
        }
    }
}
