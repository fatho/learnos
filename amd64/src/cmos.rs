use crate::io;

pub const SELECT_PORT: io::PortNumber = io::PortNumber(0x70);
pub const DATA_PORT: io::PortNumber = io::PortNumber(0x71);

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct CmosRegister(pub u8);

pub static CMOS_LOCK: spin::Mutex<()> = spin::Mutex::new(());

pub unsafe fn read_register(reg: CmosRegister) -> u8 {
    let _lock = CMOS_LOCK.lock();
    let select = io::inb(SELECT_PORT);
    io::outb(SELECT_PORT, (select & 0x80) | (reg.0 & 0x7F));
    io::inb(DATA_PORT)
}

pub unsafe fn write_register(reg: CmosRegister, value: u8) {
    let _lock = CMOS_LOCK.lock();
    let select = io::inb(SELECT_PORT);
    io::outb(SELECT_PORT, (select & 0x80) | (reg.0 & 0x7F));
    io::outb(DATA_PORT, value);
}