#![cfg_attr(not(test), no_std)]
#![feature(asm)]

mod align;
mod addr;

pub mod io;

pub use self::align::*;
pub use self::addr::*;