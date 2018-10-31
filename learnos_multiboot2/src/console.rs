use crate::vga::{Vga, VgaEntry, Color};
use core::fmt;

pub struct Console {
    buffer: Vga,
    x: u32,
    y: u32,
    /// current foreground color
    fg: Color,
    /// current background color
    bg: Color
}

impl Console {
    pub fn new(buffer: Vga) -> Console {
        Self::with_colors(buffer, Color::White, Color::Black)
    }

    pub fn with_colors(buffer: Vga, fg: Color, bg: Color) -> Console {
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

    pub fn set_fg(&mut self, fg: Color) {
        self.fg = fg
    }

    pub fn set_bg(&mut self, bg: Color) {
        self.bg = bg
    }

    pub fn clear(&mut self) {
        self.buffer.clear(VgaEntry::new(self.fg, self.bg, 0));
    }

    pub fn write_char(&mut self, ch: u8) {
        let mut clear_next_line = false;
        if ch == b'\n' {
            self.y += 1;
            self.x = 0;
            clear_next_line = true;
        } else {
            let entry = VgaEntry::new(self.fg, self.bg, ch);
            let offset = self.x + Vga::WIDTH * self.y;
            self.buffer.write(offset as usize, entry);
            self.x += 1;
            if self.x == Vga::WIDTH {
                self.y += 1;
                self.x = 0;
                clear_next_line = true;
            }
        }
        if self.y == Vga::HEIGHT {
            self.y = 0;
        }
        if clear_next_line {
            for x in 0..Vga::WIDTH {
                let entry = VgaEntry::new(self.fg, self.bg, 0);
                let offset = self.y * Vga::WIDTH + x;
                self.buffer.write(offset as usize, entry);
            }
        }
    }

    pub fn write(&mut self, text: &[u8]) {
        for ch in text {
            self.write_char(*ch);
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