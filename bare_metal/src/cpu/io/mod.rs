//! Provides primitive operations for working with the CPUs I/O ports

use core::ops;

pub mod com;

/// A CPU I/O port number.
#[derive(Debug, Eq, PartialEq, Copy, Clone, PartialOrd, Ord)]
pub struct PortNumber(pub u16);

impl ops::Add<u16> for PortNumber {
    type Output = PortNumber;

    fn add(self, offset: u16) -> PortNumber {
        PortNumber(self.0 + offset)
    }
}

// unsafe primitives

#[inline]
pub unsafe fn outb(port: PortNumber, data: u8) {
    asm!("out dx, al" : : "{dx}"(port.0), "{al}"(data) : : "intel", "volatile" );
}

#[inline]
pub unsafe fn outsb(port: PortNumber, data: &[u8]) {
    asm!("rep outsb" : : "{rsi}"(data.as_ptr()), "{dx}"(port), "{rcx}"(data.len()) :  : "intel")
}

#[inline]
pub unsafe fn inb(port: PortNumber) -> u8 {
    let data: u8;
    asm!("in al, dx" : "={al}"(data) : "{dx}"(port.0) : : "intel", "volatile" );
    data
}