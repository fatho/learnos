use crate::io;

pub const PIC1_CMD: io::PortNumber = io::PortNumber(0x0020);
pub const PIC1_DATA: io::PortNumber = io::PortNumber(0x0021);
pub const PIC2_CMD: io::PortNumber = io::PortNumber(0x00A0);
pub const PIC2_DATA: io::PortNumber = io::PortNumber(0x00A1);

/// ICW4
pub const ICW1_ICW4: u8 = 0x01;
/// Initialization
pub const ICW1_INIT: u8 = 0x10;
/// Level triggered (edge) mode
pub const ICW1_LEVEL: u8 = 0x08;

/// 8086/88 (MCS-80/85) mode
pub const ICW4_8086: u8 = 0x01;

/// Reinitialize the PICs, mapping them to the given interupt vector offsets.
pub unsafe fn remap(pic1_offset: u8, pic2_offset: u8) {
    // ICW1: start initialization in cascade mode
    io::outb(PIC1_CMD, ICW1_INIT | ICW1_ICW4);
	io::outb(PIC2_CMD, ICW1_INIT | ICW1_ICW4);
    // ICW2: write new offsets
	io::outb(PIC1_DATA, pic1_offset);
	io::outb(PIC2_DATA, pic2_offset);
    // ICW3: setup master/slave connection
	io::outb(PIC1_DATA, 1_u8 << 2); // tell master that the slave is at IRQ2
    io::outb(PIC2_DATA, 2); // // tell slave that it's connected to IRQ2 on master
    // ICW4: tell PICs that they're in 8086 mode
    io::outb(PIC1_DATA, ICW4_8086);
    io::outb(PIC2_DATA, ICW4_8086);
}

/// Return the IRQ masks for PIC1 and PIC2.
pub unsafe fn get_masks() -> (u8, u8) {
    let mask1 = io::inb(PIC1_DATA);
    let mask2 = io::inb(PIC2_DATA);
    (mask1, mask2)
}

/// Set the IRQ masks for PIC1 and PIC2.
pub unsafe fn set_masks(pic1_mask: u8, pic2_mask: u8) {
    io::outb(PIC1_DATA, pic1_mask);
    io::outb(PIC2_DATA, pic2_mask);
}

/// Notify PICs that the interrupt was handled.
pub unsafe fn send_eoi(irq: u8) {
    if irq >= 8 {
        // IRQs >= 8 went through both PICs.
        io::outb(PIC2_CMD, 0x20);
    }
	io::outb(PIC1_CMD, 0x20);
}