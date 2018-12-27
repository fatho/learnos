use amd64::{PhysAddr, PhysAddrRange, VirtAddr};

use kmem::physical::alloc::{PageFrameAllocator, SlowPageFrameAllocator};
use kmem::physical::mgmt::{PageFrameTable, PageFrameState};
use kmem::physical::{PageFrame, PageFrameRegion};
use kmem::paging::direct::DirectMapping;

mod init;
mod layout;

pub use self::layout::{PhysicalMemoryLayout, PhysicalMemoryRegion};

pub struct MemorySubsystem {
    /// The memory subsystem needs a contiguous view of all of the physical memory.
    physical_mapping: DirectMapping,
    /// Page frame allocation.
    page_frame_allocator: spin::Mutex<SlowPageFrameAllocator>,
}

impl MemorySubsystem {
    pub unsafe fn new<I>(
        physical_mapping: DirectMapping, physical_layout: &PhysicalMemoryLayout, memory_map: I
    ) -> MemorySubsystem where
        I: Clone + Iterator<Item=PhysicalMemoryRegion>
    {
        let pf_table = init::initialize_page_frame_table(physical_layout, memory_map, &physical_mapping);
        MemorySubsystem {
            physical_mapping: physical_mapping,
            page_frame_allocator: spin::Mutex::new(SlowPageFrameAllocator::new(pf_table))
        }
    }

    /// Returns the virtual address for accessing a physical memory location.
    pub fn physical(&self, addr: PhysAddr) -> VirtAddr {
        self.physical_mapping.phys_to_virt(addr)
    }
}

