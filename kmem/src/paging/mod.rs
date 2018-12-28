//! This module provides functionality for manipulating page tables.
//! 
//! TODO: deallocate page tables when they become unused

pub mod direct;

use amd64::paging;
use amd64::paging::{PageTableEntry};
use crate::physical::alloc::PageFrameAllocator;
use amd64::{Alignable, PhysAddr, VirtAddr};

/// Index of a level in the page table hierarchy. 0 represents the lowest level (4K pages).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Level(pub u32);

impl Level {
    /// Page Table level
    pub const PT: Level = Level(0);
    /// Page Directory level
    pub const PD: Level = Level(1);
    /// Page Directory Pointer level
    pub const PDP: Level = Level(2);
    /// Page Map Table Level 4 level
    pub const PML4: Level = Level(3);

    /// The parent level in the page table hierarchy.
    pub fn parent(&self) -> Level {
        Level(self.0 + 1)
    }

    /// The child level in the page table hierarchy, if the current level is not the leaf level (0).
    pub fn child(&self) -> Option<Level> {
        if self.0 == 0 {
            None
        } else {
            Some(Level(self.0 - 1))
        }
    }
}

pub trait AddressSpace {    
    /// Map a virtual address to the given physical address in this address space.
    ///
    /// # Arguments
    /// 
    /// * `vaddr` the virtual address that should be mapped
    /// * `paddr` the physical address to which the virtual address will be mapped
    /// * `level` the level in the page table hierarchy at which the mapping should be added
    ///   Level 0 refers to the smallest mapping unit (4K pages on AMD64).
    ///   Higher levels are not necessarily supported.
    /// * `pfa` a page frame allocator that is used for allocating new page tables if necessary
    unsafe fn map(&self, vaddr: VirtAddr, paddr: PhysAddr, level: Level, pfa: &mut PageFrameAllocator) -> Result<(), MapError>;
    /// Unmap a virtual address
    /// 
    /// * `vaddr` the virtual address that should be unmapped
    unsafe fn unmap(&self, vaddr: VirtAddr) -> Result<(), UnmapError>;

    /// Resolve a virtual address to a physical address in this address space.
    /// 
    /// # Returns
    /// 
    /// The physical address that the given virtual address is mapped to, or `None` if the
    /// virtual address is not mapped.
    unsafe fn resolve(&self, vaddr: VirtAddr) -> Option<PhysAddr>;

    /// The maximum level of this address space. This is the same for all address spaces on the same system,
    /// but it may vary depending on some CPU flags.
    fn max_level(&self) -> Level;
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum MapError {
    /// There can be no mappings on the requested level.
    InvalidLevel(Level),
    /// There is already a mapping at the given virtual address.
    MappingExists,
    /// There is no memory left for allocating new page tables.
    OutOfMemory,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum UnmapError {
    /// The mapping that should be unmapped does not exist.
    NoMapping,
}

/// Mask for extracting the 9-bit index into a page table.
const INDEX_MASK: usize = 0x1FF;
/// Width of the index in bits.
const INDEX_BIT_WIDTH: u32 = 9;

/// Return the index in the page table at the given level (0 is PT, 3 is PML4)
/// that is responsible for mapping the given virtual address.
pub fn index_at_level(level: Level, vaddr: VirtAddr) -> usize {
    (vaddr.0 >> (12 + INDEX_BIT_WIDTH * level.0)) & INDEX_MASK
}

/// Provides access to the current address space, assuming a recursive mapping at the given index.
pub struct CurrentRecursiveMapping {
    recursive_index: usize,
}

impl CurrentRecursiveMapping {
    pub fn new(recursive_index: usize) -> Self {
        assert!(recursive_index < 512);
        CurrentRecursiveMapping {
            recursive_index: recursive_index
        }
    }

    /// Return the virtual address of the page table at the given level (0 is PT, 3 is PML4)
    /// that contains the entry for the virtual address in question.
    pub fn table_at_level(&self, level: Level, vaddr: VirtAddr) -> VirtAddr {
        // compute the address of the entry, then align to page boundary
        self.entry_at_level(level, vaddr).align_down(4096)
    }

    /// Return the virtual address of the page table entry at the given level (0 is PT, 3 is PML4)
    /// that contains the entry for the virtual address in question.
    pub fn entry_at_level(&self, level: Level, vaddr: VirtAddr) -> VirtAddr {
        const CLEAR_PML4_MASK: usize = 0xFFFF_007F_FFFF_FFFF;
        let set_pml4_index_mask: usize = self.recursive_index << (3 * INDEX_BIT_WIDTH + crate::PAGE_ALIGN_BITS);
        let mut addr: usize = vaddr.0;
        
        for _current_level in 0 ..= level.0 {
            addr = ((addr >> INDEX_BIT_WIDTH) & CLEAR_PML4_MASK) | set_pml4_index_mask;
        }

        // make sure address is canonical
        if self.recursive_index >= 256 {
            addr |= 0xFFFF_0000_0000_0000;
        } else {
            addr &= 0x0000_FFFF_FFFF_FFFF;
        }
        // align to table entry
        addr &= 0xFFFF_FFFF_FFFF_FFF8;

        VirtAddr(addr)
    }

    /// Recursive implementation of mapping. If an error occurs, the current mapping is left unchanged.
    unsafe fn map_impl_rec(
        &self, vaddr: VirtAddr, paddr: PhysAddr, target_level: Level,
        pfa: &mut PageFrameAllocator, current_level: Level,
    ) -> Result<(), MapError> {
        let entry_addr = self.entry_at_level(current_level, vaddr);
        let entry: &mut PageTableEntry = &mut *entry_addr.as_mut_ptr();

        if current_level == target_level && ! entry.flags().contains(paging::Flags::PRESENT) {
            trace!("[VMM] setting entry at level {}", current_level.0);
            // compute flags of new entry
            let mut new_flags = paging::Flags::PRESENT | paging::Flags::WRITABLE;
            if current_level > Level::PT {
                // set huge page size flag if we're not mapping at the lowest level
                new_flags |= paging::Flags::SIZE;
            }
            // set the page table entry
            let mut new_entry = PageTableEntry::new();
            new_entry.set_base(paddr);
            new_entry.set_flags(new_flags);
            *entry = new_entry;
            // we must invalidate the cache
            amd64::paging::invalidate_tlb_address(vaddr);
            Ok(())
        } else if ! entry.flags().contains(paging::Flags::SIZE | paging::Flags::PRESENT) {
            // ^ there shouldn't be a valid mapping yet at this point

            let child_level = current_level.child().expect("we shouldn't be at the PT level yet");
            let old_entry = *entry;

            if ! entry.flags().contains(paging::Flags::PRESENT) {
                trace!("[VMM] allocating new page table at level {}", current_level.0);

                // no entry on that level yet, allocate a table
                let new_table = pfa.alloc().ok_or(MapError::OutOfMemory)?;

                // and assign it to the entry
                let mut new_entry = PageTableEntry::new();
                new_entry.set_base(new_table.start_address());
                new_entry.set_flags(paging::Flags::PRESENT | paging::Flags::WRITABLE);
                *entry = new_entry;
                // access the table via the recursive mapping:
                let new_table_addr = self.table_at_level(child_level, vaddr);
                amd64::paging::invalidate_tlb_address(new_table_addr);
                // clear out page table before attempting to reference anything in it
                crate::util::memset(new_table_addr.as_mut_ptr(), crate::PAGE_SIZE, 0);

                match self.map_impl_rec(vaddr, paddr, target_level, pfa, child_level) {
                    Ok(()) => Ok(()),
                    Err(err) => {
                        *entry = old_entry;
                        amd64::paging::invalidate_tlb_address(new_table_addr);
                        pfa.free(new_table);
                        Err(err)
                    }
                }
            } else {
                self.map_impl_rec(vaddr, paddr, target_level, pfa, child_level)
            }
        } else {
            Err(MapError::MappingExists)
        }
    }
}

impl AddressSpace for CurrentRecursiveMapping {
    unsafe fn map(
        &self, vaddr: VirtAddr, paddr: PhysAddr, level: Level, pfa: &mut PageFrameAllocator
    ) -> Result<(), MapError> 
    {
        // TODO: allow mapping 1GB pages if supported
        if level >= Level(2) {
            // can only map 4K and 2M pages
            return Err(MapError::InvalidLevel(level))
        }
        // ensure address is correctly aligned
        let required_alignment = 1 << (crate::PAGE_ALIGN_BITS + 9 * level.0);
        assert!(paddr.is_aligned(required_alignment));
        assert!(vaddr.is_aligned(required_alignment));
        trace!("[VMM] mmap({:p}, {:p})", vaddr, paddr);
        // perform actual mapping
        self.map_impl_rec(vaddr, paddr, level, pfa, Level::PML4)
    }

    unsafe fn unmap(&self, _vaddr: VirtAddr) -> Result<(), UnmapError> {
        unimplemented!()
    }

    unsafe fn resolve(&self, vaddr: VirtAddr) -> Option<PhysAddr> {
        let mut current_level = Level::PML4;
        loop {
            let entry: &PageTableEntry = &*self.entry_at_level(Level::PT, vaddr).as_ptr();

            if entry.flags().contains(paging::Flags::PRESENT) {
                if current_level == Level::PT || entry.flags().contains(paging::Flags::SIZE) {
                    let offset_mask = (1 << (crate::PAGE_ALIGN_BITS + INDEX_BIT_WIDTH * current_level.0)) - 1;
                    let base = entry.base();
                    let offset = vaddr.0 & offset_mask;
                    break Some(PhysAddr(base.0 + offset));
                } else {
                    // we're not at the lowest level yet (otherwise, we'd be in the above branch)
                    current_level = current_level.child().unwrap();
                }
            } else {
                break None;
            }
        }
    }

    fn max_level(&self) -> Level {
        Level::PML4
    }
}
