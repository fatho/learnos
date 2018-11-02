use crate::addr::{PhysAddr, VirtAddr};


use super::{PageFrameAllocator, PageFrame};

/// A simple page frame allocator that uses the free pages themselves as a free list.
/// 
/// The whole range managed by the allocator must be mapped to a consecutive virtual
/// address range.
pub struct BumpAllocator {
}

impl BumpAllocator {
    pub unsafe fn new(phys_base: PhysAddr, _phys_limit: PhysAddr, virt_base: VirtAddr) -> Self {
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