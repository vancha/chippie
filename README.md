# Chippie

A chip8 emulator/interpreter, written in rust. This project is intended to be a learning excersize.

The emulator now runs, and is able to execute (some) roms written for the chip8 cpu.
the user interface is now based on macroquad

## Usage:
Assuming you have rust (with cargo) installed, all you have to do is clone this repository, and run `cargo run`.
To install rust, run the `curl` command over at https://www.rust-lang.org/learn/get-started 


## Goals:
This is basically a todo list, in order, of what I plan to do with this repository
- [ ] Complete support for the basic chip8 isa.
  - [ ] Fuse
  - [ ] _Knumber knower
  - [ ] Spock Paper Scissors
  - [ ] Snek
- [ ] Implement support for quirks
- [ ] Implement support for superchip instructions
- [ ] Implement support for XO-chip instructions
- [ ] add a GUI written in iced to let a user select different roms and rebind the keys

