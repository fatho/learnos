//! Newtype wrappers that make it harder to accidentally confuse physical and virtual addresses.

use core::fmt;

// For now, this kernel is 64 bit only. Ensure that `usize` has the right size.
assert_eq_size!(ptr_size; usize, u64);

/// A virtual address. It's validity depends on the current page mapping.
#[repr(C)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct VirtAddr(pub usize);

/// A physical address. Whether it is accessible depends on the current page mapping.
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
#[repr(C)]
pub struct PhysAddr(pub usize);

impl VirtAddr {
    pub fn add(self, offset: usize) -> Self {
        VirtAddr(self.0 + offset)
    }

    pub fn align_up(self, zero_bits: u32) -> Self {
        VirtAddr(align_up(self.0, zero_bits))
    }

    pub fn align_down(self, zero_bits: u32) -> Self {
        VirtAddr(align_down(self.0, zero_bits))
    }

    pub unsafe fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }
}

impl PhysAddr {
    pub fn add(self, offset: usize) -> Self {
        PhysAddr(self.0 + offset)
    }

    pub fn align_up(self, zero_bits: u32) -> Self {
        PhysAddr(align_up(self.0, zero_bits))
    }

    pub fn align_down(self, zero_bits: u32) -> Self {
        PhysAddr(align_down(self.0, zero_bits))
    }
}

impl fmt::Pointer for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PHYS_0x{:016x}", self.0)
    }
}

impl fmt::Pointer for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VIRT_0x{:016x}", self.0)
    }
}

pub fn align_up(num: usize, zero_bits: u32) -> usize {
    let multiple = 1 << zero_bits;
    let mask = multiple - 1;
    let padding = multiple - (num & mask);
    num + padding
}

pub fn align_down(num: usize, zero_bits: u32) -> usize {
    let multiple = 1 << zero_bits;
    let mask = multiple - 1;
    let padding = num & mask;
    num - padding
}