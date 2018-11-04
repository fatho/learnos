use core::mem;
use core::ops;

/// Load an IDT for the current CPU.
pub unsafe fn load_idt(idt: &Idt) {
    let idtr = Idtr {
        limit: core::mem::size_of::<Idt>() as u16 - 1,
        offset: idt as *const Idt as u64,
    };
    asm!("lidt [$0]" : : "r"(&idtr) : : "intel", "volatile")
}

/// IDT Register value
#[repr(C,packed)]
pub struct Idtr {
    limit: u16,
    offset: u64,
}

/// Interrupt descriptor table
#[repr(C,packed)]
pub struct Idt {
    entries: [IdtEntry; 256]
}
assert_eq_size!(idt_size; Idt, [u64; 512]);

impl Idt {
    pub const fn new() -> Idt {
        Idt {
            entries: [IdtEntry::new(); 256]
        }
    }
}

impl ops::Index<u8> for Idt {
    type Output = IdtEntry;

    fn index(&self, idx: u8) -> &IdtEntry {
        &self.entries[idx as usize]
    }
}

impl ops::IndexMut<u8> for Idt {
    fn index_mut(&mut self, idx: u8) -> &mut IdtEntry {
        &mut self.entries[idx as usize]
    }
}

/// An entry in the interrupt descriptor table.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C,packed)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    reserved_ist: u8,
    /// `[P:1][DPL:2][MBZ:1][Type:4]`
    type_attr: u8,
    offset_middle: u16,
    offset_high: u32,
    reserved: u32,
}
assert_eq_size!(idt_entry_size; IdtEntry, [u64; 2]);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GateType(u8);

impl GateType {
    pub const CALL_GATE: GateType = GateType(0x0C);
    pub const INTERRUPT_GATE: GateType = GateType(0x0E);
    pub const TRAP_GATE: GateType = GateType(0x0F);
}

impl IdtEntry {
    const DPL_MASK: u8 = 0b0110_0000;
    const TYPE_MASK: u8 = 0b0000_1111;
    const PRESENT_MASK: u8 = 0b1000_0000;

    /// Create a new empty IDT entry.
    pub const fn new() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            reserved_ist: 0,
            type_attr: 0,
            offset_middle: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    /// Return whether this entry is present.
    pub fn present(&self) -> bool {
        self.type_attr & Self::PRESENT_MASK != 0
    }

    /// Set the present bit of the entry.
    pub fn set_present(&mut self, value: bool) {
        if value {
            self.type_attr |= Self::PRESENT_MASK;
        } else {
            self.type_attr &= !Self::PRESENT_MASK;
        }
    }

    /// Return the assigned handler, if it is not null.
    pub unsafe fn handler(&self) -> Option<unsafe extern "C" fn() -> ()> {
        let addr = (self.offset_low as usize) |
                ((self.offset_middle as usize) << 16) |
                ((self.offset_high as usize) << 32);

        if addr != 0 {
            Some(mem::transmute(addr))
        } else {
            None
        }
    }

    /// Set the handler.
    pub unsafe fn set_handler(&mut self, handler: Option<unsafe extern "C" fn() -> ()>) {
        let addr: usize = mem::transmute(handler);
        self.offset_low = (addr & 0xFFFF) as u16;
        self.offset_middle = ((addr >> 16) & 0xFFFF) as u16;
        self.offset_high = ((addr >> 32) & 0xFFFF_FFFF) as u32;
    }

    /// Offset in the GDT that determines the segment used for this gate.
    pub fn selector(&self) -> u16 {
        self.selector
    }

    pub fn set_selector(&mut self, selector: u16) {
        self.selector = selector
    }

    pub fn gate_type(&self) -> GateType {
        GateType(self.type_attr & Self::TYPE_MASK)
    }

    pub fn set_gate_type(&mut self, gate_type: GateType) {
        self.type_attr = (self.type_attr & !Self::TYPE_MASK) | gate_type.0
    }

    /// The privilege level required to call this gate.
    pub fn descriptor_privilege(&self) -> u8 {
        (self.type_attr & Self::DPL_MASK) >> 5
    }

    pub fn set_descriptor_privilege(&mut self, descriptor_privilege: u8) {
        self.type_attr = (self.type_attr & !Self::DPL_MASK) | ((descriptor_privilege << 5) & Self::DPL_MASK)
    }
    // TODO: IST field
}