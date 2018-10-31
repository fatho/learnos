use crate::addr::{PhysAddr, VirtAddr};

/// Physical address of the VGA text buffer.
pub const VGA_PHYS_ADDR: PhysAddr = PhysAddr(0xB8000);

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

impl Color {
    pub fn from_index(idx: u8) -> Option<Color> {
        match idx {
            0 => Some(Color::Black),
            1 => Some(Color::Blue),
            2 => Some(Color::Green),
            3 => Some(Color::Cyan),
            4 => Some(Color::Red),
            5 => Some(Color::Magenta),
            6 => Some(Color::Brown),
            7 => Some(Color::LightGray),
            8 => Some(Color::DarkGray),
            9 => Some(Color::LightBlue),
            10 => Some(Color::LightGreen),
            11 => Some(Color::LightCyan),
            12 => Some(Color::LightRed),
            13 => Some(Color::LightMagenta),
            14 => Some(Color::Yellow),
            15 => Some(Color::White),
            _ => None,
        }
    }
}

/// Entry of the VGA buffer.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct VgaEntry(u16);

impl VgaEntry {
    pub fn new(fg: Color, bg: Color, ch: u8) -> VgaEntry {
        VgaEntry((ch as u16) | ((fg as u16) << 8) | ((bg as u16) << 12))
    }

    pub fn fg(self) -> Color {
        Color::from_index(((self.0 >> 8) & 0x0F) as u8).unwrap()
    }

    pub fn bg(self) -> Color {
        Color::from_index(((self.0 >> 12) & 0x0F) as u8).unwrap()
    }

    pub fn ch(self) -> u8 {
        (self.0 & 0xFF) as u8
    }
}

pub struct Vga {
    buffer: *mut u16
}

impl Vga {
    /// Create a new wrapper for the VGA buffer. This is unsafe because it allows the
    /// creation of multiple instances, even though there is just one single VGA buffer.
    pub unsafe fn with_addr(virt_vga_address: VirtAddr) -> Self {
        Vga {
            buffer: virt_vga_address.0 as *mut u16
        }
    }

    pub const WIDTH: u32 = 80;
    pub const HEIGHT: u32 = 25;
    pub const SIZE: usize = (Self::WIDTH * Self::HEIGHT) as usize;

    pub fn clear(&mut self, fill_entry: VgaEntry) {
        for off in 0..Self::SIZE {
            unsafe { self.buffer.add(off).write_volatile(fill_entry.0) }
        }
    }

    pub fn read(&self, off: usize) -> VgaEntry {
        assert!(off < Self::SIZE);
        unsafe { VgaEntry(self.buffer.add(off).read_volatile()) }
    }

    pub fn write(&mut self, off: usize, entry: VgaEntry) {
        assert!(off < Self::SIZE);
        unsafe { self.buffer.add(off).write_volatile(entry.0) }
    }

    pub fn read_char(&self, off: usize) -> u8 {
        assert!(off < Self::SIZE);
        unsafe { (self.buffer.add(off) as *const u8).read_volatile() }
    }

    pub fn write_char(&mut self, off: usize, ch: u8) {
        assert!(off < Self::SIZE);
        unsafe { (self.buffer.add(off) as *mut u8).write_volatile(ch) }
    }
}
