#![feature(asm)]
#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate static_assertions;
#[macro_use]
extern crate log;
extern crate bare_metal;

pub mod idt;
pub mod pic;
pub mod apic;

/// Enable interrupts on the current CPU.
#[inline]
pub unsafe fn enable() {
    asm!("sti" : : : : "intel", "volatile")
}

/// Disable interrupts on the current CPU.
#[inline]
pub unsafe fn disable() {
    asm!("cli" : : : : "intel", "volatile")
}