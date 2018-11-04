//! Provides primitive operations for working with the CPUs I/O ports

#[inline]
pub unsafe fn outb(port: u16, data: u8) {
    asm!("out dx, al" : : "{dx}"(port), "{al}"(data) : : "intel", "volatile" );
}

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let data: u8;
    asm!("in al, dx" : "={al}"(data) : "{dx}"(port) : : "intel", "volatile" );
    data
}