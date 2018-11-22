//! This module provides functionality for manipulating page tables.
//! 
//! TODO: deallocate page tables when they become unused

pub mod direct;
pub mod tables;

use self::tables::{PageTableEntry};
use crate::physical::alloc::PageFrameAllocator;
use amd64::{Alignable, PhysAddr, VirtAddr};

/// Index into the PML4 where it recursively maps onto itself.
pub const PML4_RECURSIVE_MAPPING_INDEX: usize = 510;

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
pub enum MappingLevel {
    /// Map a 4 KiB page
    Page4K,
    /// Map a 2 MiB page
    Page2M
}

/// Map a virtual address to the given physical address.
/// The given page frame allocator is used for allocating additional page tables.
pub unsafe fn mmap(vaddr: VirtAddr, paddr: PhysAddr, level: MappingLevel, pfa: &mut PageFrameAllocator) {
    // ensure address is correctly aligned
    let (required_alignment, mapping_level) = match level {
        MappingLevel::Page4K => (crate::PAGE_SIZE, 0),
        MappingLevel::Page2M => (crate::LARGE_PAGE_SIZE, 1),
    };
    assert!(paddr.is_aligned(required_alignment));
    assert!(vaddr.is_aligned(required_alignment));
    trace!("[VMM] mmap({:p}, {:p})", vaddr, paddr);
    // make sure the PDP, PD and PT tables exist
    for i in 0..4 {
        let current_level = 3 - i;
        let entry_addr = entry_at_level(current_level, vaddr);
        let entry: &mut PageTableEntry = &mut *entry_addr.as_mut_ptr();
        if ! entry.flags().contains(tables::Flags::PRESENT) {
            if current_level > mapping_level {
                trace!("[VMM] allocating new page table at level {}", current_level);
                // no entry on that level yet, allocate a table
                let new_table = pfa.alloc().expect("VMM out of memory");
                // and assign it to the entry
                let mut new_entry = PageTableEntry::new();
                new_entry.set_base(new_table.start_address());
                new_entry.set_flags(tables::Flags::PRESENT | tables::Flags::WRITABLE);
                *entry = new_entry;
                // make sure it's available
                let new_table_addr = table_at_level(current_level - 1, vaddr);
                invalidate_address(new_table_addr);
                // clear out page table before attempting to reference anything in it
                crate::util::memset(new_table_addr.as_mut_ptr(), crate::PAGE_SIZE, 0)
            } else {
                trace!("[VMM] setting entry at level {}", current_level);
                // set the page table entry
                let mut new_entry = PageTableEntry::new();
                new_entry.set_base(paddr);
                new_entry.set_flags(tables::Flags::PRESENT | tables::Flags::WRITABLE);
                *entry = new_entry;
                invalidate_address(vaddr);
                break;
            }
        } else if current_level > 0 && entry.flags().contains(tables::Flags::SIZE) {
            panic!("Address already mapped at a conflicting size")
        }
    }
}

/// Unmap a virtual address.
/// The given page frame allocator is used for allocating additional page tables.
pub unsafe fn unmmap(_vaddr: VirtAddr) {
    unimplemented!()
}

/// Return the index in the page table at the given level (0 is PT, 3 is PML4)
/// that is responsible for mapping the given virtual address.
pub fn index_at_level(level: u8, vaddr: VirtAddr) -> usize {
    const INDEX_MASK: usize = 0x1FF;
    const INDEX_WIDTH: u8 = 9;

    (vaddr.0 >> (12 + INDEX_WIDTH * level)) & INDEX_MASK
}

/// Return the virtual address of the page table at the given level (0 is PT, 3 is PML4)
/// that contains the entry for the virtual address in question.
pub fn table_at_level(level: u8, vaddr: VirtAddr) -> VirtAddr {
    // compute the address of the entry, then align to page boundary
    entry_at_level(level, vaddr).align_down(4096)
}

/// Return the virtual address of the page table entry at the given level (0 is PT, 3 is PML4)
/// that contains the entry for the virtual address in question.
pub fn entry_at_level(level: u8, vaddr: VirtAddr) -> VirtAddr {
    const CLEAR_PML4_INDEX: usize = 0xFFFF_007F_FFFF_FFFF;
    const SET_PML4_RECURSIVE_INDEX: usize = PML4_RECURSIVE_MAPPING_INDEX << 39;
    let mut addr: usize = vaddr.0;
    
    for _current_level in 0..(level + 1) {
        addr = ((addr >> 9) & CLEAR_PML4_INDEX) | SET_PML4_RECURSIVE_INDEX;
    }

    // make sure address is canonical
    if PML4_RECURSIVE_MAPPING_INDEX >= 256 {
        addr |= 0xFFFF_0000_0000_0000;
    } else {
        addr &= 0x0000_FFFF_FFFF_FFFF;
    }
    // align to table entry
    addr &= 0xFFFF_FFFF_FFFF_FFF8;

    VirtAddr(addr)
}

pub unsafe fn invalidate_address(vaddr: VirtAddr) {
    asm!("invlpg [$0]" : : "r"(vaddr.0) : : "intel", "volatile")
}
