//! # chippie-common
//!
//! Common types used by other chippie's crates

use serde::{Deserialize, Serialize};

use chippie_emulator::NUM_KEYS;

/// How many cycles the cpu advances for every frame. This decides how fast the cpu will run
const DEFAULT_CYCLES_PER_FRAME: usize = 5;

pub type Keybindings = [char; NUM_KEYS as usize];

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub frame_cycles: usize,
    pub keybindings: Keybindings,
}

impl Default for Settings {
    fn default() -> Self {
        let keybindings: Keybindings = [
            '1', '2', '3', '4', 'q', 'w', 'e', 'r', 'a', 's', 'd', 'f', 'z', 'x', 'c', 'v',
        ];

        Self {
            frame_cycles: DEFAULT_CYCLES_PER_FRAME,
            keybindings,
        }
    }
}
