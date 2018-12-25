use crate::io;

pub const SELECT_PORT: io::PortNumber = io::PortNumber(0x70);
pub const DATA_PORT: io::PortNumber = io::PortNumber(0x71);

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct CmosRegister(pub u8);

/// Read the given CMOS register. This function is not reentrant and not thread-safe.
pub unsafe fn read_register(reg: CmosRegister) -> u8 {
    let select = io::inb(SELECT_PORT);
    io::outb(SELECT_PORT, (select & 0x80) | (reg.0 & 0x7F));
    io::inb(DATA_PORT)
}

/// Write the given CMOS register. This function is not reentrant and not thread-safe.
pub unsafe fn write_register(reg: CmosRegister, value: u8) {
    let select = io::inb(SELECT_PORT);
    io::outb(SELECT_PORT, (select & 0x80) | (reg.0 & 0x7F));
    io::outb(DATA_PORT, value);
}