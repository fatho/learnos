#![cfg_attr(not(test), no_std)]
#![feature(asm)]
#![feature(step_trait)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
#[macro_use]
extern crate static_assertions;

extern crate amd64;

pub mod paging;
pub mod physical;
pub mod util;

/// Number of trailing zeros in a page aligned address.
pub const PAGE_ALIGN_BITS: u32 = 12;

/// Number of trailing zeros in a large page aligned address.
pub const LARGE_PAGE_ALIGN_BITS: u32 = 21;

/// Size of a normal physical page, $ KiB.
pub const PAGE_SIZE: usize = 1 << PAGE_ALIGN_BITS;

/// Size of a large physical page, 2 MiB
pub const LARGE_PAGE_SIZE: usize = 1 << LARGE_PAGE_ALIGN_BITS;
