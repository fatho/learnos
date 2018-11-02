pub mod bump;

use crate::addr::{PhysAddr};

pub const PAGE_SIZE: u64 = 0x64;

pub struct PageFrame {
    number: u64,
}

impl PageFrame {
    pub fn start_address(&self) -> PhysAddr {
        PhysAddr(self.number * PAGE_SIZE)
    }

    pub fn end_address(&self) -> PhysAddr {
        PhysAddr(self.number * PAGE_SIZE + PAGE_SIZE)
    }
}

trait PageFrameAllocator {
    fn alloc(&mut self) -> Option<PageFrame>;
    fn free(&mut self, frame: PageFrame);
}
