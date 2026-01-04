//! # chippie-gui
//!
//! A GUI wrapper for the chippie-emulator crate

mod constants;
mod widgets;

use std::{cell::RefCell, rc::Rc};

use iced::{
    Element, Fill, Subscription, Task, keyboard, time,
    widget::{button, column},
};
use iced_aw::menu::{Item, Menu, MenuBar};
use rfd::{AsyncFileDialog, FileHandle};

use chippie_common::Settings;
use chippie_emulator::{Cpu, Framebuffer, RomBuffer};

/// Messages that are used for communication between iced widgets.
#[derive(Debug, Clone)]
pub enum Message {
    /// A message that is used as a clock source's signal
    Tick,
    KeyboardEvent(keyboard::Event),
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
    settings: Rc<RefCell<Settings>>,
}

impl Application {
    fn new(settings: Rc<RefCell<Settings>>) -> Self {
        let framebuffer = Rc::new(RefCell::new(Framebuffer::new(settings.borrow().resolution)));

        Self {
            cpu: Cpu::new(Rc::clone(&framebuffer)),
            display: widgets::Display::new(framebuffer),
            initialized: false,
            running: false,
            settings,
        }
    }

    /// Starts the emulator and creates a window with which a user can interact
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use chippie_gui::Application;
    /// use chippie_common::Settings;
    ///
    /// let settings = Settings::default();
    /// let _ = Application::run(settings);
    /// ```
    pub fn run(settings: Rc<RefCell<Settings>>) -> iced::Result {
        iced::application(
            move || Application::new(settings.clone()),
            Application::update,
            Application::view,
        )
        .subscription(Application::subscription)
        .title(constants::APP_NAME)
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
                    let settings = self.settings.borrow();
                    for _ in 0..settings.frame_cycles {
                        self.cpu.cycle();
                    }
                    self.cpu.decrement_timers();
                }
            }
            Message::KeyboardEvent(event) => {
                if !self.running {
                    return Task::none();
                }

                let translator = |key: keyboard::Key| -> Option<usize> {
                    let settings = self.settings.borrow();
                    let mut keybindings = settings.keybindings.into_iter();
                    if let keyboard::Key::Character(data) = key {
                        let character = data.chars().next().unwrap();
                        return keybindings.position(|x| x == character);
                    }

                    None
                };

                match event {
                    keyboard::Event::KeyPressed { key, .. } => {
                        if let Some(i) = translator(key) {
                            self.cpu.set_key_state(i as u8, true)
                        }
                    }
                    keyboard::Event::KeyReleased { key, .. } => {
                        if let Some(i) = translator(key) {
                            self.cpu.set_key_state(i as u8, false)
                        }
                    }
                    _ => {}
                }
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
}
