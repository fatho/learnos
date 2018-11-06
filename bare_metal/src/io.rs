//! Provides primitive operations for working with the CPUs I/O ports

use core::ops;

/// A CPU I/O port number.
pub struct PortNumber(pub u16);

impl ops::Add<u16> for PortNumber {
    type Output = PortNumber;

    fn add(self, offset: u16) -> PortNumber {
        PortNumber(self.0 + offset)
    }
}

#[inline]
pub unsafe fn outb(port: PortNumber, data: u8) {
    asm!("out dx, al" : : "{dx}"(port.0), "{al}"(data) : : "intel", "volatile" );
}

#[inline]
pub unsafe fn inb(port: PortNumber) -> u8 {
    let data: u8;
    asm!("in al, dx" : "={al}"(data) : "{dx}"(port.0) : : "intel", "volatile" );
    data
}