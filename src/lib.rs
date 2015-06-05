#[macro_use]
extern crate log;

mod bios;
pub mod cpu;
pub mod gpu;
pub mod keypad;
pub mod memory;
mod mbc;
pub mod mmu;
pub mod registers;

/// A Result with a string literal as an error type.
pub type ResultStr<T> = Result<T, &'static str>;
