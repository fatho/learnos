//! Private initialization functions
use amd64::{PhysAddr, PhysAddrRange};

use kmem::physical::mgmt::{PageFrameTable};
use kmem::physical::{PageFrame, PageFrameRegion};
use kmem::paging::direct::DirectMapping;

use core::cmp;

use super::{PhysicalMemoryLayout, PhysicalMemoryRegion};

/// Initialize the page frame table which is used for managing physical memory.
/// It holds one entry per page frame in physical memory.
/// 
/// 1. Based on the `memory_map` and `layout`, a contiguous region of available memory is
///    selected for holding the page frame table.
/// 2. The important regions in `layout` and the page frame table itself are marked as allocated,
///    all unavalable regions in `memory_map` are marked as reserved.
pub unsafe fn initialize_page_frame_table<I>(
    layout: &PhysicalMemoryLayout, memory_map: I, mapping: &DirectMapping
) -> PageFrameTable where
    I: Clone + Iterator<Item=PhysicalMemoryRegion>
{
    let heap_start_frame = PageFrame::next_above(layout.heap_start);

    debug!("[kmem] first frame = {:p}", heap_start_frame.start_address());

    // Compute initial allocation regions: all available RAM regions, rounded down to page sizes,
    // and above the important kernel data.
    let available_regions = memory_map.clone()
        .filter(|r| r.available)
        .map(|r| PageFrameRegion {
            start: cmp::max(r.frames.start, heap_start_frame),
            end: r.frames.end
        })
        .filter(|r| ! r.is_empty());

    // compute size required size of page frame table
    let page_frame_count = memory_map.clone()
        .map(|r| r.frames.end.0)
        .max().unwrap_or(0);
    
    let page_frame_table_size = PageFrameTable::required_size_bytes(page_frame_count);

    debug!("[kmem] #pfa={} tblsize={} B", page_frame_count, page_frame_table_size);

    // manually allocate page frame table
    let page_frame_table_addr = available_regions.clone()
        .find(|r| r.length() * kmem::PAGE_SIZE >= page_frame_table_size)
        .map(|r| r.start.start_address())
        .expect("cannot allocate page frame table");

    debug!("[kmem] tbladdr={:p}", page_frame_table_addr);

    let mut page_frame_table =
        PageFrameTable::from_addr(
            mapping.phys_to_virt(page_frame_table_addr),
            page_frame_count
        );

    debug!("[kmem] marking reserved memory areas");

    // mark all BIOS reserved areas
    memory_map.clone()
        .filter(|r| ! r.available)
        .for_each(|r| page_frame_table.mark_reserved(r.frames));

    // mark page frame table as allocated
    page_frame_table.mark_allocated(PageFrameRegion::new_including(
        &PhysAddrRange::new(page_frame_table_addr, page_frame_table_size)));

    // mark kernel area as reserved
    page_frame_table.mark_allocated(PageFrameRegion::new_including(&layout.kernel_memory));

    // mark rest until heap_start as allocated (may be freed later)
    page_frame_table.mark_allocated(PageFrameRegion::new_including(
        &PhysAddrRange::from_bounds(PhysAddr(0), layout.heap_start)));

    page_frame_table
}
