use crate::physical::{PageFrame, PageFrameRegion};

mod slow;

pub use self::slow::SlowPageFrameAllocator;

/// Generic interface for a page frame allocator.
pub trait PageFrameAllocator {
    unsafe fn alloc(&mut self) -> Option<PageFrame>;
    /// Free a single page frame previously allocated via `alloc`.
    unsafe fn free(&mut self, frame: PageFrame);

    /// Allocate a consecutive region of physical page frames.
    unsafe fn alloc_region(&mut self, page_count: usize) -> Option<PageFrameRegion>;
    /// Free a consecutive region of physical page frames previously allocated via `alloc_region`.
    unsafe fn free_region(&mut self, region: PageFrameRegion);
}
