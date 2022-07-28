#[macro_use]
extern crate log;

mod bios;
pub mod cpu;
pub mod gpu;
pub mod irq;
pub mod joypad;
pub mod mbc;
pub mod memory;
pub mod mmu;
pub mod serial;

/// A Result with a string literal as an error type.
pub type ResultStr<T> = Result<T, &'static str>;
