//! # chippie-common
//!
//! Common types used by other chippie's crates

use serde::{Deserialize, Serialize};

use chippie_emulator::{NUM_KEYS, ScreenResolution};

mod serialization;

/// How many cycles the cpu advances for every frame. This decides how fast the cpu will run
const DEFAULT_CYCLES_PER_FRAME: usize = 5;

/// A special type that describes how default PC keyboards are mapped to the standard CHIP-8
/// keyboard.
pub type Keybindings = [char; NUM_KEYS as usize];

/// A special struct that holds all the application's settings. It is used to update parameters of
/// of emulation at runtime.
///
/// This struct derives from serde::Serialize and serde::Deserialize so that the settings can be
/// preserved across different application runs.
#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(serialize_with = "serialization::serialize_resolution")]
    #[serde(deserialize_with = "serialization::deserialize_resolution")]
    pub resolution: ScreenResolution,
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
            resolution: ScreenResolution::default(),
            keybindings,
        }
    }
}
