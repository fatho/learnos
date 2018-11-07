use bare_metal::PhysAddr;

/// An entry in a page table.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct PageTableEntry(u64);

bitflags! {
    pub struct Flags : u8 {
        const PRESENT  = 0b00000001;
        const WRITABLE = 0b00000010;
        const USER     = 0b00000100;
        const PWT      = 0b00001000;
        const PCD      = 0b00010000;
        const ACCESSED = 0b00100000;
        const DIRTY    = 0b01000000;
        // in a PTE, this is the PAT (page attribute table) bit
        // Must be zero in PML4.
        const SIZE     = 0b10000000;
    }
}

impl PageTableEntry {
    const NO_EXECUTE_BIT: u32 = 63;
    // mask for valid physical base addresses
    const ADDR_MASK: usize = 0x000F_FFFF_FFFF_F000;
    const USER_DATA_HIGH_MASK: u64 = 0x7FF8_0000_0000_0000;
    const USER_DATA_LOW_MASK: u64  = 0x0000_0000_0000_0E00;

    pub const fn new() -> Self {
        PageTableEntry(0)
    }

    pub fn flags(&self) -> Flags {
        // unwrapping cannot fail, all combinations are valid
        Flags::from_bits((self.0 & 0xFF) as u8).unwrap()
    }

    pub fn set_flags(&mut self, flags: Flags) {
        self.0 = (self.0 & !0xFF_u64) | flags.bits() as u64;
    }

    /// Return whether the page is executable.
    pub fn executable(&self) -> bool {
        ! self.check_bit(Self::NO_EXECUTE_BIT)
    }

    /// Sets whether the page is executable or not. NOTE: marking a page as non-executable
    /// will cause a page fault when no-execute support has not been enabled first.
    pub fn set_executable(&mut self, executable: bool) {
        self.set_bit(Self::NO_EXECUTE_BIT, ! executable)
    }

    /// Return the physical page address of the page or page table pointed to by this entry.
    pub fn base(&self) -> PhysAddr {
        PhysAddr((self.0 as usize) & Self::ADDR_MASK)
    }

    /// Set the physical base address in this entry.
    /// The address is aligned downwards if necessary.
    pub fn set_base(&mut self, addr: PhysAddr) {
        self.0 = ((self.0 as usize & !Self::ADDR_MASK) | (addr.0 & Self::ADDR_MASK)) as u64;
    }

    /// Return the 14 bits of user data stored in a page table entry.
    /// They are composed from bits `[62..52][11..9]` both ends of the range inclusive.
    pub fn user_data(&self) -> u16 {
        let high = ((self.0 & Self::USER_DATA_HIGH_MASK) >> (52 - 3)) as u16;
        let low = ((self.0 & Self::USER_DATA_LOW_MASK) >> 9) as u16;
        high | low
    }

    /// Set the 14 bits of user data stored in a page table entry.
    pub fn set_user_data(&mut self, user_data: u16) {
        self.0 = (self.0 & !Self::USER_DATA_HIGH_MASK & !Self::USER_DATA_LOW_MASK)
               | (((user_data as u64) << (52 - 3)) & Self::USER_DATA_HIGH_MASK)
               | (((user_data as u64) << 9) & Self::USER_DATA_LOW_MASK);
    }

    fn check_bit(&self, bit: u32) -> bool {
        self.0 & (1 << bit) != 0
    }

    fn set_bit(&mut self, bit: u32, value: bool) {
        let mask = 1_u64 << bit;
        if value {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn page_table_entry_accessors() {
        let mut pte = PageTableEntry::new();

        let user_data = 0x3A75; // 14 bits available
        let flags = Flags::PRESENT | Flags::SIZE | Flags::USER;
        let addr = PhysAddr(0x0008_0F7A_BA02_1000);

        pte.set_user_data(user_data);
        pte.set_flags(flags);
        pte.set_base(addr);

        assert_eq!(pte.user_data(), user_data, "user data roundtrip failed");
        assert_eq!(pte.flags(), flags, "flag roundtrip failed");
        assert_eq!(pte.base(), addr, "addr roundtrip failed");

        // set fields in a different order now
        pte.set_flags(flags);
        pte.set_user_data(user_data);

        assert_eq!(pte.user_data(), user_data, "user data roundtrip failed");
        assert_eq!(pte.flags(), flags, "flag roundtrip failed");
        assert_eq!(pte.base(), addr, "addr roundtrip failed");
    }
}