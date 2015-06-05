/// Can address up to 125 ROM banks of 16KB each (i.e. 2MB of ROM maximum) and
/// supports (eventually battery-backed) RAM.

use super::{MBC, CartridgeHeader};
use super::CartridgeHeader::*;

pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl MBC1 {
    pub fn new(data: Vec<u8>) -> ::ResultStr<MBC1> {
        if data.len() > 16384 * 125 {
            return Err("MBC1 does not support more than 2MB of ROM");
        }
        let ram_size: usize = match data[CartridgeHeader::address(MBC_TYPE)] {
            // RAM
            0x02 => CartridgeHeader::ram_size(&data),
            // RAM+BATTERY
            0x03 => {
                // TODO : state loading
                CartridgeHeader::ram_size(&data)
            },
            _ => 0,
        };
        Err("")
    }
}

impl MBC for MBC1 {
    fn rom_read(&self, address: u16) -> u8 { 0x0 }
    fn ram_read(&self, address: u16) -> u8 { 0x0 }
    fn rom_control(&mut self, address: u16, value: u8) { }
    fn ram_write(&mut self, address: u16, value: u8) { }
}
