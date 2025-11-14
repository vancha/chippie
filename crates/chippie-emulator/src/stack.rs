/// 16 16-bit addresses, used to call subroutines or functions and return from them
/// can go into 16 nested subroutines before stack overflows
#[derive(Clone, Copy, Default)]
pub struct Stack {
    values: [u16; 16],
}
impl Stack {
    pub fn get(&self, idx: u8) -> u16 {
        self.values[idx as usize]
    }

    pub fn set(&mut self, idx: u8, value: u16) {
        self.values[idx as usize] = value;
    }
}
