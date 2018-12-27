///! Functionality for managing physical memory pages.

use crate::physical::{PageFrame, PageFrameRegion};
use crate::KFixedVec;

use core::mem;
use core::ops::{Index, IndexMut};

use amd64::VirtAddr;

pub struct PageFrameTable {
    data: KFixedVec<PageFrameInfo>,
}

impl PageFrameTable {
    /// Required number of bytes for holding a page frame table for at most
    /// `num_page_frames` page frames.
    pub fn required_size_bytes(num_page_frames: usize) -> usize {
        num_page_frames * mem::size_of::<PageFrameInfo>()
    }

    /// Create a `PageFrameTable` at a given location and initialize all entries marking
    /// them as free.
    pub unsafe fn from_addr(addr: VirtAddr, num_page_frames: usize) -> PageFrameTable {
        let mut data = KFixedVec::from_raw_uninitialized(addr.as_mut_ptr(), num_page_frames);
        for _ in 0..num_page_frames {
            data.push(PageFrameInfo {
                state: PageFrameState::Free,
            });
        }
        PageFrameTable {
            data: data
        }
    }

    /// Marks a whole region as reserved
    pub fn mark_allocated(&mut self, region: PageFrameRegion) {
        for entry in self.region_iter_mut(region) {
            assert!(entry.state != PageFrameState::Reserved, "cannot allocate reserved region");
            entry.state = PageFrameState::Allocated;
        }
    }

    /// Marks a whole region as reserved
    pub fn mark_reserved(&mut self, region: PageFrameRegion) {
        for entry in self.region_iter_mut(region) {
            assert!(entry.state != PageFrameState::Allocated, "cannot reserve allocated region");
            entry.state = PageFrameState::Reserved;
        }
    }

    pub fn upper_bound(&self) -> PageFrame {
        PageFrame(self.data.len())
    }

    fn region_iter_mut(& mut self, region: PageFrameRegion) -> impl Iterator<Item=& mut PageFrameInfo> {
        self.data.as_slice_mut()[region.start.0 .. region.end.0].iter_mut()
    }

    pub fn stats(&self) -> PageFrameStats {
        let (mut alloced, mut reserved) = (0, 0);
        for frame in PageFrame(0)..self.upper_bound() {
            match self.index(frame).state {
                PageFrameState::Allocated => alloced += 1,
                PageFrameState::Reserved => reserved += 1,
                PageFrameState::Free => {},
            }
        }
        PageFrameStats {
            total_count: self.data.len(),
            reserved_count: reserved,
            allocated_count: alloced,
        }
    }
}

impl Index<PageFrame> for PageFrameTable {
    type Output = PageFrameInfo;

    fn index<'a>(&'a self, frame: PageFrame) -> &'a PageFrameInfo {
        &self.data[frame.0]
    }
}

impl IndexMut<PageFrame> for PageFrameTable {
    fn index_mut<'a>(&'a mut self, frame: PageFrame) -> &'a mut PageFrameInfo {
        &mut self.data[frame.0]
    }
}

/// Statistics about the page frame table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageFrameStats {
    pub total_count: usize,
    pub reserved_count: usize,
    pub allocated_count: usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageFrameState {
    Free = 0,
    Allocated = 1,
    Reserved = 2,
}

pub struct PageFrameInfo {
    pub state: PageFrameState
}