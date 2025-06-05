/// Used by small games who can fit in the 32 KB of ROM and the GameBoy's 8 KB
/// of external RAM.
use super::{CartridgeHeader, MBC};

pub const ROM_SIZE: usize = 0x10000;
pub const ERAM_SIZE: usize = 0x2000; // TODO: add comment

// TODO: add comments (check Pandoc documentation)s
pub struct MBC0 {
    rom: [u8; ROM_SIZE],
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
        todo!()
        // Ok(MBC0 { rom, eram: ram })
    }
}

impl MBC for MBC0 {
    fn rom_read(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }
    fn ram_read(&self, address: u16) -> u8 {
        // match self.eram {
        //     Some([_, eram_size]) => {}
        //     Some([size, _]) if size == 0 => {}
        //     _ => {}
        // }

        match self.eram {
            Some(_) => (address as usize) & 0x1FFF,
            None => 0x00,
        };

        if self.eram.is_some() {
            self.eram.unwrap()[(address as usize) & 0x1FFF]
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

    use super::{ERAM_SIZE, MBC0, ROM_SIZE};

    #[test]
    fn test_mbc0_init_success() {
        let data = Vec::with_capacity(ERAM_SIZE - 1);
        assert!(MBC0::from_data(data).is_ok());
    }

    #[test]
    fn test_mbc0_init_error_data_size() {
        for data_size in [ROM_SIZE, ROM_SIZE + 1] {
            let data = Vec::with_capacity(ROM_SIZE + 1);
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
    fn test_mbc0_init_error_ram_size_zero() {}

    // #[test]
    // fn test_mbc0_init_error_invalid_eram_size() {
    //     for rom_data_size in [0, 8] {
    //         let rom_data = MBC0::from_data(rom_data_size);
    //         let ram_size = CartridgeHeader::ram_size(rom_data);
    //     }
    // }
}
