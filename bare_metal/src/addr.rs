//! Newtype wrappers that make it harder to accidentally confuse physical and virtual addresses.

use core::fmt;
use core::ops;

use super::align::Alignable;

/// A virtual address. It's validity depends on the current page mapping.
#[repr(C)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct VirtAddr(pub usize);

/// A physical address. Whether it is accessible depends on the current page mapping.
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
#[repr(C)]
pub struct PhysAddr(pub usize);

impl VirtAddr {
    pub unsafe fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    pub unsafe fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }
}

macro_rules! impl_addr_arith {
    ($addr:tt) => {
        impl Alignable for $addr {
            type Alignment = usize;

            fn align_up(self, alignment: usize) -> Self {
                $addr(self.0.align_up(alignment))
            }

            fn align_down(self, alignment: usize) -> Self {
                $addr(self.0.align_down(alignment))
            }
        }

        impl ops::Add<usize> for $addr {
            type Output = $addr;

            fn add(self, other: usize) -> Self::Output {
                $addr(self.0 + other)
            }
        }

        impl ops::AddAssign<usize> for $addr {
            fn add_assign(&mut self, other: usize) {
                self.0 += other;
            }
        }
    };
}

impl_addr_arith!(VirtAddr);
impl_addr_arith!(PhysAddr);

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
