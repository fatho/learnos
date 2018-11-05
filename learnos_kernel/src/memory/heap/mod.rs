//! Provides a kernel space heap allocator.
//! The allocator must be instantiated in the root module as a static variable and marked with the #[global_allocator] attribute.

use crate::addr::VirtAddr;
use crate::spin::Mutex;
use ::alloc::alloc;

/// A bump allocator based heap. It can only allocate, but not free.
pub struct BumpHeap {
    /// Address up to which memory is allocated. The range between `allocated` (inclusive) up to `limit` (exclusive) is free.
    allocated: VirtAddr,
    /// Address up to which memory is already reserved.
    /// The range between `allocated` (inclusive) and `reserved` has already been claimed from the virtual memory manager,
    /// but not yet allocated to clients of the heap.
    reserved: VirtAddr,
    /// Address up to which the heap may grow. No allocations may exceed this address, and once the `allocated` pointer reaches it,
    /// no further allocations are possible.
    limit: VirtAddr,
}

impl BumpHeap {
    pub fn new(heap_start: VirtAddr, heap_end: VirtAddr) -> BumpHeap {
        BumpHeap {
            allocated: heap_start.align_up(super::PAGE_SIZE),
            reserved: heap_start.align_up(super::PAGE_SIZE),
            limit: heap_end.align_down(super::PAGE_SIZE)
        }
    }

    pub unsafe fn alloc(&mut self, layout: alloc::Layout) -> *mut u8 {
        let start = self.allocated.align_up(layout.align());
        let end = start.add(layout.size());
        if self.reserve(end) {
            self.allocated = end;
            start.as_mut_ptr()
        } else {
            core::ptr::null_mut()
        }
    }

    /// Reserves memory up to the given address, which will be rounded up to the next physical page size.
    /// Returns whether the reservation was successful. Even when returning `false`, it may already have
    /// allocated pages, but it couldn't allocate enough for the request to be fulfilled.
    /// The allocated pages are not lost. Subsequent smaller allocations may still succeed.
    pub unsafe fn reserve(&mut self, up_to: VirtAddr) -> bool {
        let page_aligned_limit = up_to.align_up(super::PAGE_SIZE);
        if page_aligned_limit > self.limit {
            return false;
        }

        while self.reserved < page_aligned_limit {
            match super::pfa::alloc_frame() {
                None => return false,
                Some(frame) => {
                    super::vmm::mmap(self.reserved, frame.start_address());
                    self.reserved = self.reserved.add(super::PAGE_SIZE);
                }
            }
        }
        true
    }
}

pub struct KernelAllocator {
    implementation: Mutex<Option<BumpHeap>>
}

impl KernelAllocator {
    pub const fn new() -> Self {
        KernelAllocator { implementation: Mutex::new(None) }
    }

    pub unsafe fn init(&self, heap_start: VirtAddr, heap_end: VirtAddr) {
        let mut allocator = self.implementation.lock();
        *allocator = Some(BumpHeap::new(heap_start, heap_end))
    }
}

unsafe impl alloc::GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: alloc::Layout) -> *mut u8 {
        let mut allocator = self.implementation.lock();
        match allocator.as_mut() {
            None => core::ptr::null_mut(),
            Some(allocator) => allocator.alloc(layout),
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: alloc::Layout) {
        debugln!("[HEAP] dealloc has no effect yet, memory is lost")
    }
}