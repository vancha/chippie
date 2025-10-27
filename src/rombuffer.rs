// Holds the data from a chip8 file as a vec of bytes
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

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        RomBuffer { buffer: bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_files() {
        let rom_buffer = RomBuffer::new("tests/1-chip8-logo.8o");
        assert!(rom_buffer.contents()[0] == 0x23);
    }
}
