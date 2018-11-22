#![cfg_attr(not(test), no_std)]
#![feature(asm)]

#[macro_use]
extern crate static_assertions;
#[macro_use]
extern crate log;

mod align;
mod addr;

pub mod cpu;
pub mod segments;
pub mod interrupts;

pub use self::align::*;
pub use self::addr::*;