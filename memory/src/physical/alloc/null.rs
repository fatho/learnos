//! An allocator that allocates nothing, mainly included for completeness.

use crate::physical::{PageFrame};

/// A null allocator that returns None on every allocation request and ignores frees.
pub struct NullAllocator;

/// A null allocator that panics on allocs and frees. Mainly useful for debugging purposes.
pub struct PanickingNullAllocator;


impl super::PageFrameAllocator for NullAllocator {
    fn alloc(&mut self) -> Option<PageFrame> {
        None
    }

    fn free(&mut self, _frame: PageFrame) { }
}

impl super::PageFrameAllocator for PanickingNullAllocator {
    fn alloc(&mut self) -> Option<PageFrame> {
        panic!("Cannot alloc with PanickingNullAllocator")
    }

    fn free(&mut self, _frame: PageFrame) { 
        panic!("He who cannot alloc must not free")
    }
}