//! Highly dangerous functions for reinterpreting memory in various ways.

use crate::addr::VirtAddr;
use core::str;
use core::slice;

/// Create an arbitrarily long lived string slice from the given virtual address.
/// It is the responsibility of the caller to make sure that the liftetime is not too long.
pub unsafe fn str_from_addr<'a>(addr: VirtAddr, len: usize) -> Result<&'a str, str::Utf8Error> {
    str::from_utf8(slice::from_raw_parts(addr.0 as *const u8, len))
}