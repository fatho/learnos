use crate::addr::{PhysAddr, VirtAddr};


use super::{PageFrameAllocator, PageFrame};

/// A simple page frame allocator that bumps the frame number for each allocation.
pub struct BumpAllocator {
    next_frame: u64
}

impl BumpAllocator {
    pub unsafe fn new() -> Self {
        unimplemented!()
    }
}


impl PageFrameAllocator for BumpAllocator {
    fn alloc(&mut self) -> Option<PageFrame> {
        unimplemented!()
    }

    fn free(&mut self, frame: PageFrame) {
        unimplemented!()
    }
}