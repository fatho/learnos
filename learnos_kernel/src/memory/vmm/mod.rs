//! The virtual memory manager.
//! 
//! Requires that the page frame allocator has been initialized.
//! 
//! This module works with a hard-coded recursive page table mapping on the 510th entry of the PML4.

pub mod page_tables;

use self::page_tables::{PageTableEntry};
use super::pfa;
use crate::addr::{PhysAddr, VirtAddr};

/// Index into the PML4 where it recursively maps onto itself.
pub const PML4_RECURSIVE_MAPPING_INDEX: usize = 510;


/// Map a virtual address to the given physical address.
pub unsafe fn mmap(vaddr: VirtAddr, paddr: PhysAddr) {
    debugln!("mmap({:p}, {:p})", vaddr, paddr);
    // make sure the PDP, PD and PT tables exist
    for i in 0..4 {
        let current_level = 3 - i;
        let entry_addr = entry_at_level(current_level, vaddr);
        debugln!("  level {} - entry {:p}", current_level, entry_addr);
        let entry: &mut PageTableEntry = &mut *entry_addr.as_mut_ptr();
        if ! entry.flags().contains(page_tables::Flags::PRESENT) {
            if current_level > 0 {
                debugln!("  allocating new page table");
                // no entry on that level yet, allocate a table
                let new_table = pfa::alloc_frame().expect("VMM out of memory");
                debugln!("    at page frame {:p}", new_table.start_address());
                // and assign it to the entry
                let mut new_entry = PageTableEntry::new();
                new_entry.set_base(new_table.0.start_address());
                new_entry.set_flags(page_tables::Flags::PRESENT | page_tables::Flags::WRITABLE);
                *entry = new_entry;
                // make sure it's available
                invalidate_address(table_at_level(current_level - 1, vaddr));
            } else {
                debugln!("  setting PT entry");
                // set the page table entry
                let mut new_entry = PageTableEntry::new();
                new_entry.set_base(paddr);
                new_entry.set_flags(page_tables::Flags::PRESENT | page_tables::Flags::WRITABLE);
                *entry = new_entry;
                invalidate_address(vaddr);
            }
        } else if current_level > 0 && entry.flags().contains(page_tables::Flags::SIZE) {
            panic!("Address already mapped at a conflicting size")
        }
    }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_index_at_leve() {
        let a = VirtAddr(0b1111_1111_1111_1111_100000001_010000010_001000100_000101000_101010101010);
        assert_eq!(index_at_level(3, a), 257);
        assert_eq!(index_at_level(2, a), 130);
        assert_eq!(index_at_level(1, a), 68);
        assert_eq!(index_at_level(0, a), 40);
    }

    #[test]
    fn test_table_at_level() {
        let a    = VirtAddr(0b0000_0000_0000_0000_010000001_010000010_001000100_000101000_101010101010);
        let pt   = VirtAddr(0b1111_1111_1111_1111_111111110_010000001_010000010_001000100_000000000000);
        let pd   = VirtAddr(0b1111_1111_1111_1111_111111110_111111110_010000001_010000010_000000000000);
        let pdp  = VirtAddr(0b1111_1111_1111_1111_111111110_111111110_111111110_010000001_000000000000);
        let pml4 = VirtAddr(0b1111_1111_1111_1111_111111110_111111110_111111110_111111110_000000000000);
        assert_eq!(table_at_level(0, a), pt);
        assert_eq!(table_at_level(1, a), pd);
        assert_eq!(table_at_level(2, a), pdp);
        assert_eq!(table_at_level(3, a), pml4);
    }

    #[test]
    fn test_entry_at_level() {
        let a    = VirtAddr(0b0000_0000_0000_0000_010000001_010000010_001000100_000101000_101010101010);
        let pt   = VirtAddr(0b1111_1111_1111_1111_111111110_010000001_010000010_001000100_000101000000);
        let pd   = VirtAddr(0b1111_1111_1111_1111_111111110_111111110_010000001_010000010_001000100000);
        let pdp  = VirtAddr(0b1111_1111_1111_1111_111111110_111111110_111111110_010000001_010000010000);
        let pml4 = VirtAddr(0b1111_1111_1111_1111_111111110_111111110_111111110_111111110_010000001000);
        assert_eq!(entry_at_level(0, a), pt);
        assert_eq!(entry_at_level(1, a), pd);
        assert_eq!(entry_at_level(2, a), pdp);
        assert_eq!(entry_at_level(3, a), pml4);
    }
}