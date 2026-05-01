///! Can address up to 125 ROM banks of 16KB each (i.e. 2MB of ROM at most) and
///! supports 0, 2, 8 or 32 KB of RAM (eventually battery-buffered).
///!
///! See: https://gbdev.io/pandocs/MBC1.html

use crate::ResultStr;

use super::{CartridgeHeader, MBC};

pub const ROM_SIZE: usize = 0x200000; // 2 MB: 128 banks × 16 KiB

pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    /// The current ROM bank to use when reading in the 0x4000...0x7FFF range.
    ///
    /// Possible values are in 0x00...0x7F with the exception of 0x20, 0x40 and
    /// 0x60 which explains the limit of 125 banks (including ROM bank 0x00).
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
    /// Value:
    /// - false: "simple" (default): 0000–3FFF and A000–BFFF are locked to bank 0 of ROM and SRAM respectively
    /// - true: "advanced": 0000–3FFF and A000-BFFF can be bank-switched via the 4000–5FFF register
    ram_mode: bool,
}

impl MBC1 {
    pub fn from_data(data: Vec<u8>) -> ResultStr<MBC1> {
        if data.len() > ROM_SIZE {
            return Err("ROM size too big for MBC1");
        }

        let ram_size = match data[CartridgeHeader::MPC_TYPE.address()] {
            // RAM
            0x02 => CartridgeHeader::ram_size(&data),
            // RAM+BATTERY
            0x03 => {
                // TODO: state loading
                CartridgeHeader::ram_size(&data)
            }
            _ => 0,
        };

        Ok(MBC1 {
            rom: data,
            ram: vec![0x00; ram_size],
            rom_bank: 0x01,
            ram_bank: 0x00,
            ram_enabled: false,
            ram_mode: false,
        })
    }
}

impl MBC for MBC1 {
    fn rom_read(&self, address: u16) -> u8 {
        // ROM bank 00
        if address < 0x4000 {
            self.rom[address as usize]
        }
        // ROM bank 01-7F
        else {
            self.rom[self.rom_bank * 0x4000 + ((address as usize) & 0x3FFF)]
        }
    }

    fn ram_read(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            return 0x00;
        }
        let ram_bank = if self.ram_mode { self.ram_bank } else { 0x00 };
        self.ram[ram_bank * 0x2000 + ((address as usize) & 0x1FFF)]
    }

    fn rom_control(&mut self, address: u16, value: u8) {
        match address {
            // external RAM switch
            0x0000..=0x1FFF => {
                self.ram_enabled = value == 0x0A;
            }
            // ROM bank number : lower 5 bits
            0x2000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0x60)
                    + match (value as usize) & 0x1F {
                        0x0 => 0x1,
                        n => n,
                    };
            }
            0x4000..=0x5FFF => {
                let n = (value as usize) & 0x03;
                if self.ram_mode {
                    // RAM bank number
                    self.ram_bank = n;
                } else {
                    // bits 6 and 7 of ROM bank number in ROM mode
                    self.rom_bank = (self.rom_bank & 0x1F) | (n << 5);
                }
            }
            // ROM/RAM mode select
            0x6000..=0x7FFF => {
                self.ram_mode = value == 0x01;
            }
            _ => panic!("MBC1 : cannot write to ROM at {address:0>4X}"),
        }
    }

    fn ram_write(&mut self, address: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }
        let ram_bank = if self.ram_mode { self.ram_bank } else { 0x00 };
        self.ram[ram_bank * 0x2000 + ((address as usize) & 0x1FFF)] = value;
    }
}

#[cfg(test)]
mod test {
    use crate::mbc::{CartridgeHeader, MBC};

    use super::{MBC1, ROM_SIZE};

    // MBC1 cartridge type header values (0x0147).
    const MBC1_ROM_ONLY: u8 = 0x01;
    const MBC1_RAM: u8 = 0x02;
    const MBC1_RAM_BATTERY: u8 = 0x03;

    fn make_rom(mbc_type: u8, ram_size_byte: u8) -> Vec<u8> {
        let mut data = vec![0x00; ROM_SIZE];
        data[0x0147] = mbc_type;
        data[CartridgeHeader::RAM_Size.address()] = ram_size_byte;
        data
    }

    // from_data

    #[test]
    fn test_mbc1_init_success_no_ram() {
        assert!(MBC1::from_data(make_rom(MBC1_ROM_ONLY, 0x00)).is_ok());
    }

    #[test]
    fn test_mbc1_init_success_with_ram() {
        assert!(MBC1::from_data(make_rom(MBC1_RAM, 0x02)).is_ok());
    }

    #[test]
    fn test_mbc1_init_success_with_ram_battery() {
        assert!(MBC1::from_data(make_rom(MBC1_RAM_BATTERY, 0x02)).is_ok());
    }

    #[test]
    fn test_mbc1_init_error_rom_too_large() {
        for size in [ROM_SIZE + 1, ROM_SIZE + 2] {
            assert!(MBC1::from_data(vec![0x00u8; size]).is_err());
        }
    }

    // rom_read

    #[test]
    fn test_mbc1_rom_read_bank0() {
        // 0x0000..=0x3FFF is always fixed to bank 0.
        let mut data = make_rom(MBC1_ROM_ONLY, 0x00);
        data[0x0000] = 0xAB;
        data[0x3FFF] = 0xCD;
        let mbc = MBC1::from_data(data).unwrap();
        assert_eq!(mbc.rom_read(0x0000), 0xAB);
        assert_eq!(mbc.rom_read(0x3FFF), 0xCD);
    }

    #[test]
    fn test_mbc1_rom_read_bank_default_bank1() {
        // rom_bank initialises to 1; reads in 0x4000..=0x7FFF map to ROM[0x4000..=0x7FFF].
        let mut data = make_rom(MBC1_ROM_ONLY, 0x00);
        data[0x4000] = 0xBE;
        data[0x7FFF] = 0xEF;
        let mbc = MBC1::from_data(data).unwrap();
        assert_eq!(mbc.rom_read(0x4000), 0xBE);
        assert_eq!(mbc.rom_read(0x7FFF), 0xEF);
    }

    // ROM bank switching (0x2000..=0x3FFF)

    #[test]
    fn test_mbc1_rom_bank_select() {
        // Writing value 0x02 to 0x2000 should select bank 2 (ROM offset 0x8000).
        let mut data = make_rom(MBC1_ROM_ONLY, 0x00);
        data[0x8000] = 0x77;
        let mut mbc = MBC1::from_data(data).unwrap();
        mbc.rom_control(0x2000, 0x02);
        assert_eq!(mbc.rom_read(0x4000), 0x77);
    }

    #[test]
    fn test_mbc1_rom_bank_zero_remaps_to_one() {
        // Per spec, writing 0x00 to the ROM bank register selects bank 1 instead.
        let mut data = make_rom(MBC1_ROM_ONLY, 0x00);
        data[0x4000] = 0x11;
        let mut mbc = MBC1::from_data(data).unwrap();
        mbc.rom_control(0x2000, 0x00);
        assert_eq!(mbc.rom_read(0x4000), 0x11);
    }

    #[test]
    fn test_mbc1_rom_bank_uses_lower_5_bits_of_value() {
        // Bits 5-7 of the written value are ignored; 0x22 & 0x1F == 2 → bank 2.
        let mut data = make_rom(MBC1_ROM_ONLY, 0x00);
        data[0x4000] = 0x55; // bank 1
        data[0x8000] = 0x66; // bank 2
        let mut mbc = MBC1::from_data(data).unwrap();
        mbc.rom_control(0x2000, 0x01);
        assert_eq!(mbc.rom_read(0x4000), 0x55);
        mbc.rom_control(0x2000, 0x22); // 0x22 & 0x1F == 0x02
        assert_eq!(mbc.rom_read(0x4000), 0x66);
    }

    // RAM enable / disable (0x0000..=0x1FFF)

    #[test]
    fn test_mbc1_ram_disabled_by_default() {
        let mbc = MBC1::from_data(make_rom(MBC1_RAM, 0x02)).unwrap();
        assert_eq!(mbc.ram_read(0xA000), 0x00);
    }

    #[test]
    fn test_mbc1_ram_enable_with_0x0a() {
        let mut mbc = MBC1::from_data(make_rom(MBC1_RAM, 0x02)).unwrap();
        mbc.rom_control(0x0000, 0x0A);
        mbc.ram_write(0xA000, 0x42);
        assert_eq!(mbc.ram_read(0xA000), 0x42);
    }

    #[test]
    fn test_mbc1_ram_not_enabled_by_other_values() {
        let mut mbc = MBC1::from_data(make_rom(MBC1_RAM, 0x02)).unwrap();
        for value in [0x00u8, 0x09, 0x0B, 0xAA, 0xFF] {
            mbc.rom_control(0x0000, value);
            mbc.ram_write(0xA000, 0x42);
            assert_eq!(
                mbc.ram_read(0xA000),
                0x00,
                "RAM should stay disabled for enable byte {value:#04x}"
            );
        }
    }

    #[test]
    fn test_mbc1_ram_disable_after_enable() {
        let mut mbc = MBC1::from_data(make_rom(MBC1_RAM, 0x02)).unwrap();
        mbc.rom_control(0x0000, 0x0A);
        mbc.ram_write(0xA000, 0x42);
        mbc.rom_control(0x0000, 0x00);
        assert_eq!(mbc.ram_read(0xA000), 0x00);
    }

    #[test]
    fn test_mbc1_ram_write_ignored_when_disabled() {
        let mut mbc = MBC1::from_data(make_rom(MBC1_RAM, 0x02)).unwrap();
        mbc.ram_write(0xA000, 0x99); // disabled — must be a no-op
        mbc.rom_control(0x0000, 0x0A);
        assert_eq!(mbc.ram_read(0xA000), 0x00);
    }

    // RAM / ROM mode select (0x6000..=0x7FFF)

    #[test]
    fn test_mbc1_mode_select_to_ram_mode() {
        // Writing 0x01 to 0x6000 switches to RAM banking mode.
        // In RAM mode, the 0x4000..=0x5FFF register selects the RAM bank.
        let mut mbc = MBC1::from_data(make_rom(MBC1_RAM, 0x03)).unwrap(); // 32 KB RAM
        mbc.rom_control(0x6000, 0x01); // enter RAM mode
        mbc.rom_control(0x0000, 0x0A); // enable RAM
        mbc.rom_control(0x4000, 0x00);
        mbc.ram_write(0xA000, 0xAA); // write to bank 0
        mbc.rom_control(0x4000, 0x01);
        mbc.ram_write(0xA000, 0xBB); // write to bank 1
        mbc.rom_control(0x4000, 0x00);
        assert_eq!(mbc.ram_read(0xA000), 0xAA);
        mbc.rom_control(0x4000, 0x01);
        assert_eq!(mbc.ram_read(0xA000), 0xBB);
    }

    #[test]
    fn test_mbc1_mode_select_back_to_rom_mode() {
        // After returning to ROM mode, RAM reads always use bank 0.
        let mut mbc = MBC1::from_data(make_rom(MBC1_RAM, 0x03)).unwrap();
        mbc.rom_control(0x6000, 0x01); // RAM mode
        mbc.rom_control(0x0000, 0x0A);
        mbc.rom_control(0x4000, 0x00);
        mbc.ram_write(0xA000, 0xAA); // bank 0
        mbc.rom_control(0x4000, 0x01);
        mbc.ram_write(0xA000, 0xBB); // bank 1
        mbc.rom_control(0x6000, 0x00); // back to ROM mode
        // 0x4000 register now affects upper ROM bits, RAM is locked to bank 0
        mbc.rom_control(0x4000, 0x00); // keep upper ROM bits 0 (stay in-bounds)
        assert_eq!(mbc.ram_read(0xA000), 0xAA);
    }

    #[test]
    fn test_mbc1_ram_locked_to_bank0_in_rom_mode() {
        // In ROM mode (default), RAM reads always come from bank 0 regardless of the
        // 0x4000..=0x5FFF register.
        let mut mbc = MBC1::from_data(make_rom(MBC1_RAM, 0x03)).unwrap();
        mbc.rom_control(0x0000, 0x0A);
        mbc.ram_write(0xA000, 0xCC); // writes to bank 0 (ROM mode)
        assert_eq!(mbc.ram_read(0xA000), 0xCC);
    }
}
