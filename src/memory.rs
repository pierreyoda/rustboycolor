
/// This trait must be implemented by any memory bank, allowing an interface
/// independent of the nature of such a bank.
pub trait Memory {
    fn read_byte(address: u16, byte: u8);
    fn write_byte(address: u16) -> u8;
}
