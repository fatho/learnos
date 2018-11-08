//! Provides an interface to the serial COM ports.

use core::fmt;

use super::PortNumber;

/// The usual address of the COM1 port.
pub const COM1_ADDR: PortNumber = PortNumber(0x3F8);

/// A safe interface to a serial port identified by its base port number.
#[derive(Debug, Eq, PartialEq)]
pub struct SerialPort(PortNumber);

impl SerialPort {
    /// Creates a new handle to a serial port. This is unsafe for several reason:
    ///   1. some ports allow access to hardware that safe code shouldn't have
    ///   2. it would allow multiple threads to concurrently access the same port
    ///   3. is only safe to use with COM ports
    /// 
    /// Therefore, the caller must make sure that writing to this port can do no harm (e.g. writing to COM1),
    /// must ensure that it won't instantiate the same port twice, and that the port number refers to a COM port.
    pub const unsafe fn new(port_number: PortNumber) -> SerialPort {
        // TODO: perform additional initialization (baud rate etc.) for the COM port
        SerialPort(port_number)
    }

    #[inline]
    pub fn write(&mut self, data: &[u8]) {
        unsafe {
            super::outsb(self.0, data);
        }
    }

    #[inline]
    pub fn write_byte(&mut self, data: u8) {
        unsafe {
            super::outb(self.0, data);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}