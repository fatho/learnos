//! Parser for the Multiboot2 memory map.

use crate::addr::{PhysAddr};

use core::iter::{Iterator};
use core::marker;

use super::raw;

#[derive(Debug)]
pub struct MemoryMap<'a> {
    tag: *const MemoryMapTagRaw,
    _lifetime: marker::PhantomData<&'a MemoryMapTagRaw>,
}

impl<'a> MemoryMap<'a> {
    pub unsafe fn from_raw(tag_raw: *const raw::Tag) -> MemoryMap<'a> {
        assert!((*tag_raw).tag_type == 6);
        
        let mem_tag = tag_raw as *const MemoryMapTagRaw;

        MemoryMap {
            tag: mem_tag,
            _lifetime: marker::PhantomData,
        }
    }

    pub fn entries(&self) -> Entries<'a> {
        unsafe {
            Entries {
                current: self.tag.add(1) as *const EntryRaw,
                end: (self.tag as *const u8).add((*self.tag).header.size as usize) as *const EntryRaw,
                _lifetime: self._lifetime
            }
        }
    }
}

/// An iterator over the entries of a multiboot2 memory map.
pub struct Entries<'a> {
    current: *const EntryRaw,
    end: *const EntryRaw,
    _lifetime: marker::PhantomData<&'a MemoryMapTagRaw>,
}

impl<'a> Iterator for Entries<'a> {
    type Item = Entry;

    fn next(&mut self) -> Option<Entry> {
        assert!(self.current <= self.end);
        if self.current == self.end {
            None
        } else {
            unsafe {
                let entry = Entry {
                    base_addr: PhysAddr((*self.current).base_addr),
                    length: (*self.current).length,
                    entry_type: EntryType::from_raw((*self.current).entry_type),
                };
                self.current = self.current.add(1);
                Some(entry)
            }
        }
    }
}

/// An entry in the memory map.
pub struct Entry {
    pub base_addr: PhysAddr,
    pub length: u64,
    pub entry_type: EntryType
}

impl Entry {
    /// Return whether the memory range described by this entry is available to the OS.
    pub fn is_available(&self) -> bool {
        self.entry_type == EntryType::Available
    }
}

/// The type of an entry in the memory map.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum EntryType {
    /// Memory that is available for the OS to use.
    Available,
    /// Available memory that contains ACPI information
    /// (should not be overwritten until it's not needed anymore).
    AvailableACPI,
    /// Reserved memory that needs to be preserved on hibernation.
    ReservedHibernation,
    /// Defective memory.
    Defective,
    /// Otherwisely reserved memory.
    Reserved,
}

impl EntryType {
    fn from_raw(raw_type: u32) -> EntryType {
        match raw_type {
            1 => EntryType::Available,
            3 => EntryType::AvailableACPI,
            4 => EntryType::ReservedHibernation,
            5 => EntryType::Defective,
            _ => EntryType::Reserved
        }
    }
}


#[repr(C, packed)]
struct MemoryMapTagRaw {
    header: raw::Tag,
    entry_size: u32,
    entry_version: u32,
}

#[repr(C, packed)]
struct EntryRaw {
    base_addr: u64,
    length: u64,
    entry_type: u32,
    reserved: u32,
}