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

    pub fn align_up(self, alignment: usize) -> Self {
        VirtAddr(align_up(self.0, alignment))
    }

    pub fn align_down(self, alignment: usize) -> Self {
        VirtAddr(align_down(self.0, alignment))
    }

    pub unsafe fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }
}

impl PhysAddr {
    pub fn add(self, offset: usize) -> Self {
        PhysAddr(self.0 + offset)
    }

    pub fn align_up(self, alignment: usize) -> Self {
        PhysAddr(align_up(self.0, alignment))
    }

    pub fn align_down(self, alignment: usize) -> Self {
        PhysAddr(align_down(self.0, alignment))
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

/// Return the smallest `x` that is a multiple of `alignment` such that `x >= num`.
#[inline]
pub fn align_up(num: usize, alignment: usize) -> usize {
    if alignment == 0 {
        num
    } else {
        let mask = alignment - 1;
        assert!(alignment & mask == 0, "alignment must be power of two");
        let padding = alignment - (num & mask);
        num + (padding & mask)
    }
}

/// Return the largest `x` that is a multiple of `alignment` such that `x <= num`.
pub fn align_down(num: usize, alignment: usize) -> usize {
    if alignment == 0 {
        num
    } else {
        let mask = alignment - 1;
        assert!(alignment & mask == 0, "alignment must be power of two");
        let padding = num & mask;
        num - padding
    }
}

#[cfg(test)]
mod test {
    use super::{align_down, align_up};

    #[test]
    fn align_down_test() {
        assert_eq!(align_down(23, 8), 16);
        assert_eq!(align_down(24, 8), 24);
        assert_eq!(align_down(25, 8), 24);

        // edge cases
        assert_eq!(align_down(23, 0), 23);
        assert_eq!(align_down(0, 0), 0);
        assert_eq!(align_down(0xFFFF_FFFF_FFFF_FFFF, 0), 0xFFFF_FFFF_FFFF_FFFF);
    }

    #[test]
    fn align_up_test() {
        assert_eq!(align_up(23, 8), 24);
        assert_eq!(align_up(24, 8), 24);
        assert_eq!(align_up(25, 8), 32);

        // edge cases
        assert_eq!(align_up(23, 0), 23);
        assert_eq!(align_up(0, 0), 0);
        assert_eq!(align_up(0xFFFF_FFFF_FFFF_FFFF, 0), 0xFFFF_FFFF_FFFF_FFFF);
    }
}