/// Holds the data from a chip8 file as a vec of bytes
pub struct RomBuffer {
    buffer: Vec<u8>,
}

impl RomBuffer {
    pub fn new(file: &str) -> Self {
        let buffer: Vec<u8> = std::fs::read(file).unwrap();
        RomBuffer { buffer }
    }
    pub fn contents(&self) -> &[u8] {
        &self.buffer
    }
}
