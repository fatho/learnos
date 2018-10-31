//! This module provides a simple text console interface on top of the VGA buffer.
//! 
//! It automatically proceeds on the next line when encountering `\n` and starts at
//! the top again when reaching the lower end of the VGA buffer.
//! 
//! It also implements `core::fmt::Write`, so that it can used with the `write!` (etc.) macros.
//! Since the VGA buffer does 

use crate::vga::{VgaMem, VgaChar, Color};
use core::fmt;

pub struct Console {
    buffer: VgaMem,
    // current output column
    x: u32,
    // current output line
    y: u32,
    /// current foreground color
    fg: Color,
    /// current background color
    bg: Color
}

impl Console {
    /// Build a new console writer on top of the VGA buffer.
    pub fn new(buffer: VgaMem) -> Console {
        Self::with_colors(buffer, Color::White, Color::Black)
    }

    /// Build a new console writer with the given intial colors.
    pub fn with_colors(buffer: VgaMem, fg: Color, bg: Color) -> Console {
        let mut con = Console {
            buffer: buffer,
            x: 0,
            y: 0,
            fg: fg,
            bg: bg,
        };
        con.clear();
        con
    }

    /// Set the colors that are used for subsequent writes.
    pub fn set_colors(&mut self, fg: Color, bg: Color) {
        self.fg = fg;
        self.bg = bg;
    }

    /// Clear the VGA buffer and reset the cursor to the top left.
    pub fn clear(&mut self) {
        self.buffer.clear(VgaChar::new(self.fg, self.bg, 0));
        self.x = 0;
        self.y = 0;
    }

    /// Write a single character.
    /// This advances the cursor one step to the right.
    /// A newline character causes the cursor to be set at the start of the next line.
    pub fn write_char(&mut self, ch: u8) {
        if ch == b'\n' {
            self.next_line();
        } else {
            let entry = VgaChar::new(self.fg, self.bg, ch);
            let offset = self.x + VgaMem::WIDTH * self.y;
            self.buffer.write(offset as usize, entry);
            self.x += 1;
            if self.x == VgaMem::WIDTH {
                self.next_line();
            }
        }
    }

    /// Write an ASCII string at the current cursor position.
    pub fn write_bytes(&mut self, text: &[u8]) {
        for ch in text {
            self.write_char(*ch);
        }
    }

    /// Advance the cursor to the next line.
    pub fn next_line(&mut self) {
        // advance cursor
        self.y += 1;
        self.x = 0;
        if self.y == VgaMem::HEIGHT {
            self.y = 0;
        }
        // clear line, so that newly written text does not mix with previous text that was still there.
        for x in 0..VgaMem::WIDTH {
            let entry = VgaChar::new(self.fg, self.bg, 0);
            let offset = self.y * VgaMem::WIDTH + x;
            self.buffer.write(offset as usize, entry);
        }
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.bytes() {
            if ch <= 0x7F {
                self.write_char(ch);
            }
        }
        Ok(())
    }
}