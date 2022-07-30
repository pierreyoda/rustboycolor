//! The MBC module implements the different Memory Bank Controllers and the
//! interface for the MMU to use.
//!
//! MBCs allow the game cartridge to have access to more address space by
//! explicit bank switching.
//!
//! Main reference for the implementation :
//! http://bgb.bircd.org/pandocs.htm (Pan Docs)

use std::fs::File;
use std::io::Read;
use std::path::Path;

mod mbc0;
mod mbc1;

/// Allows to access information stored in the cartridge header.
#[allow(non_camel_case_types)]
pub enum CartridgeHeader {
    MBC_Type,
    ROM_Size,
    RAM_Size,
    /// Destination code : 0x00 for Japan market, 0x01 otherwise.
    DestinationCode,
    /// Licensee (publisher) code. If equals to 0x33, the new format will
    /// be used instead (in range 0x144...0x0145).
    LicenseeCodeOld,
}

impl CartridgeHeader {
    /// Return the ROM address for the given header information, or None
    /// if the header information does not fit in a single byte and/or is
    /// unsupported.
    pub fn address(header_info: CartridgeHeader) -> Option<usize> {
        match header_info {
            MBC_Type => Some(0x0147),
            ROM_Size => Some(0x0148),
            RAM_Size => Some(0x0149),
            DestinationCode => Some(0x014A),
            LicenseeCodeOld => Some(0x014B),
        }
    }

    /// Return the RAM size in the given ROM file.
    pub fn ram_size(rom: &[u8]) -> usize {
        match rom[CartridgeHeader::address(RAM_Size).unwrap()] {
            // 2 KB
            0x01 => 0x0800,
            // 8 KB
            0x02 => 0x2000,
            // 32 KB
            0x03 => 0x8000,
            // None
            _ => 0,
        }
    }
}
use self::CartridgeHeader::*;

pub trait MBC {
    fn rom_read(&self, address: u16) -> u8;
    fn ram_read(&self, address: u16) -> u8;
    /// For some MBCs, trying to write at specific ROM addresses allows to
    /// write to the Control Registers.
    fn rom_control(&mut self, address: u16, value: u8);
    fn ram_write(&mut self, address: u16, value: u8);
}

/// Try to load a cartridge from the given filepath and return the appropriate
/// MBC with its content loaded in.
/// TODO: read cartridge information
/// TODO: cartridge header checksum validation
/// TODO: state saving with battery-backed RAM
/// TODO: take an u8 array instead to move file loading into the actual application
pub fn load_cartridge(filepath: &Path) -> crate::ResultStr<Box<dyn MBC + Send>> {
    let mut data = Vec::<u8>::new();
    File::open(filepath)
        .and_then(|mut f| f.read_to_end(&mut data))
        .map_err(|_| "could not load the file as a gameboy ROM")?;
    match data[CartridgeHeader::address(MBC_Type).unwrap()] {
        // MBC0 : no MBC
        0x00 => {
            info!("MBC used by the cartridge : none.");
            mbc0::MBC0::new(data).map(|v| Box::new(v) as Box<dyn MBC + Send>)
        }
        // MBC1
        0x01..=0x03 => {
            info!("MBC used by the cartridge : MBC1.");
            mbc1::MBC1::new(data).map(|v| Box::new(v) as Box<dyn MBC + Send>)
        }
        _ => Err("unsupported cartridge MBC"),
    }
}
