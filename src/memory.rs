/// This trait must be implemented by any memory device, either virtual (CPU testing harness) or "real",
/// allowing an interface independent of the nature of the device.
pub trait Memory {
    fn read_byte(&mut self, address: u16) -> u8;
    fn read_word(&mut self, address: u16) -> u16 {
        self.read_byte(address) as u16 | ((self.read_byte(address + 1) as u16) << 8)
    }

    fn write_byte(&mut self, address: u16, byte: u8);
    fn write_word(&mut self, address: u16, word: u16) {
        self.write_byte(address, (word & 0x00FF) as u8);
        self.write_byte(address + 1, ((word & 0xFF00) >> 8) as u8);
    }
}
