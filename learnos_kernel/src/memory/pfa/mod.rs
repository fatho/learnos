//! Page frame allocation.

pub mod bump;

use bare_metal::{PhysAddr};
use crate::spin::Mutex;
use multiboot2::memmap::Regions;

use super::{PageFrame};

/// Generic interface for a page frame allocator.
pub trait PageFrameAllocator {
    fn alloc(&mut self) -> Option<PageFrame>;
    fn free(&mut self, frame: PageFrame);
}

/// The global page frame allocator that can be used from anywhere in the kernel.
static GLOBAL_PAGE_FRAME_ALLOCATOR: Mutex<Option<bump::BumpAllocator>> = Mutex::new(None);

pub fn init(base: PhysAddr, regions: Regions) {
    let mut global_allocator = GLOBAL_PAGE_FRAME_ALLOCATOR.lock();
    if global_allocator.is_some() {
        panic!("Memory subsystem has already been initialized")
    }
    let mut new_allocator = bump::BumpAllocator::new(regions);
    new_allocator.reserve_until_address(base);
    *global_allocator = Some(new_allocator);
}

/// Allocate a physical page frame.
pub fn alloc_frame() -> Option<PageFrame> {
    GLOBAL_PAGE_FRAME_ALLOCATOR.lock().as_mut().expect("Memory subsystem not initialized").alloc()
}

/// Free a physical page frame.
pub fn free_frame(frame: PageFrame) {
    GLOBAL_PAGE_FRAME_ALLOCATOR.lock().as_mut().expect("Memory subsystem not initialized").free(frame)
}
