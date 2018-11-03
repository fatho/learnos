//! This module provides a simple wrapper around the VGA buffer.
//! 
//! The creation of the wrapper is unsafe, because it would allow
//! concurrent modification of the same memory location, as there is
//! only one VGA buffer.

use crate::addr::{PhysAddr, VirtAddr};
use crate::spin;
use core::fmt;

mod writer;
pub use self::writer::Writer;

/// Provides a single synchronized access to the console.
pub static GLOBAL_WRITER: spin::Mutex<Option<Writer>> = spin::Mutex::new(None);

/// Intialize the global VGA subsystem.
pub fn init(vga_base: VirtAddr) {
    let mut vga = GLOBAL_WRITER.lock();
    let mem = unsafe { VgaMem::from_addr(vga_base) };
    let console = Writer::new(mem);
    *vga = Some(console);
}

pub fn writer() -> WriterHandle {
    WriterHandle
}

/// Handle to the globally synchronized VGA console.
#[derive(Debug)]
pub struct WriterHandle;

impl fmt::Write for WriterHandle {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut console_guard = GLOBAL_WRITER.lock();
        let console = (*console_guard).as_mut().ok_or(fmt::Error)?;
        
        for ch in s.bytes() {
            if ch <= 0x7F {
                console.write_char(ch);
            }
        }
        Ok(())
    }
}


/// Physical address of the VGA text buffer.
pub const VGA_PHYS_ADDR: PhysAddr = PhysAddr(0xB8000);

/// The 16 VGA colors
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
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
    /// Return the color corresponding to the given VGA code.
    pub fn from_vga(idx: u8) -> Option<Color> {
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

    /// Return the VGA code of the given color.
    pub fn to_vga(self) -> u8 {
        self as u8
    }
}

/// Entry in the VGA buffer consisting of a foreground and background color, and an 8 bit character.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct VgaChar(u16);

impl VgaChar {
    /// Create a new VGA character representation from its colors and a character.
    /// For example, a white R on a blue background:
    /// 
    /// ```
    /// let vc = VgaChar::new(Color::White, Color::Blue, b'R')
    /// ```
    pub fn new(fg: Color, bg: Color, ch: u8) -> VgaChar {
        VgaChar((ch as u16) | ((fg as u16) << 8) | ((bg as u16) << 12))
    }

    /// Extract the foreground color.
    pub fn fg(self) -> Color {
        Color::from_vga(((self.0 >> 8) & 0x0F) as u8).unwrap()
    }

    /// Extract the background color.
    pub fn bg(self) -> Color {
        Color::from_vga(((self.0 >> 12) & 0x0F) as u8).unwrap()
    }

    /// Extract the character.
    pub fn ch(self) -> u8 {
        (self.0 & 0xFF) as u8
    }
}

/// Wrapper providing access to the VGA memory area.
/// Internally, it works with the virtual address of the VGA memory,
/// as there is no means of accessing the physical memory directly in long mode.
pub struct VgaMem {
    buffer: *mut u16
}

impl VgaMem {
    /// Create a new wrapper for the VGA buffer. This is unsafe because it allows the
    /// creation of multiple instances, even though there is just one single VGA buffer.
    pub unsafe fn from_addr(virt_vga_address: VirtAddr) -> Self {
        VgaMem {
            buffer: virt_vga_address.0 as *mut u16
        }
    }

    pub const WIDTH: u32 = 80;
    pub const HEIGHT: u32 = 25;
    pub const SIZE: usize = (Self::WIDTH * Self::HEIGHT) as usize;

    /// Set every character to the same value.
    pub fn clear(&mut self, fill_entry: VgaChar) {
        for off in 0..Self::SIZE {
            unsafe { self.buffer.add(off).write_volatile(fill_entry.0) }
        }
    }

    /// Extract a colored character from the given offset.
    #[inline]
    pub fn read(&self, off: usize) -> VgaChar {
        assert!(off < Self::SIZE);
        unsafe { VgaChar(self.buffer.add(off).read_volatile()) }
    }

    /// Set the colored character at the given offset.
    #[inline]
    pub fn write(&mut self, off: usize, entry: VgaChar) {
        assert!(off < Self::SIZE);
        unsafe { self.buffer.add(off).write_volatile(entry.0) }
    }

    /// Extract the character at the given offset.
    #[inline]
    pub fn read_char(&self, off: usize) -> u8 {
        assert!(off < Self::SIZE);
        unsafe { (self.buffer.add(off) as *const u8).read_volatile() }
    }

    /// Set the character at the given offset, keeping its old color.
    #[inline]
    pub fn write_char(&mut self, off: usize, ch: u8) {
        assert!(off < Self::SIZE);
        unsafe { (self.buffer.add(off) as *mut u8).write_volatile(ch) }
    }

    /// Compute the offset in the VGA buffer for accessing the character
    /// with the given x and y coordinates.
    #[inline]
    pub fn offset_at(x: u32, y: u32) -> usize {
        (y * Self::WIDTH + x) as usize
    }
}
