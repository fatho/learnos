use core::mem;
use core::ops;

use crate::segments::{Ring, Selector};

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

// sanity check that everything adds up in terms of size
assert_eq_size!(idt_size; Idt, [u64; 512]);

impl Idt {
    pub const fn new() -> Idt {
        Idt {
            entries: [IdtEntry::empty(); 256]
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

// sanity check that everything adds up in terms of size
assert_eq_size!(idt_entry_size; IdtEntry, [u64; 2]);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GateType(u8);

pub type Handler = unsafe extern "C" fn() -> !;

impl GateType {
    pub const CALL_GATE: GateType = GateType(0x0C);
    pub const INTERRUPT_GATE: GateType = GateType(0x0E);
    pub const TRAP_GATE: GateType = GateType(0x0F);
}

impl IdtEntry {
    const DPL_MASK: u8 = 0b0110_0000;
    const TYPE_MASK: u8 = 0b0000_1111;
    const PRESENT_MASK: u8 = 0b1000_0000;

    pub unsafe fn new(gate_type: GateType, selector: Selector, handler: Option<Handler>, dpl: Ring, present: bool) -> IdtEntry {
        let mut e = Self::empty();
        e.set_gate_type(gate_type);
        e.set_selector(selector);
        e.set_handler(handler);
        e.set_descriptor_privilege(dpl);
        e.set_present(present);
        e
    }

    /// Create a new non-present, DPL 0 and empty IDT entry with the invalid selector 0.
    pub const fn empty() -> IdtEntry {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            reserved_ist: 0,
            type_attr: GateType::INTERRUPT_GATE.0,
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
    pub unsafe fn handler(&self) -> Option<Handler> {
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
    pub unsafe fn set_handler(&mut self, handler: Option<Handler>) {
        let addr: usize = mem::transmute(handler);
        self.offset_low = (addr & 0xFFFF) as u16;
        self.offset_middle = ((addr >> 16) & 0xFFFF) as u16;
        self.offset_high = ((addr >> 32) & 0xFFFF_FFFF) as u32;
    }

    /// Offset in the GDT that determines the segment used for this gate.
    pub fn selector(&self) -> Selector {
        Selector(self.selector)
    }

    pub fn set_selector(&mut self, selector: Selector) {
        self.selector = selector.0
    }

    pub fn gate_type(&self) -> GateType {
        GateType(self.type_attr & Self::TYPE_MASK)
    }

    pub fn set_gate_type(&mut self, gate_type: GateType) {
        self.type_attr = (self.type_attr & !Self::TYPE_MASK) | gate_type.0
    }

    /// The privilege level required to call this gate.
    pub fn descriptor_privilege(&self) -> Ring {
        // we can safely unwrap because we only ever store a valid ring
        Ring::new((self.type_attr & Self::DPL_MASK) >> 5).unwrap()
    }

    pub fn set_descriptor_privilege(&mut self, descriptor_privilege: Ring) {
        self.type_attr = (self.type_attr & !Self::DPL_MASK) | ((descriptor_privilege.number() << 5) & Self::DPL_MASK)
    }
    // TODO: IST field
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn idt_entry_accessors_roundtrip() {
        let mut e = IdtEntry::empty();
        unsafe {
            e.set_handler(Some(test_handler));
            assert_eq!(e.handler(), Some(test_handler as Handler));
            e.set_handler(None);
            assert_eq!(e.handler(), None);
        }

        e.set_descriptor_privilege(Ring::RING3);
        e.set_gate_type(GateType::CALL_GATE);
        e.set_selector(Selector(0xDEAD));
        assert_eq!(e.descriptor_privilege(), Ring::RING3);
        assert_eq!(e.gate_type(), GateType::CALL_GATE);
        assert_eq!(e.selector(), Selector(0xDEAD));
    }

    #[test]
    fn empty_idt_entry_correctly_set_up() {
        let e = IdtEntry::empty();
        unsafe {
            assert_eq!(e.handler(), None);
        }
        assert_eq!(e.descriptor_privilege(), Ring::RING0);
        assert_eq!(e.gate_type(), GateType::INTERRUPT_GATE);
        assert_eq!(e.selector(), Selector(0));
    }

    #[test]
    fn new_idt_entry_correctly_set_up() {
        unsafe {
            let e = IdtEntry::new(GateType::TRAP_GATE, Selector(32), Some(test_handler), Ring::RING2, true);
            assert_eq!(e.handler(), Some(test_handler as Handler));
            assert_eq!(e.selector(), Selector(32));
            assert_eq!(e.descriptor_privilege(), Ring::RING2);
            assert_eq!(e.gate_type(), GateType::TRAP_GATE);
            assert!(e.present());
        }
    }

    unsafe extern "C" fn test_handler() -> ! {
        panic!("You should not have come!")
    }
}