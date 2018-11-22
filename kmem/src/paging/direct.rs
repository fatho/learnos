use amd64::{PhysAddr, VirtAddr};

/// Implements translation of physical to virtual addresses for a direct mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectMapping {
    virtual_base: VirtAddr,
    physical_base: PhysAddr,
    size_in_bytes: usize,
}

impl DirectMapping {
    pub const fn new(virtual_base: VirtAddr, physical_base: PhysAddr, size_in_bytes: usize) -> Self {
        DirectMapping {
            virtual_base: virtual_base,
            physical_base: physical_base,
            size_in_bytes: size_in_bytes
        }
    }

    /// The start of the virtual address range of this mapping, mapped to `self.physical_base()`.
    pub fn virtual_base(&self) -> VirtAddr {
        self.virtual_base
    }

    /// The start of the physical address range of this mapping.
    pub fn physical_base(&self) -> PhysAddr {
        self.physical_base
    }

    /// The size of the mapped range in bytes.
    pub fn size_in_bytes(&self) -> usize {
        self.size_in_bytes
    }

    /// Returns whether the given physical address is part of this mapping.
    pub fn contains_phys(&self, phys_addr: PhysAddr) -> bool {
        phys_addr >= self.physical_base && phys_addr < self.physical_base + self.size_in_bytes
    }

    /// Returns whether the given virtual address is part of this mapping.
    pub fn contains_virt(&self, virt_addr: VirtAddr) -> bool {
        virt_addr >= self.virtual_base && virt_addr < self.virtual_base + self.size_in_bytes
    }

    /// Translates a physical to a virtual address using the direct mapping.
    /// 
    /// # Panics
    /// 
    /// 1. Panics, if the given physical address is outside of the range provided by this direct mapping.
    pub fn phys_to_virt(&self, phys_addr: PhysAddr) -> VirtAddr {
        if ! self.contains_phys(phys_addr) {
            panic!("[DirectMapping::phys_to_virt] physical address {:p} out of bounds", phys_addr);
        }
        VirtAddr(phys_addr.0 - self.physical_base.0 + self.virtual_base.0)
    }

    /// Translates a virtual to a physical address using the direct mapping.
    /// 
    /// # Panics
    /// 
    /// 1. Panics, if the given physical address is outside of the range provided by this direct mapping.
    pub fn virt_to_phys(&self, virt_addr: VirtAddr) -> PhysAddr {
        if ! self.contains_virt(virt_addr) {
            panic!("[DirectMapping::virt_to_phys] virtual address {:p} out of bounds", virt_addr);
        }
        PhysAddr(virt_addr.0 - self.virtual_base.0 + self.physical_base.0)
    }
}


#[cfg(test)]
mod tests {
    use amd64::{PhysAddr, VirtAddr};
    use super::*;

    #[test]
    fn direct_mapping_roundtrips() {
        let low_phys = PhysAddr(0x0000000000001000);
        let low_virt = VirtAddr(0xFFFFFFFF00000000);
        let size = 4096 * 10000;
        let high_phys = low_phys + size;
        let dm = DirectMapping::new(low_virt, low_phys, size);

        // test something in between
        let test_phys = PhysAddr(0x4000);
        assert!(dm.contains_phys(test_phys));
        assert_eq!(dm.virt_to_phys(dm.phys_to_virt(test_phys)), test_phys);

        // test edge cases
        assert!(dm.contains_phys(low_phys));
        assert_eq!(dm.virt_to_phys(dm.phys_to_virt(low_phys)), low_phys);
        assert!(dm.contains_phys(high_phys - 1));
        assert_eq!(dm.virt_to_phys(dm.phys_to_virt(high_phys - 1)), high_phys - 1);

        // just outside
        assert!(!dm.contains_phys(low_phys - 1));
        assert!(!dm.contains_phys(high_phys));
    }
}
