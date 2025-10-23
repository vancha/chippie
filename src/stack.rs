/// 16 16-bit addresses, used to call subroutines or functions and return from them
/// can go into 16 nested subroutines before stack overflows
#[derive(Clone, Copy)]
pub struct Stack {
    values: [u16; 16],
}
impl Stack {
    pub fn new() -> Self {
        Stack { values: [0; 16] }
    }
    pub fn get(&self, idx: u8) -> u16 {
        self.values[idx as usize]
    }

    pub fn set(&mut self, idx: u8, value: u16) {
        self.values[idx as usize] = value;
    }
}
