#![cfg_attr(not(test), no_std)]
#![feature(asm)]

mod align;
mod addr;

pub mod cpu;
pub mod segments;

pub use self::align::*;
pub use self::addr::*;