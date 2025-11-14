#![allow(unused_variables, dead_code)]

///This holds all of the constants (written in capital letters in the code)
mod constants;
///An overview of all instructions in the chip 8 instruction set architecture
mod instruction;
///A data structure modeling ram
mod ram;
///The registers for the chip8 cpu
mod registers;
///The stack that is used in the cpu
mod stack;

///Handles the fetch, decode execute cycle
pub mod cpu;
///Holds the data loaded from disk
pub mod rombuffer;
