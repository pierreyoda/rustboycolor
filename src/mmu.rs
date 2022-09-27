use crate::apu::{Apu, ApuChannel};
use crate::bios::GB_BIOS;
use crate::cpu::CycleType;
use crate::gpu::{Gpu, RGB};
use crate::irq::{Interrupt, IrqHandler};
use crate::joypad::{Joypad, JoypadKey};
use crate::mbc::MBC;
use crate::memory::Memory;
use crate::serial::{Serial, SerialCallback};

use self::timers::Timers;

mod timers;

const WRAM_SIZE: usize = 0x2000;
const ZRAM_SIZE: usize = 0x0080;

pub trait MemoryManagementUnit {
    fn step(&mut self, ticks: CycleType) -> CycleType;
    fn interrupt_enable(&self) -> u8;
    fn interrupt_flag(&self) -> u8;
    fn set_interrupt_flag(&mut self, flag: u8);
}

/// The Game Boy (Color)'s Memory Management Unit, interfacing between
/// its CPU and the different memory components (RAM, ROM banks...).
///
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
    /// Timers.
    timers: Timers,
    /// GPU.
    gpu: Gpu,
    /// APU.
    apu: Apu,
    /// The MBC interfacing with the cartridge ROM and (optionally) RAM banks.
    mbc: Box<dyn MBC + 'static>,
    /// The joypad controller.
    joypad: Joypad,
    /// The serial port.
    serial: Serial,
    /// Interrupt Request handler.
    irq_handler: MachineIrqHandler,
    /// 8K of internal working RAM.
    wram: [u8; WRAM_SIZE],
    ///'Zero-page' RAM of 128 bytes.
    zram: [u8; ZRAM_SIZE],
}

/// MMU sub-component passed around to throw interrupt requests from various
/// components.
struct MachineIrqHandler {
    /// Interrupt Enable Register.
    ie_reg: u8,
    /// Interrupt Flag Register.
    if_reg: u8,
}

impl MachineIrqHandler {
    pub fn new() -> MachineIrqHandler {
        MachineIrqHandler {
            ie_reg: 0x00,
            if_reg: 0x00,
        }
    }
}

impl IrqHandler for MachineIrqHandler {
    fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.if_reg |= interrupt as u8;
    }
}

impl MMU {
    pub fn new(
        mbc: Box<dyn MBC>,
        cgb_mode: bool,
        skip_bios: bool,
        serial_callback: Option<SerialCallback>,
    ) -> MMU {
        MMU {
            in_bios: !skip_bios,
            bios: &GB_BIOS,
            timers: Timers::default(),
            gpu: Gpu::new(cgb_mode),
            // TODO: determine default channel?
            apu: Apu::new(ApuChannel::SweepAndEnvelope),
            mbc,
            joypad: Joypad::default(),
            serial: Serial::new(serial_callback),
            irq_handler: MachineIrqHandler::new(),
            wram: [0x0; WRAM_SIZE],
            zram: [0x0; ZRAM_SIZE],
        }
    }

    pub fn key_down(&mut self, key: &JoypadKey) {
        self.joypad.key_down(key, &mut self.irq_handler);
    }

    pub fn key_up(&mut self, key: &JoypadKey) {
        self.joypad.key_up(key);
    }

    /// If the GPU's framebuffer is marked as dirty, return it
    /// and set its dirty flag as false.
    pub fn frame_buffer(&mut self) -> Option<Vec<RGB>> {
        if self.gpu.dirty {
            self.gpu.dirty = false;
            Some(self.gpu.screen_data())
        } else {
            None
        }
    }
}

impl MemoryManagementUnit for MMU {
    fn step(&mut self, ticks: CycleType) -> CycleType {
        self.timers.cycle(ticks, &mut self.irq_handler);
        let gpu_ticks = ticks; // TODO: DMA
        self.gpu.step(gpu_ticks, &mut self.irq_handler);
        gpu_ticks
    }

    fn interrupt_enable(&self) -> u8 {
        self.irq_handler.ie_reg
    }
    fn interrupt_flag(&self) -> u8 {
        self.irq_handler.if_reg
    }
    fn set_interrupt_flag(&mut self, flag: u8) {
        self.irq_handler.if_reg = flag;
    }
}

// MMU implements the Memory trait to provide transparent interfacing
// with the CPU.
impl Memory for MMU {
    fn read_byte(&mut self, address: u16) -> u8 {
        let a = address as usize;
        match a {
            // BIOS mode
            _ if self.in_bios => match address {
                v if v < 0x100 => self.bios[a],
                0x100 => {
                    self.in_bios = false;
                    info!("MMU : leaving the BIOS");
                    self.read_byte(address)
                }
                _ => {
                    error!("MMU : BIOS overflow, leaving the BIOS");
                    self.in_bios = false;
                    self.read_byte(address)
                }
            },
            // cartridge ROM
            0x0000..=0x7FFF => self.mbc.rom_read(address),
            // GPU : background and sprite data
            0x8000..=0x9FFF => self.gpu.read_byte(address),
            // cartridge external RAM
            0xA000..=0xBFFF => self.mbc.ram_read(address),
            // working ram and its echo (TODO: RAM bank switch for GBC)
            0xC000..=0xFDFF => self.wram[a & 0x1FFF],
            // GPU : Object Attribute Memory
            0xFE00..=0xFE9F => self.gpu.read_byte(address),
            // not usable
            0xFEA0..=0xFEFF => 0x00,
            // joypad
            0xFF00 => self.joypad.read_byte(address),
            // SB - Serial Transfer Data
            0xFF01 => self.serial.read_data(),
            // SC - Serial Transfer Control
            0xFF02 => self.serial.read_control(),
            // timers
            0xFF04..=0xFF07 => self.timers.read_byte(address),
            // APU
            0xFF10..=0xFF14 => self.apu.read_byte(address),
            0xFF16..=0xFF26 => self.apu.read_byte(address),
            0xFF1A..=0xFF1E => self.apu.read_byte(address),
            0xFF30..=0xFF3F => self.apu.read_byte(address),
            // Interrupt Flag Register
            0xFF0F => self.irq_handler.if_reg,
            // GPU registers
            0xFF40..=0xFF4F => self.gpu.read_byte(address),
            // GPU registers (CGB mode)
            0xFF68..=0xFF6B => self.gpu.read_byte(address),
            // Zero-page RAM
            0xFF80..=0xFFFE => self.zram[a & 0x7F],
            // Interrupt Enable Register
            0xFFFF => self.irq_handler.ie_reg,
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        let a = address as usize;
        match a {
            // cartridge ROM
            0x0000..=0x7FFF => self.mbc.rom_control(address, byte),
            0x8000..=0x9FFF => self.gpu.write_byte(address, byte),
            0xA000..=0xBFFF => self.mbc.ram_write(address, byte),
            0xC000..=0xFDFF => self.wram[a & 0x1FFF] = byte,
            0xFE00..=0xFE9F => self.gpu.write_byte(address, byte),
            0xFEA0..=0xFEFF => {}
            0xFF00 => self.joypad.write_byte(address, byte),
            0xFF01 => self.serial.write_data(byte),
            0xFF02 => self.serial.write_control(byte),
            0xFF04..=0xFF07 => self.timers.write_byte(address, byte),
            0xFF10..=0xFF14 => self.apu.write_byte(address, byte),
            0xFF16..=0xFF26 => self.apu.write_byte(address, byte),
            0xFF1A..=0xFF1E => self.apu.write_byte(address, byte),
            0xFF30..=0xFF3F => self.apu.write_byte(address, byte),
            0xFF0F => self.irq_handler.if_reg = byte,
            0xFF40..=0xFF4F => self.gpu.write_byte(address, byte),
            0xFF68..=0xFF6B => self.gpu.write_byte(address, byte),
            0xFF80..=0xFFFE => self.zram[a & 0x7F] = byte,
            0xFFFF => self.irq_handler.ie_reg = byte,
            _ => (),
        }
    }
}
