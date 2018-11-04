//! This module contains the memory management of the kernel.

pub mod pfa;
pub mod vmm;

use crate::addr::{PhysAddr};

/// Number of trailing zeros in a page aligned address.
pub const PAGE_ALIGN_BITS: u32 = 12;

pub const PAGE_SIZE: usize = 1 << PAGE_ALIGN_BITS;

/// Number of a physical page frame, counted from the start.
/// The first page frame at physicall address 0x0 has number zero.
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct PageFrameNumber(usize);

impl PageFrameNumber {
    /// Return the next page frame starting at or above the given physical address.
    pub fn next_above(addr: PhysAddr) -> PageFrameNumber {
        PageFrameNumber(addr.align_up(PAGE_SIZE).0 >> PAGE_ALIGN_BITS)
    }

    /// Return the page frame including the given physical address.
    pub fn including(addr: PhysAddr) -> PageFrameNumber {
        PageFrameNumber(addr.align_down(PAGE_SIZE).0 >> PAGE_ALIGN_BITS)
    }

    pub fn start_address(&self) -> PhysAddr {
        PhysAddr(self.0 * PAGE_SIZE)
    }

    pub fn end_address(&self) -> PhysAddr {
        PhysAddr(self.0 * PAGE_SIZE + PAGE_SIZE)
    }
}

/// Handle to an allocated page frame.
#[derive(Debug)]
pub struct PageFrame(PageFrameNumber);

impl PageFrame {
    pub fn start_address(&self) -> PhysAddr {
        self.0.start_address()
    }

    pub fn end_address(&self) -> PhysAddr {
        self.0.end_address()
    }
}

/// A region of physical page frames.
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct PageFrameRegion {
    /// The first frame included in the region.
    pub start: PageFrameNumber,
    /// The first frame after the region (not included).
    pub end: PageFrameNumber,
}

impl PageFrameRegion {
    /// Construct the largest page frame region that is included in the given physical memory region.
    pub fn new_included_in(start: PhysAddr, end: PhysAddr) -> PageFrameRegion {
        PageFrameRegion {
            start: PageFrameNumber(start.align_up(PAGE_SIZE).0 >> PAGE_ALIGN_BITS),
            end: PageFrameNumber(end.align_down(PAGE_SIZE).0 >> PAGE_ALIGN_BITS)
        }
    }
    /// Construct the smallest page frame region that is fully including the given physical memory region.
    pub fn new_including(start: PhysAddr, end: PhysAddr) -> PageFrameRegion {
        PageFrameRegion {
            start: PageFrameNumber(start.align_down(PAGE_SIZE).0 >> PAGE_ALIGN_BITS),
            end: PageFrameNumber(end.align_up(PAGE_SIZE).0 >> PAGE_ALIGN_BITS)
        }
    }

    pub fn length(&self) -> usize {
        if self.start > self.end {
            0
        } else {
            self.end.0 - self.start.0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

#[cfg(test)]
mod test {
    use crate::addr::{PhysAddr};
    use super::PageFrameRegion;

    #[test]
    fn test_page_frame_region() {
        let a = PhysAddr(0x400F);
        let b = PhysAddr(0x4EF0);
        let c = PhysAddr(0x7FFF);

        let in_ab = PageFrameRegion::new_included_in(a, b);
        assert!(in_ab.is_empty(), "in_ab = {:?}", in_ab);

        let in_ac = PageFrameRegion::new_included_in(a, c);
        assert!(!in_ac.is_empty(), "in_ac = {:?}", in_ac);
        assert!(in_ac.length() == 2, "in_ac = {:?}", in_ac);
        assert!(in_ac.start.0 == 5 && in_ac.end.0 == 7, "in_ac = {:?}", in_ac);

        let around_ab = PageFrameRegion::new_including(a, b);
        assert!(!around_ab.is_empty(), "around_ab = {:?}", around_ab);
        assert!(around_ab.length() == 1, "around_ab = {:?}", around_ab);
        assert!(around_ab.start.0 == 4 && around_ab.end.0 == 5, "around_ab = {:?}", around_ab);

        let around_ac = PageFrameRegion::new_including(a, c);
        assert!(!around_ac.is_empty(), "around_ac = {:?}", around_ac);
        assert!(around_ac.length() == 4, "around_ac = {:?}", around_ac);
        assert!(around_ac.start.0 == 4 && around_ac.end.0 == 8, "around_ac = {:?}", around_ac);

        // TODO: Cannot construct page frame region for highest frame.
        // Only relevant on systems with 256 TiB memory.

        // let whole_mem = PageFrameRegion::new_including(PhysAddr(0), PhysAddr(0xFFFFFFFFFFFFFFFF));
        // assert!(!whole_mem.is_empty());
        // assert!(whole_mem.length() == 0x0008_0000_0000_0000)
    }
}