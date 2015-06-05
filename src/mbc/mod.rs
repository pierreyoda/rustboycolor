/// The MBC module implements the different Memory Bank Controllers and the
/// interface for the MMU to use.
/// MBCs allow the game cartridge to have access to more address space by
/// explicit bank switching.

use std::path::Path;
use std::io::Read;
use std::fs::File;

mod mbc0;
mod mbc1;

/// Allows to access information stored in the cartridge header.
pub enum CartridgeHeader {
    MBC_TYPE,
    ROM_SIZE,
    RAM_SIZE,
    /// Destination code : 0x00 for Japan market, 0x01 otherwise.
    DESTINATION,
    /// Licensee (publisher) code. If equals to 0x33, the new format will
    /// be used instead (in range 0x144...0x0145).
    LICENSEE_OLD,
}

impl CartridgeHeader {
    /// Return the ROM address for the given header information.
    /// NB : only makes sense for single-byte information.
    pub fn address(header_info: CartridgeHeader) -> usize {
        match header_info {
            MBC_TYPE => 0x0147,
            ROM_SIZE => 0x0148,
            RAM_SIZE => 0x0149,
            DESTINATION => 0x014A,
            LICENSEE_OLD => 0x014B,
        }
    }

    /// Return the RAM size in the given ROM file.
    pub fn ram_size(rom: &Vec<u8>) -> usize {
        match rom[CartridgeHeader::address(RAM_SIZE)] {
            // 2 KB
            0x01 => 0x0800,
            // 8 KB
            0x02 => 0x2000,
            // 32 KB
            0x03 => 0x8000,
            // None
            _    => 0,
        }
    }
}
use self::CartridgeHeader::*;

pub trait MBC {
    fn rom_read(&self, address: u16) -> u8;
    fn ram_read(&self, address: u16) -> u8;
    /// For some MBCs, trying to write at specific ROM addresses allow it
    /// to switch the ROM banks.
    fn rom_control(&mut self, address: u16, value: u8);
    fn ram_write(&mut self, address: u16, value: u8);
}

/// Try to load a cartridge from the given filepath and return the appropriate
/// MBC with its content loaded in.
/// TODO : read cartridge information
/// TODO : cartridge header checksum validation
/// TODO : state saving with battery-backed RAM
pub fn load_cartridge(filepath: &Path) -> ::ResultStr<Box<MBC + 'static>> {

    let mut data = Vec::<u8>::new();
    try!(File::open(filepath).and_then(|mut f| f.read_to_end(&mut data))
         .map_err(|_| "could not load the file as a gameboy ROM"));
    match data[CartridgeHeader::address(MBC_TYPE)] {
        // MBC0 : no MBC
        0x00 => mbc0::MBC0::new(data).map(|v| Box::new(v) as Box<MBC>),
        // MBC1
        0x01 ... 0x03 => mbc1::MBC1::new(data).map(|v| Box::new(v) as Box<MBC>),
        _    => Err("unsupported cartridge MBC")
    }
}
