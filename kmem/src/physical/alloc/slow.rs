//! A slow but working page frame allocator.

use crate::physical::{PageFrame, PageFrameRegion};
use crate::physical::alloc::PageFrameAllocator;
use crate::physical::mgmt::{PageFrameTable, PageFrameState};

pub struct SlowPageFrameAllocator {
    page_frame_table: PageFrameTable,
}

impl SlowPageFrameAllocator {
    pub fn new(page_frames: PageFrameTable) -> Self {
        SlowPageFrameAllocator {
            page_frame_table: page_frames,
        }
    }

    pub fn page_frame_table(&self) -> &PageFrameTable {
        &self.page_frame_table
    }

    pub fn page_frame_table_mut(&mut self) -> &mut PageFrameTable {
        &mut self.page_frame_table
    }
}

impl PageFrameAllocator for SlowPageFrameAllocator {
    unsafe fn alloc(&mut self) -> Option<PageFrame> {
        // search first free page
        for frame in PageFrame(0) .. self.page_frame_table.upper_bound() {
            let entry = &mut self.page_frame_table[frame];
            if entry.state == PageFrameState::Free {
                entry.state = PageFrameState::Allocated;
                return Some(frame)
            }
        }
        return None
    }

    unsafe fn free(&mut self, frame: PageFrame) {
        let entry = &mut self.page_frame_table[frame];
        assert_eq!(entry.state, PageFrameState::Allocated);
        entry.state = PageFrameState::Free;
    }

    
    unsafe fn alloc_region(&mut self, page_count: usize) -> Option<PageFrameRegion> {
        // search first free region of that size
        let mut cur = PageFrame(0);
        let mut free_count = 0;
        while cur < self.page_frame_table.upper_bound() && free_count < page_count {
            if self.page_frame_table[cur].state == PageFrameState::Free {
                free_count += 1;
            } else {
                free_count = 0;
            }
            cur += 1;
        }
        if free_count == page_count {
            for i in cur - free_count .. cur {
                self.page_frame_table[i].state = PageFrameState::Allocated;
            }
            Some(PageFrameRegion {
                start: cur - page_count,
                end: cur
            })
        } else {
            None
        }
    }
    
    unsafe fn free_region(&mut self, region: PageFrameRegion) {
        for frame in region.start .. region.end {
            let entry = &mut self.page_frame_table[frame];
            assert_eq!(entry.state, PageFrameState::Allocated);
            entry.state = PageFrameState::Free;
        }
    }

}