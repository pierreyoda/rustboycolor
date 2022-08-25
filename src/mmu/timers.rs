use crate::{
    cpu::CycleType,
    irq::{Interrupt, IrqHandler},
    memory::Memory,
};

/// High-level structure replicating the Game Boy (Color)'s Timer and Divider registers behavior.
///
/// See the corresponding Pandoc page: https://gbdev.io/pandocs/Timer_and_Divider_Registers.html
pub struct Timers {
    /// 0xFF04 Divider Register (DIV).
    ///
    /// This register is incremented at a rate of 16384Hz (~16779Hz on SGB).
    /// Writing any value to this register resets it to $00.
    ///
    /// Additionally, this register is reset when executing the `stop` instruction,
    /// and only begins ticking again once `stop` mode ends.
    ///
    /// NB: The divider is affected by CGB double speed mode, and will increment at 32768Hz in double speed.
    divider: u8,
    /// Divider Register clock.
    divider_clock: TimerClock,
    /// 0xFF05 Timer Counter (TIMA).
    ///
    /// This timer is incremented at the clock frequency specified by the TAC register ($FF07).
    /// When the value overflows (exceeds $FF) it is reset to the value specified in TMA (FF06)
    /// and an interrupt is requested, as described for `modulo`.
    counter: u8,
    /// 0xFF06 Timer Modulo (TMA).
    ///
    /// When TIMA overflows, it is reset to the value in this register and an interrupt is requested.
    ///
    /// If a TMA write is executed on the same cycle as the content of TMA is transferred to TIMA due to
    /// a timer overflow, the old value is transferred to TIMA.
    modulo: u8,
    /// Timer Modulo clock.
    modulo_clock: TimerClock,
    /// 0xFF07 Timer Control (TAC).
    ///
    /// - Bit  2   - Timer Enable
    /// - Bits 1-0 - Input Clock Select
    ///   00: CPU Clock / 1024 (DMG, SGB2, CGB Single Speed Mode:   4096 Hz, SGB1:   ~4194 Hz, CGB Double Speed Mode:   8192 Hz)
    ///   01: CPU Clock / 16   (DMG, SGB2, CGB Single Speed Mode: 262144 Hz, SGB1: ~268400 Hz, CGB Double Speed Mode: 524288 Hz)
    ///   10: CPU Clock / 64   (DMG, SGB2, CGB Single Speed Mode:  65536 Hz, SGB1:  ~67110 Hz, CGB Double Speed Mode: 131072 Hz)
    ///   11: CPU Clock / 256  (DMG, SGB2, CGB Single Speed Mode:  16384 Hz, SGB1:  ~16780 Hz, CGB Double Speed Mode:  32768 Hz)
    control: u8,
}

impl Default for Timers {
    fn default() -> Self {
        Self {
            divider: 0,
            divider_clock: TimerClock::with_period(256),
            counter: 0,
            modulo_clock: TimerClock::with_period(1024),
            modulo: 0,
            control: 0,
        }
    }
}

impl Memory for Timers {
    fn read_byte(&mut self, address: u16) -> u8 {
        match address {
            0xFF04 => self.divider,
            0xFF05 => self.counter,
            0xFF06 => self.modulo,
            0xFF07 => self.control,
            _ => unreachable!(
                "mmu::Timers.read_byte(address={:0>4X}) read overflow",
                address
            ),
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0xFF04 => {
                self.divider = 0;
                self.divider_clock.reset();
            }
            0xFF05 => {
                self.counter = byte;
            }
            0xFF06 => {
                self.modulo = byte;
            }
            0xFF07 => {
                if self.control & 0x03 != (byte & 0x03) {
                    self.modulo_clock.reset();
                    self.modulo_clock.set_period(match byte & 0x03 {
                        0x00 => 1024,
                        0x01 => 16,
                        0x02 => 64,
                        0x03 => 256,
                        _ => unreachable!("mmu::Timers.write_byte(address={:0>4X} byte={:0>2X}) timer control error", address, byte),
                    });
                    self.counter = self.modulo;
                }
                self.control = byte;
            }
            _ => unreachable!(
                "mmu::Timers.write_byte(address={:0>4X} byte={:0>2X}) write overflow",
                address, byte
            ),
        }
    }
}

impl Timers {
    pub fn cycle(&mut self, ticks: CycleType, irq_handler: &mut dyn IrqHandler) {
        // increment`divider` every 256 cycles (4194304Hz / 16384Hz)
        self.divider = self
            .divider
            .wrapping_add(self.divider_clock.update(ticks) as u8);

        // increment `counter` (TIMA) if enabled
        if (self.control & 0x04) != 0x00 {
            let n = self.modulo_clock.update(ticks);
            for _ in 0..n {
                self.counter = self.counter.wrapping_add(1);
                if self.counter == 0x00 {
                    self.counter = self.modulo;
                    irq_handler.request_interrupt(Interrupt::Timer);
                }
            }
        }
    }
}

/// Increments its internal counter by 1 every `period` cycles.
struct TimerClock {
    period: CycleType,
    counter: CycleType,
}

impl TimerClock {
    pub fn with_period(period: CycleType) -> Self {
        Self { period, counter: 0 }
    }

    pub fn set_period(&mut self, period: CycleType) {
        self.period = period;
    }

    pub fn reset(&mut self) {
        self.counter = 0;
    }

    pub fn update(&mut self, cycles: CycleType) -> CycleType {
        self.counter += cycles;
        let rest = self.counter / self.period;
        self.counter %= self.period;
        rest
    }
}
