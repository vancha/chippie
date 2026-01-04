#![allow(unused_variables, dead_code)]

///This holds all of the constants (written in capital letters in the code)
mod constants;
///Handles the fetch, decode execute cycle
mod cpu;
///An overview of all instructions in the chip 8 instruction set architecture
mod instruction;
///A data structure modeling ram
mod ram;
///The registers for the chip8 cpu
mod registers;
///Holds the data loaded from disk
mod rombuffer;
///The stack that is used in the cpu
mod stack;

// Re-export structs and modules that migth be used by graphics libraries
pub use constants::NUM_KEYS;
pub use cpu::Cpu;
pub use rombuffer::RomBuffer;

pub type Keyboard = [bool; NUM_KEYS as usize];

// A custom type which describes possible screen resolutions for the emulator
#[derive(Clone, Copy, Default)]
pub enum ScreenResolution {
    #[default]
    Basic,
    ETI,
    SuperChip,
    MegaChip,
}

impl ScreenResolution {
    pub fn size(&self) -> (u16, u16) {
        match self {
            ScreenResolution::Basic => (64, 32),
            ScreenResolution::ETI => (64, 64),
            ScreenResolution::SuperChip => (128, 64),
            ScreenResolution::MegaChip => (256, 192),
        }
    }

    pub fn width(&self) -> u16 {
        self.size().0
    }

    pub fn height(&self) -> u16 {
        self.size().1
    }
}

// The struct that stores information about pixels
pub struct Framebuffer {
    resolution: ScreenResolution,
    data: Vec<bool>,
}

impl Framebuffer {
    pub fn new(resolution: ScreenResolution) -> Self {
        let size = (resolution.width() * resolution.height()) as usize;

        Self {
            resolution,
            data: vec![false; size],
        }
    }

    pub fn resolution(&self) -> ScreenResolution {
        self.resolution
    }

    pub fn clear(&mut self) {
        let size = (self.resolution.width() * self.resolution.height()) as usize;
        self.data = vec![false; size];
    }

    pub fn get(&self, x: u16, y: u16) -> bool {
        let index = (x * self.resolution.width() + y) as usize;
        assert!(index < self.data.len());

        self.data[index]
    }

    pub fn set(&mut self, x: u16, y: u16, value: bool) {
        let index = (x * self.resolution.width() + y) as usize;
        assert!(index < self.data.len());

        self.data[index] = value;
    }
}
