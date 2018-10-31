//! Newtype wrappers that make it harder to accidentally confuse physical and virtual addresses.

/// A virtual address. It's validity depends on the current page mapping.
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct VirtAddr(pub u64);

/// A physical address. Whether it is accessible depends on the current page mapping.
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct PhysAddr(pub u64);


impl PhysAddr {
    /// Convert a physical address to a virtual address assuming identity mapping.
    pub fn identity_mapping(self) -> VirtAddr {
        VirtAddr(self.0)
    }
}