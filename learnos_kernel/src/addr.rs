//! Newtype wrappers that make it harder to accidentally confuse physical and virtual addresses.

use core::fmt;

/// A virtual address. It's validity depends on the current page mapping.
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct VirtAddr(pub u64);

/// A physical address. Whether it is accessible depends on the current page mapping.
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct PhysAddr(pub u64);

/// A 32 bit physical address. Whether it is accessible depends on the current page mapping.
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct PhysAddr32(pub u32);

impl VirtAddr {
    pub fn add(self, offset: u64) -> Self {
        VirtAddr(self.0 + offset)
    }

    pub fn align_up(self, zero_bits: u32) -> Self {
        VirtAddr(align_up(self.0, zero_bits))
    }

    pub fn align_down(self, zero_bits: u32) -> Self {
        VirtAddr(align_down(self.0, zero_bits))
    }
}

impl PhysAddr {
    pub fn add(self, offset: u64) -> Self {
        PhysAddr(self.0 + offset)
    }

    pub fn align_up(self, zero_bits: u32) -> Self {
        PhysAddr(align_up(self.0, zero_bits))
    }

    pub fn align_down(self, zero_bits: u32) -> Self {
        PhysAddr(align_down(self.0, zero_bits))
    }
}

impl PhysAddr32 {
    /// Re-interpret a 32 bit address as 64 bit.
    pub fn extend(self) -> PhysAddr {
        PhysAddr(self.0 as u64)
    }
}

impl fmt::Pointer for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PHYS_0x{:016x}", self.0)
    }
}

impl fmt::Pointer for PhysAddr32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PHYS_0x{:08x}", self.0)
    }
}

impl fmt::Pointer for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VIRT_0x{:016x}", self.0)
    }
}

fn align_up(num: u64, zero_bits: u32) -> u64 {
    let multiple = 1 << zero_bits;
    let mask = multiple - 1;
    let padding = multiple - (num & mask);
    num + padding
}

fn align_down(num: u64, zero_bits: u32) -> u64 {
    let multiple = 1 << zero_bits;
    let mask = multiple - 1;
    let padding = num & mask;
    num - padding
}