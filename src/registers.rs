use crate::constants::NUM_REGISTERS;

#[derive(Clone, Copy, Default)]
///# Holds all the registers and the sound and delay timers
pub struct Registers {
    register: [u8; NUM_REGISTERS as usize],
    vindex: u16,
    /// 0 by default, unless its set to a number then it will just start decrementing by one 60 times per
    /// second
    delay_timer: u8,
    /// Also 0, and decremented with 60hz when set to a number like the delay timer. Except the
    /// sound timer causes a beep when its not zero. So: quiet when 0, beeping when not 0
    sound_timer: u8,
}

impl Registers {
    /*pub fn new() -> Self {
        Registers {
            register: [0u8; 16],
            vindex: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }*/
    pub fn set_index_register(&mut self, value: u16) {
        self.vindex = value;
    }
    pub fn get_index_register(&self) -> u16 {
        self.vindex
    }
    pub fn set_sound_timer(&mut self, value: u8) {
        self.sound_timer = value;
    }
    pub fn get_sound_timer(&self) -> u8 {
        self.sound_timer
    }
    pub fn set_delay_timer(&mut self, value: u8) {
        self.delay_timer = value;
    }
    pub fn get_delay_timer(&self) -> u8 {
        self.delay_timer
    }
    pub fn decrement_sound_timer(&mut self) {
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
    pub fn decrement_delay_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    pub fn get_register(&self, register: u8) -> u8 {
        self.register[register as usize]
    }
    pub fn set_register(&mut self, register: u8, value: u8) {
        self.register[register as usize] = value;
    }
}
