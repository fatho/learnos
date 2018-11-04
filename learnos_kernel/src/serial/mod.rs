use crate::spin::Mutex;
use core::fmt;

pub const COM1_ADDR: u16 = 0x3F8;

pub static COM1: Mutex<SerialPort> = Mutex::new(SerialPort(COM1_ADDR));

/// A serial port identified by its port number.
#[derive(Debug, Eq, PartialEq)]
pub struct SerialPort(u16);

impl SerialPort {
    /// Creates a new handle to a serial port.
    /// This is unsafe because it would allow multiple threads to concurrently access the same port.
    pub const unsafe fn new(port_number: u16) -> SerialPort {
        SerialPort(port_number)
    }

    #[inline]
    pub fn write(&mut self, data: &[u8]) {
        unsafe {
            let first_byte = data.as_ptr();
            asm!("rep outsb" : : "{rsi}"(first_byte), "{dx}"(self.0), "{rcx}"(data.len()) :  : "intel")
        }
    }

    #[inline]
    pub fn write_byte(&mut self, data: u8) {
        unsafe {
            asm!("out dx, al" : : "{dx}"(self.0), "{al}"(data) :  : "intel")
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}