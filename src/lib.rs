#![allow(unused_variables, dead_code)]

///This holds all of the constants (written in capital letters in the code)
pub mod constants;
pub mod cpu;
mod instruction;
mod ram;
mod registers;
mod rombuffer;
mod stack;
use crate::{cpu::*, instruction::*, ram::*, rombuffer::*, stack::*};
