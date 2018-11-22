use crate::util::Bits;

/// The identifier of an IOAPIC.
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct IoApicId(pub u8);

pub struct IoApicRegisters(*mut u32);

impl IoApicRegisters {
    pub const ID_REG: u32 = 0;
    pub const VER_REG: u32 = 1;
    pub const ARB_REG: u32 = 2;

    pub unsafe fn id(&self) -> IoApicId {
        IoApicId(self.read_reg(Self::ID_REG).get_bits(24..=27) as u8)
    }

    pub unsafe fn write_reg(&mut self, register_index: u32, value: u32) {
        self.address().write_volatile(register_index);
        self.data().write_volatile(value);
    }

    pub unsafe fn read_reg(&self, register_index: u32) -> u32 {
        self.address().write_volatile(register_index);
        self.data().read_volatile()
    }

    #[inline(always)]
    fn address(&self) -> *mut u32 {
        self.0
    }

    #[inline(always)]
    unsafe fn data(&self) -> *mut u32 {
        self.0.add(1)
    }
}