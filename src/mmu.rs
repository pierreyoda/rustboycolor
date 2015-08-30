use super::bios::GB_BIOS;
use super::memory::Memory;
use super::gpu::Gpu;
use super::mbc::{MBC};

const WRAM_SIZE: usize = 0x2000;
const ZRAM_SIZE: usize = 0x0080;

/// The Game Boy (Color)'s Memory Management Unit, interfacing between
/// its CPU and the different memory components (RAM, ROM banks...).
/// Responsible for switching between the different ROM and RAM banks.
///
/// For now the MMU struct actually owns the different components, effectively
/// becoming the central hub of the hardware. References may be used in the
/// future instead.
pub struct MMU {
    /// Internal flag to handle the BIOS loading.
    in_bios: bool,
    /// The BIOS file to execute when starting the emulation.
    bios: &'static [u8],
    /// GPU.
    gpu: Gpu,
    /// The MBC interfacing with the cartridge ROM and (optionally) RAM banks.
    mbc: Box<MBC + 'static>,
    /// 8K of internal working RAM.
    wram: [u8; WRAM_SIZE],
    ///'Zero-page' RAM of 128 bytes.
    zram: [u8; ZRAM_SIZE],
    /// Interrupt Enable Register.
    ie_reg: u8,
    /// Interrupt Flag Register.
    if_reg: u8,
}

impl MMU {
    pub fn new(mbc: Box<MBC>) -> MMU {
        MMU {
            in_bios: true,
            bios: &GB_BIOS,
            gpu: Gpu::new(),
            mbc: mbc,
            wram: [0x0; WRAM_SIZE],
            zram: [0x0; ZRAM_SIZE],
            ie_reg: 0x00,
            if_reg: 0x00,
        }
    }
}

// MMU implements the Memory trait to provide transparent interfacing
// with the CPU.
impl Memory for MMU {
    fn read_byte(&mut self, address: u16) -> u8 {
        let a = address as usize;
        match a {
            // BIOS mode
            _ if self.in_bios => {
                if address < 0x100 {
                    self.bios[a]
                } else if address == 0x100 {
                    self.in_bios = false;
                    info!("MMU : leaving the BIOS");
                    self.read_byte(address)
                } else {
                    error!("MMU : BIOS overflow, leaving the BIOS");
                    self.in_bios = false;
                    self.read_byte(address)
                }
            },
            // cartridge ROM
            0x0000 ... 0x7FFF => self.mbc.rom_read(address),
            // GPU : background and sprite data
            0x8000 ... 0x9FFF => self.gpu.read_byte(address),
            // cartridge external RAM
            0xA000 ... 0xBFFF => self.mbc.ram_read(address),
            // working ram and its echo (TODO : RAM bank switch for GBC)
            0xC000 ... 0xFDFF => self.wram[a & 0x1FFF],
            // GPU : Object Attribute Memory
            0xFE00 ... 0xFE9F => self.gpu.read_byte(address),
            // not usable
            0xFEA0 ... 0xFEFF => 0x00,
            // I/O ports : TODO
            0xFF0F            => self.if_reg,
            // Zero-page RAM
            0xFF80 ... 0xFFFE => self.zram[a & 0x7F],
            // Interrupt Enable Register
            0xFFFF            => self.ie_reg,
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        let a = address as usize;
        match a {
            // cartridge ROM
            0x0000 ... 0x7FFF => self.mbc.rom_control(address, byte),
            0x8000 ... 0x9FFF => self.gpu.write_byte(address, byte),
            0xA000 ... 0xBFFF => self.mbc.ram_write(address, byte),
            0xC000 ... 0xFDFF => self.wram[a & 0x1FFF] = byte,
            0xFE00 ... 0xFE9F => self.gpu.write_byte(address, byte),
            0xFEA0 ... 0xFEFF => {},
            0xFF0F            => self.if_reg = byte,
            0xFF80 ... 0xFFFE => self.zram[a & 0x7F] = byte,
            0xFFFF            => self.ie_reg = byte,
            _ => (),
        }
    }
}
