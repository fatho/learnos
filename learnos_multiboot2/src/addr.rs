//! Newtype wrappers that make it harder to accidentally confuse physical and virtual addresses.

/// A virtual address. It's validity depends on the current page mapping.
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct VirtAddr(pub u64);

/// A physical address. Whether it is accessible depends on the current page mapping.
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct PhysAddr(pub u64);

/// A 32 bit physical address. Whether it is accessible depends on the current page mapping.
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct PhysAddr32(pub u32);

impl PhysAddr {
    /// Convert a physical address to a virtual address assuming identity mapping.
    pub fn identity_mapping(self) -> VirtAddr {
        VirtAddr(self.0)
    }
}

impl PhysAddr32 {
    /// Re-interpret a 32 bit address as 64 bit.
    pub fn extend(self) -> PhysAddr {
        PhysAddr(self.0 as u64)
    }
}