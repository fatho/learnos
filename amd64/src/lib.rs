#![cfg_attr(not(test), no_std)]
#![feature(asm)]

#[macro_use]
extern crate static_assertions;
#[macro_use]
extern crate log;

mod align;
mod addr;

pub mod segments;
pub mod interrupts;
pub mod util;
pub mod idt;
pub mod pic;
pub mod apic;
pub mod ioapic;
pub mod msr;
pub mod io;
pub mod cpuid;

pub use self::align::*;
pub use self::addr::*;

#[inline(always)]
pub unsafe fn hlt() {
    asm!("hlt" : : : : "intel", "volatile");
}
