
/// This trait must be implemented by any memory device, virtual or real,
/// allowing an interface independent of the nature of such a device.
pub trait Memory {
    fn read_byte(&mut self, address: u16) -> u8;
    fn read_word(&mut self, address: u16) -> u16;
    fn write_byte(&mut self, address: u16, byte: u8);
    fn write_word(&mut self, address: u16, word: u16);
}
