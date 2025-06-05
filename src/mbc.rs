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

use super::ResultStr;

mod mbc0;
mod mbc1;

/// Allows to access information stored in the cartridge header.
#[allow(non_camel_case_types)]
pub enum CartridgeHeader {
    mbc_type,
    ROM_Size,
    RAM_Size,
    /// Destination code : 0x00 for Japan market, 0x01 otherwise.
    DestinationCode,
    /// Licensee (publisher) code. If equals to 0x33, the new format will
    /// be used instead (in range 0x144...0x0145).
    LicenseeCodeOld,
}

impl CartridgeHeader {
    /// Return the ROM address from the given header information, or None
    /// if the header information does not fit in a single byte and/or is
    /// unsupported.
    pub fn address(header_info: CartridgeHeader) -> Option<u8> {
        // match header_info {
        //     mbc_type => Some(0x0147),
        //     ROM_Size => Some(0x0148),
        //     RAM_Size => Some(0x0149),
        //     DestinationCode => Some(0x014A),
        //     LicenseeCodeOld => Some(0x014B),
        //     _ => None,
        // }
        todo!()
    }

    /// Return the RAM size from the given cartridge data.
    pub fn ram_size(data: &[u8]) -> usize {
        match CartridgeHeader::address(RAM_Size) {
            // 2 KB
            Some(size) if size == 0x01 => 0x0800,
            // 8 KB
            Some(size) if size == 0x02 => 0x2000,
            // 32 KB
            Some(size) if size == 0x03 => 0x8000,
            // Not possible (see Pandoc)
            Some(_) => unreachable!(),
            // None
            None => unreachable!(),
        }
    }
}

use self::CartridgeHeader::*;

/// Memory Bank Controller trait.
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
pub fn load_cartridge(filepath: &Path) -> ResultStr<Box<dyn MBC + Send>> {
    let mut data = Vec::<u8>::new();
    File::open(filepath)
        .and_then(|mut f| f.read_to_end(&mut data))
        .map_err(|_| "could not load the file as a GameBoy (Color) ROM")?;

    // TODO: // because WIP refactoring needed in CartridgeHeader
    todo!();

    // let cartridge_data = CartridgeHeader::ram_size(&data);
    // let cartridge_header = // CartridgeHeader::address(cartridge_data).expect("error reading header address"); // TODO: avoid .expect
    // match cartridge_header {
    //     // MBC0: no MBC
    //     n if n == 0x00 => {
    //         info!("MBC used by the cartridge : none.");
    //         mbc0::MBC0::from_data(data).map(|v| Box::new(v) as Box<dyn MBC + Send>)
    //     }
    //     // MBC1
    //     n if n == 0x03 => {
    //         info!("MBC used by the cartridge : MBC1.");
    //         mbc1::MBC1::from_data(data).map(|v| Box::new(v) as Box<dyn MBC + Send>)
    //     }
    //     // MBs not yet implemented
    //     n => Err("MBC not implemented yet."),
    //     //
    //     _ => unreachable!(),
    // }
}
