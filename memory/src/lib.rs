#![cfg_attr(not(test), no_std)]

extern crate bare_metal;

pub mod mapping;

/// Number of trailing zeros in a page aligned address.
pub const PAGE_ALIGN_BITS: u32 = 12;

/// Size of a normal physical page, 4096 bytes.
pub const PAGE_SIZE: usize = 1 << PAGE_ALIGN_BITS;