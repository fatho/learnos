///! Functionality for managing physical memory pages.

use crate::physical::{PageFrame, PageFrameRegion};

use core::mem;

use amd64::VirtAddr;

pub struct PageFrameTable {
    ptr: *mut PageFrameInfo,
    length: usize,
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
        let ptr: *mut PageFrameInfo = addr.as_mut_ptr();
        for i in 0..num_page_frames {
            ptr.add(i).write(PageFrameInfo {
                state: PageFrameState::Free,
            });
        }
        PageFrameTable {
            ptr: ptr,
            length: num_page_frames,
        }
    }

    /// Marks a whole region as reserved
    pub fn mark_allocated(&mut self, region: PageFrameRegion) {
        for entry in self.region_iter_mut(region) {
            assert!(entry.state == PageFrameState::Free, "cannot allocate non-free region");
            entry.state = PageFrameState::Allocated;
        }
    }

    /// Marks a whole region as reserved
    pub fn mark_reserved(&mut self, region: PageFrameRegion) {
        for entry in self.region_iter_mut(region) {
            assert!(entry.state == PageFrameState::Free, "cannot reserve non-free region");
            entry.state = PageFrameState::Reserved;
        }
    }

    fn index<'a>(&'a self, idx: usize) -> &'a PageFrameInfo {
        assert!(idx < self.length);
        unsafe { &*self.ptr.add(idx) }
    }

    fn index_mut<'a>(&'a mut self, idx: usize) -> &'a mut PageFrameInfo {
        assert!(idx < self.length);
        unsafe { &mut *self.ptr.add(idx) }
    }

    fn region_iter_mut<'a>(&'a mut self, region: PageFrameRegion) -> impl Iterator<Item=&'a mut PageFrameInfo> {
        assert!(region.start.0 < self.length && region.end.0 <= self.length);
        (region.start.0 .. region.end.0).into_iter().map(move |i| unsafe { &mut *self.ptr.add(i) } )
    }

    pub fn stats(&self) -> PageFrameStats {
        let (mut alloced, mut reserved) = (0, 0);
        for i in 0..self.length {
            match self.index(i).state {
                PageFrameState::Allocated => alloced += 1,
                PageFrameState::Reserved => reserved += 1,
                PageFrameState::Free => {},
            }
        }
        PageFrameStats {
            total_count: self.length,
            reserved_count: reserved,
            allocated_count: alloced,
        }
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
    state: PageFrameState
}