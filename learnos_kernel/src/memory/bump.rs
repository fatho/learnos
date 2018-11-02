use crate::multiboot2::memmap::{Regions, Region};

use super::{PageFrameAllocator, PageFrame, PageFrameNumber, PageFrameRegion};

/// A simple page frame allocator that bumps the frame number for each allocation.
/// It does not support freeing.
pub struct BumpAllocator {
    /// Next free page frame number. This does not mean that the page frame with that number is available.
    next_frame: PageFrameNumber,
    /// Pointer to the multiboot2 memory map
    regions: Regions,
}

impl BumpAllocator {
    pub unsafe fn new(initial_frame_number: PageFrameNumber, regions: Regions) -> Self {
        BumpAllocator {
            next_frame: initial_frame_number,
            regions: regions
        }
    }

    /// Reserve all page frames up to the given page frame number.
    /// Doesn't have any effect if the reserved page frames have already been allocated.
    pub fn reserve(&mut self, reserved_frame_number: PageFrameNumber) {
        self.next_frame = core::cmp::max(reserved_frame_number, self.next_frame);
    }

    /// The number of frames that can still be allocated.
    pub fn remaining_frames(&self) -> u64 {
        let next = self.next_frame;
        self.regions.clone()
            .filter(|r| r.is_available())
            .map(page_frames_in_region)
            .filter(|r| r.end > next)
            .map(|r| if next <= r.start { r.length() } else { r.length() - (next.0 - r.start.0) } )
            .sum()
    }

    /// The total number of available frames in memory.
    /// This is maximum number of page frames that could have been allocated.
    pub fn total_available_frames(&self) -> u64 {
        self.regions.clone()
            .filter(|r| r.is_available())
            .map(page_frames_in_region)
            .map(|r| r.length())
            .sum()
    }
}


impl PageFrameAllocator for BumpAllocator {
    fn alloc(&mut self) -> Option<PageFrame> {
        for region in self.regions.clone() {
            if region.is_available() {
                let region_frames = page_frames_in_region(region);

                if self.next_frame < region_frames.start {
                    self.next_frame = region_frames.start;
                }
                if self.next_frame < region_frames.end {
                    let allocated_frame = PageFrame(self.next_frame);
                    self.next_frame.0 += 1;
                    return Some(allocated_frame)
                }
            }
        }
        None
    }

    fn free(&mut self, _frame: PageFrame) {
        panic!("A bump allocator cannot free")
    }
}

fn page_frames_in_region(region: &Region) -> PageFrameRegion {
    PageFrameRegion::new_included_in(region.base_addr(), region.base_addr().add(region.length()))
}