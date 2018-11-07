//! A bump memory allocator that is meant for the early system startup.
//! Memory allocated with it can not be freed, so it should only be used
//! for data that has to live for the whole up time, or that can be reclaimed
//! via different means later.

use core::cmp;
use bare_metal::PhysAddr;

use crate::physical::{PageFrame, PageFrameRegion};
use super::{PageFrameAllocator};

/// A simple page frame allocator that bumps the frame number for each allocation.
/// It does not support freeing. It is parameterized over an iterator yielding
/// memory regions available for allocation.
pub struct BumpAllocator<R> {
    /// Current memory region that is up for allocation. Allocations start at the bottom.
    current_region: Option<PageFrameRegion>,
    /// Pointer to the multiboot2 memory map
    regions: R,
}

impl<R> BumpAllocator<R> where
    R: Iterator<Item=PageFrameRegion>
{
    pub fn new(mut regions: R) -> Self {
        let first_region = regions.next();
        BumpAllocator {
            current_region: first_region,
            regions: regions
        }
    }

    /// Reserve all page frames up to the given page frame number.
    /// Doesn't have any effect if the reserved page frames have already been allocated.
    pub fn reserve_until(&mut self, reserved_frame_number: PageFrame) {
        self.current_region = self.current_region
            .iter().cloned()
            .chain(&mut self.regions)
            .find_map(|mut r| {
                if r.end > reserved_frame_number {
                    r.start = cmp::max(r.start, reserved_frame_number);
                    Some(r)
                } else {
                    None
                }
            });
    }

    /// Reserve all page frames up to the given physical address page frame number.
    /// Doesn't have any effect if the reserved page frames have already been allocated.
    pub fn reserve_until_address(&mut self, reserved_address: PhysAddr) {
        self.reserve_until(PageFrame::next_above(reserved_address));
    }
}


impl<R> PageFrameAllocator for BumpAllocator<R> where
    R: Iterator<Item=PageFrameRegion>
{
    fn alloc(&mut self) -> Option<PageFrame> {
        // find first region that is not empty, including the current one
        self.current_region = self.current_region
            .iter().cloned()
            .chain(&mut self.regions)
            .find(|r| !r.is_empty());

        match self.current_region {
            None => None,
            Some(ref mut region) => {
                let pf = region.start;
                region.start += 1;
                Some(pf)
            }
        }
    }

    fn free(&mut self, _frame: PageFrame) {
        panic!("A bump allocator cannot free")
    }
}
