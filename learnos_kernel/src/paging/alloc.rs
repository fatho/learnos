use crate::addr::{PhysAddr, VirtAddr};

/// A simple page frame allocator that uses the free pages themselves as a free list.
/// 
/// The whole range managed by the allocator must be mapped to a consecutive virtual
/// address range.
pub struct PageFrameAllocator {
    virtual_offset: u64,
    free_list_head: Option<PhysAddr>
}

impl PageFrameAllocator {
    pub unsafe fn new(phys_base: PhysAddr, _phys_limit: PhysAddr, virt_base: VirtAddr) -> Self {
        PageFrameAllocator {
            virtual_offset: virt_base.0 - phys_base.0,
            free_list_head: None,
        }
    }

    /// Allocates the given number of consecutive 4K pages.
    pub fn alloc(&mut self, count: u32) -> Option<PageFrame> {
        let mut candidate = self.free_list_head;
        let mut prev: Option<*mut FreeRegion> = None;

        while let Some(candidate_addr) = candidate {
            unsafe {
                let region = self.view_region(candidate_addr);
                if (*region).num_page_frames >= count {
                    // mark frame as used
                    let remaining_page_frames = (*region).num_page_frames - count;

                    let next = if remaining_page_frames == 0 {
                        // if we used the current region completely, link previous to next
                        (*region).next
                    } else {
                        // otherwise, make new region header of the remaining part of the
                        // current region and link it to the next region
                        let new_base = PhysAddr(candidate_addr.0 + 0x1000 * count as u64);
                        self.view_region(new_base).write(FreeRegion {
                            num_page_frames: remaining_page_frames,
                            next: (*region).next
                        });
                        Some(new_base)
                    };

                    // adjust previous element of the linked list or head pointer
                    if let Some(prev_region) = prev {
                        (*prev_region).next = next;
                    } else {
                        
                        self.free_list_head = next
                    }

                    // return the allocated page frame
                    let frame = PageFrame {
                        base_addr: candidate_addr,
                        num_page_frames: count,
                        reserved: 0
                    };
                    return Some(frame)
                } else {
                    // check next linked region
                    prev = Some(region);
                    candidate = (*region).next;
                }
            }
        }

        None
    }

    pub fn free(&mut self, frame: PageFrame) {
        unsafe { 
            // TODO: validate that the page frame comes from this allocator
            self.add_space(frame.base_addr, frame.num_page_frames)
        }
    }

    /// Add additional space for the page frame allocator to use.
    /// The base address must be aligned to 4K.
    pub unsafe fn add_space(&mut self, base_addr: PhysAddr, num_page_frames: u32) {
        assert!(base_addr.0 & 0xFFF == 0);
        let region = self.view_region(base_addr);
        (*region).num_page_frames = num_page_frames;
        (*region).next = self.free_list_head;
        self.free_list_head = Some(base_addr);
    }

    unsafe fn view_region(&self, base_addr: PhysAddr) -> *mut FreeRegion {
        (base_addr.0 + self.virtual_offset as u64) as *mut FreeRegion
    }
}

/// Reference to an allocated page frame.
#[repr(C, packed)]
pub struct PageFrame {
    base_addr: PhysAddr,
    num_page_frames: u32,
    reserved: u32
}

#[repr(C, packed)]
struct FreeRegion {
    /// Number of 4K pages in that region
    num_page_frames: u32,
    next: Option<PhysAddr>,
}