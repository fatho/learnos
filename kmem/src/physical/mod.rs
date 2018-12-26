use amd64::{Alignable, PhysAddr};
use core::ops;
use crate::{PAGE_SIZE, PAGE_ALIGN_BITS};

pub mod alloc;
pub mod mgmt;

/// Number of a physical page frame, counted from the start.
/// The first page frame at physical address 0x0 has number zero.
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct PageFrame(pub usize);

impl PageFrame {
    /// Return the next page frame starting at or above the given physical address.
    pub fn next_above(addr: PhysAddr) -> PageFrame {
        PageFrame(addr.align_up(PAGE_SIZE).0 >> PAGE_ALIGN_BITS)
    }

    /// Return the page frame including the given physical address.
    pub fn including(addr: PhysAddr) -> PageFrame {
        PageFrame(addr.align_down(PAGE_SIZE).0 >> PAGE_ALIGN_BITS)
    }

    pub fn start_address(&self) -> PhysAddr {
        PhysAddr(self.0 * PAGE_SIZE)
    }

    pub fn end_address(&self) -> PhysAddr {
        PhysAddr(self.0 * PAGE_SIZE + PAGE_SIZE)
    }
}

impl ops::Add<usize> for PageFrame {
    type Output = PageFrame;

    fn add(self, rhs: usize) -> PageFrame {
        PageFrame(self.0 + rhs)
    }
}

impl ops::AddAssign<usize> for PageFrame {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

/// A region of physical page frames.
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct PageFrameRegion {
    /// The first frame included in the region.
    pub start: PageFrame,
    /// The first frame after the region (not included).
    pub end: PageFrame,
}

impl PageFrameRegion {
    /// Construct the largest page frame region that is included in the given physical memory region.
    pub fn new_included_in(start: PhysAddr, end: PhysAddr) -> PageFrameRegion {
        PageFrameRegion {
            start: PageFrame(start.align_up(PAGE_SIZE).0 >> PAGE_ALIGN_BITS),
            end: PageFrame(end.align_down(PAGE_SIZE).0 >> PAGE_ALIGN_BITS)
        }
    }
    /// Construct the smallest page frame region that is fully including the given physical memory region.
    pub fn new_including(start: PhysAddr, end: PhysAddr) -> PageFrameRegion {
        let end_base = end.0 >> PAGE_ALIGN_BITS;
        let end_offset = if end.0 & (PAGE_SIZE - 1) != 0 { 1} else { 0 };
        PageFrameRegion {
            start: PageFrame(start.align_down(PAGE_SIZE).0 >> PAGE_ALIGN_BITS),
            end: PageFrame(end_base + end_offset)
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
    use amd64::{PhysAddr};
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

        let whole_mem = PageFrameRegion::new_including(PhysAddr(0), PhysAddr(0xFFFFFFFFFFFFFFFF));
        assert!(!whole_mem.is_empty());
        assert_eq!(whole_mem.length(), 0x0010_0000_0000_0000)
    }
}