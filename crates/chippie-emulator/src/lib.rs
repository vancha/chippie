#![allow(unused_variables, dead_code)]

use std::{fmt, str::FromStr};

mod constants;
mod cpu;
mod instruction;
mod ram;
mod registers;
mod rombuffer;
mod stack;

// Re-export structs and modules that migth be used by graphics libraries
pub use constants::NUM_KEYS;
pub use cpu::Cpu;
pub use rombuffer::RomBuffer;

/// A custom type which describes possible screen resolutions for the emulator.
#[derive(Clone, Copy, Default)]
pub enum ScreenResolution {
    #[default]
    Basic,
    ETI,
    SuperChip,
    MegaChip,
}

impl ScreenResolution {
    /// Get the display dimensions out of the resolution instance
    pub fn size(&self) -> (u16, u16) {
        match self {
            Self::Basic => (64, 32),
            Self::ETI => (64, 64),
            Self::SuperChip => (128, 64),
            Self::MegaChip => (256, 192),
        }
    }

    /// Get display width
    pub fn width(&self) -> u16 {
        self.size().0
    }

    /// Get display height
    pub fn height(&self) -> u16 {
        self.size().1
    }
}

impl fmt::Display for ScreenResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width(), self.height())
    }
}

impl FromStr for ScreenResolution {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "64x32" => Ok(Self::Basic),
            "64x64" => Ok(Self::ETI),
            "128x64" => Ok(Self::SuperChip),
            "256x192" => Ok(Self::MegaChip),
            _ => Err(()),
        }
    }
}

/// The struct that stores information about pixels, which is then used to draw the image on the
/// screen.
///
/// This struct is meant to be shared between the emulator instance and the instance of
/// graphical framework's wrapper, so that the emulator can read and write to this buffer, whereas
/// the graphical framework has only read-only access to it.
pub struct Framebuffer {
    resolution: ScreenResolution,
    data: Vec<bool>,
}

impl Framebuffer {
    /// Create a new framebuffer with the given resolution
    pub fn new(resolution: ScreenResolution) -> Self {
        let size = (resolution.width() * resolution.height()) as usize;

        Self {
            resolution,
            data: vec![false; size],
        }
    }

    /// Get the framebuffer's resolution
    pub fn resolution(&self) -> ScreenResolution {
        self.resolution
    }

    /// Clear the framebuffer
    pub fn clear(&mut self) {
        let size = (self.resolution.width() * self.resolution.height()) as usize;
        self.data = vec![false; size];
    }

    /// Get the value of the pixel with the given X and Y positions
    pub fn get(&self, x: u16, y: u16) -> bool {
        let index = (x * self.resolution.width() + y) as usize;
        assert!(index < self.data.len());

        self.data[index]
    }

    /// Change the value of the pixel, which has the given coordinates
    pub fn set(&mut self, x: u16, y: u16, value: bool) {
        let index = (x * self.resolution.width() + y) as usize;
        assert!(index < self.data.len());

        self.data[index] = value;
    }
}
