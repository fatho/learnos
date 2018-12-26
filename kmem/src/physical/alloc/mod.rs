mod bump;

use crate::physical::{PageFrame};

pub use self::bump::BumpAllocator;

/// Generic interface for a page frame allocator.
pub trait PageFrameAllocator {
    fn alloc(&mut self) -> Option<PageFrame>;
    fn free(&mut self, frame: PageFrame);
}
