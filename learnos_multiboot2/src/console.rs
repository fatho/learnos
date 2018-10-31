use crate::vga::{Vga, VgaEntry, Color};

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
    pub fn new(mut buffer: Vga) -> Console {
        buffer.clear(VgaEntry::new(Color::White, Color::Blue, 0));
        Console {
            buffer: buffer,
            x: 0,
            y: 0,
            fg: Color::White,
            bg: Color::Blue,
        }
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