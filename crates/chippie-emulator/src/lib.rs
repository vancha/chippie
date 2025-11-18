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
pub use constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
pub use cpu::Cpu;
pub use rombuffer::RomBuffer;

pub type Framebuffer =
    [[bool; constants::DISPLAY_WIDTH as usize]; constants::DISPLAY_HEIGHT as usize];
