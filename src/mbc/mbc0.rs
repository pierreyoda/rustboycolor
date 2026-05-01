///! Used by small games who can fit in the 32 KB of ROM and the GameBoy's 8 KB
///! of external RAM (optional).

use super::{CartridgeHeader, MBC};

pub const ROM_SIZE: usize = 0x10000;
/// 8 KB RAM (optional).
pub const ERAM_SIZE: usize = 0x2000;

/// Memory layout for cartridges without a memory bank controller.
///
/// According to Pan Docs, plain ROM-only cartridges expose up to 32 KB of ROM
/// and may optionally provide a single 8 KB external RAM bank.
///
/// See: https://gbdev.io/pandocs/nombc.html
pub struct MBC0 {
    /// Full ROM image mapped directly at `0x0000..=0x7FFF`.
    rom: [u8; ROM_SIZE],
    /// Optional external RAM mapped at `0xA000..=0xBFFF`.
    eram: Option<[u8; ERAM_SIZE]>,
}

impl MBC0 {
    pub fn from_data(data: Vec<u8>) -> Result<MBC0, &'static str> {
        if data.is_empty() || data.len() > ROM_SIZE {
            return Err("invalid ROM size for MBC0");
        }

        let mut rom = [0x00; ROM_SIZE];
        rom[..data.len()].clone_from_slice(&data);

        let eram = match CartridgeHeader::ram_size(&data) {
            0 => None,
            ERAM_SIZE => Some([0x00; ERAM_SIZE]),
            n => {
                error!("MBC0 : invalid external RAM size of {} bytes", n);
                return Err("MBC0 supports either 0kB or 8kB of external RAM");
            }
        };
        Ok(MBC0 { rom, eram })
    }
}

impl MBC for MBC0 {
    fn rom_read(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }
    fn ram_read(&self, address: u16) -> u8 {
        if let Some(eram) = self.eram {
            let eram_address = (address as usize) & 0x1FFF;
            eram[eram_address]
        } else {
            0x00
        }
    }

    fn rom_control(&mut self, _: u16, _: u8) {}
    fn ram_write(&mut self, address: u16, value: u8) {
        if let Some(ref mut eram) = self.eram {
            eram[(address as usize) & 0x1FFF] = value;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::mbc::{CartridgeHeader, MBC};

    use super::{MBC0, ROM_SIZE};

    fn make_rom(ram_size_byte: u8) -> Vec<u8> {
        let mut data = vec![0x00; ROM_SIZE];
        data[CartridgeHeader::RAM_Size.address()] = ram_size_byte;
        data
    }

    fn make_mbc0_no_eram() -> MBC0 {
        MBC0::from_data(make_rom(0x00)).unwrap()
    }

    fn make_mbc0_with_eram() -> MBC0 {
        MBC0::from_data(make_rom(0x02)).unwrap()
    }

    // from_data

    #[test]
    fn test_mbc0_init_success_ram_size_zero() {
        assert!(MBC0::from_data(make_rom(0x00)).is_ok());
    }

    #[test]
    fn test_mbc0_init_success_ram_size_8kb() {
        assert!(MBC0::from_data(make_rom(0x02)).is_ok());
    }

    #[test]
    fn test_mbc0_init_error_empty_data() {
        assert!(MBC0::from_data(vec![]).is_err());
    }

    #[test]
    fn test_mbc0_init_error_data_too_large() {
        for size in [ROM_SIZE + 1, ROM_SIZE + 2] {
            assert!(MBC0::from_data(vec![0x00; size]).is_err());
        }
    }

    #[test]
    fn test_mbc0_init_error_invalid_eram_size() {
        // 0x01 = 2 KB, 0x03 = 32 KB: valid Game Boy sizes but unsupported by MBC0
        for invalid_ram_size_byte in [0x01u8, 0x03] {
            assert!(MBC0::from_data(make_rom(invalid_ram_size_byte)).is_err());
        }
    }

    // rom_read

    #[test]
    fn test_mbc0_rom_read_returns_correct_byte() {
        let mut data = make_rom(0x00);
        data[0x0000] = 0xAB;
        data[0x1234] = 0x42;
        data[0x7FFF] = 0xCD;
        let mbc = MBC0::from_data(data).unwrap();
        assert_eq!(mbc.rom_read(0x0000), 0xAB);
        assert_eq!(mbc.rom_read(0x1234), 0x42);
        assert_eq!(mbc.rom_read(0x7FFF), 0xCD);
    }

    // ram_read

    #[test]
    fn test_mbc0_ram_read_no_eram_returns_zero() {
        let mbc = make_mbc0_no_eram();
        assert_eq!(mbc.ram_read(0xA000), 0x00);
        assert_eq!(mbc.ram_read(0xBFFF), 0x00);
    }

    #[test]
    fn test_mbc0_ram_read_with_eram_unwritten_returns_zero() {
        let mbc = make_mbc0_with_eram();
        assert_eq!(mbc.ram_read(0xA000), 0x00);
        assert_eq!(mbc.ram_read(0xBFFF), 0x00);
    }

    // rom_control

    #[test]
    fn test_mbc0_rom_control_is_noop() {
        let mut data = make_rom(0x00);
        data[0x0000] = 0xAB;
        let mut mbc = MBC0::from_data(data).unwrap();
        mbc.rom_control(0x0000, 0xFF);
        assert_eq!(mbc.rom_read(0x0000), 0xAB);
    }

    // ram_write

    #[test]
    fn test_mbc0_ram_write_no_eram_is_noop() {
        let mut mbc = make_mbc0_no_eram();
        mbc.ram_write(0xA000, 0x42);
        assert_eq!(mbc.ram_read(0xA000), 0x00);
    }

    #[test]
    fn test_mbc0_ram_write_with_eram_persists() {
        let mut mbc = make_mbc0_with_eram();
        mbc.ram_write(0xA000, 0x42);
        assert_eq!(mbc.ram_read(0xA000), 0x42);
    }

    #[test]
    fn test_mbc0_ram_write_address_masking() {
        // 0xA000 & 0x1FFF == 0x0000 (first cell); 0xBFFF & 0x1FFF == 0x1FFF (last cell)
        let mut mbc = make_mbc0_with_eram();
        mbc.ram_write(0xA000, 0x11);
        mbc.ram_write(0xBFFF, 0x22);
        assert_eq!(mbc.ram_read(0xA000), 0x11);
        assert_eq!(mbc.ram_read(0xBFFF), 0x22);
    }
}
