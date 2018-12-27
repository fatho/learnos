//! Information about the memory layout.

use amd64::{PhysAddr, PhysAddrRange};
use kmem::physical::{PageFrameRegion};

/// A region of physical memory with additional information.
pub struct PhysicalMemoryRegion {
    /// The contiguous range of page frames in that region.
    pub frames: PageFrameRegion,
    /// A flag indicating whether the region is available for use.
    pub available: bool,
}

/// The general memory layout that the memory subsystem needs to know about.
pub struct PhysicalMemoryLayout {
    /// The location of the kernel binary.
    /// May not be overwritten.
    pub kernel_memory: PhysAddrRange,
    /// The location of the multiboot info
    /// May be overwritten after the kernel has been initialized.
    pub multiboot_memory: PhysAddrRange,
    /// The location of the boot memory, consisting of the ASM bootcode
    /// and the initial page tables, stack and GDT.
    /// May be overwritten after the kernel has been initialized and all APs have been started.
    pub boot_memory: PhysAddrRange,
    /// Lowest address that is free for allocations.
    pub heap_start: PhysAddr,
}
