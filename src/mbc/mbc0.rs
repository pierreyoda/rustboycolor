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
        if data.len() > ROM_SIZE {
            return Err("ROM size too big for MBC0");
        }

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
        if self.eram.is_some() {
            self.eram.unwrap()[(address as usize) & 0x1FFF] = value;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::mbc::CartridgeHeader;

    use super::{MBC0, ROM_SIZE};

    #[test]
    fn test_mbc0_init_success() {
        let mut data = vec![0x00; ROM_SIZE];
        data[CartridgeHeader::RAM_Size.address()] = 0x02; // 8 KB ERAM
        assert!(MBC0::from_data(data).is_ok());
    }

    #[test]
    fn test_mbc0_init_success_ram_size_zero() {
        let mut data = vec![0x00; ROM_SIZE];
        data[CartridgeHeader::RAM_Size.address()] = 0x00;
        assert!(MBC0::from_data(data).is_ok());
    }

    #[test]
    fn test_mbc0_init_error_data_size() {
        for data_size in [ROM_SIZE + 1, ROM_SIZE + 2] {
            let data = vec![0x00; data_size];
            assert!(MBC0::from_data(data).is_err());
        }
    }

    #[test]
    fn test_mbc0_init_error_rom_size_overflow() {
        let rom_data = MBC0::from_data([0x00; 0].to_vec());
        assert!(rom_data.is_err());
    }

    #[test]
    fn test_rom0_init_error_ram_size_0() {
        let rom_data = MBC0::from_data([0x00; 0].to_vec());
        assert!(rom_data.is_err());
    }

    #[test]
    fn test_mbc0_init_error_invalid_eram_size() {
        // 0x01 = 2 KB, 0x03 = 32 KB: valid Game Boy sizes but unsupported by MBC0
        for invalid_ram_size_byte in [0x01, 0x03] {
            let mut data = vec![0x00; ROM_SIZE];
            data[CartridgeHeader::RAM_Size.address()] = invalid_ram_size_byte;
            assert!(MBC0::from_data(data).is_err());
        }
    }
}
