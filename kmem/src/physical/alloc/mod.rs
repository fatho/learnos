use crate::physical::{PageFrame, PageFrameRegion};

/// Generic interface for a page frame allocator.
pub trait PageFrameAllocator {
    /// Allocate a single page frame.
    fn alloc(&mut self) -> Option<PageFrame>;
    /// Free a single page frame previously allocated via `alloc`.
    fn free(&mut self, frame: PageFrame);

    /// Allocate a consecutive region of physical page frames.
    fn alloc_region(&mut self, page_count: usize) -> Option<PageFrameRegion>;
    /// Free a consecutive region of physical page frames previously allocated via `alloc_region`.
    fn free_region(&mut self, region: PageFrameRegion);
}
