use spin::Mutex;
use amd64::io::com::{COM1_ADDR, SerialPort};

pub static COM1: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(COM1_ADDR) });
