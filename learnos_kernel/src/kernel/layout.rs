//! Hardcoded memory layout of the kernel.

use crate::addr::{PhysAddr, VirtAddr};

/// The virtual address where the kernel reserved area begins (highest 2 GiB)
pub const KERNEL_VIRTUAL_BASE: VirtAddr = VirtAddr(0xFFFFFFFF80000000);

/// The highest physical address that's mapped in the kernel area.
pub const LOW_PHYS_MAX: PhysAddr = PhysAddr(0x0000000080000000);

/// Map a physical address inside the lowest 2 GiB to its corresponding virtual
/// address in the highest two 2 GiB.
pub fn low_phys_to_virt(phys: PhysAddr) -> VirtAddr {
    assert!(phys < LOW_PHYS_MAX);
    VirtAddr(phys.0 + KERNEL_VIRTUAL_BASE.0)
}