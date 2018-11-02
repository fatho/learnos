//! Parser for the Multiboot2 memory map.

use crate::addr::{PhysAddr};

use core::iter::{Iterator};
use core::fmt;

use super::raw;

#[derive(Debug)]
pub struct MemoryMap {
    tag: *const MemoryMapTagRaw,
}

impl MemoryMap {
    pub unsafe fn from_raw(tag_raw: *const raw::Tag) -> MemoryMap {
        assert!((*tag_raw).tag_type == 6);
        
        let mem_tag = tag_raw as *const MemoryMapTagRaw;

        MemoryMap {
            tag: mem_tag,
        }
    }

    pub fn regions(&self) -> Entries {
        unsafe {
            Entries {
                current: self.tag.add(1) as *const Region,
                end: (self.tag as *const u8).add((*self.tag).header.size as usize) as *const Region,
            }
        }
    }
}

/// An iterator over the entries of a multiboot2 memory map.
pub struct Entries {
    current: *const Region,
    end: *const Region,
}

impl Iterator for Entries {
    type Item = &'static Region;

    fn next(&mut self) -> Option<Self::Item> {
        assert!(self.current <= self.end);
        if self.current == self.end {
            None
        } else {
            unsafe {
                let entry = &*self.current;
                self.current = self.current.add(1);
                Some(entry)
            }
        }
    }
}

/// The type of an entry in the memory map.
#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(C)]
pub struct EntryType(u32);

impl EntryType {
    pub const AVAILABLE: EntryType = EntryType(1);
    pub const AVAILABLE_ACPI: EntryType = EntryType(3);
    pub const RESERVED_HIBERNATION: EntryType = EntryType(4);
    pub const DEFECTIVE: EntryType = EntryType(5);
}

impl fmt::Debug for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match self.0 {
            1 => "Available",
            3 => "AvailableACPI",
            4 => "ReservedHibernation",
            5 => "Defective",
            _ => "Reserved"
        };
        write!(f, "EntryType({} ~ {})", self.0, description)
    }
}

#[repr(C)]
pub struct Region {
    base_addr: PhysAddr,
    length: u64,
    entry_type: EntryType,
    reserved: u32,
}

impl Region {
    /// Return whether the memory range described by this entry is available to the OS.
    pub fn is_available(&self) -> bool {
        let type_ = self.entry_type;
        type_ == EntryType::AVAILABLE
    }

    pub fn base_addr(&self) -> PhysAddr {
        self.base_addr
    }

    pub fn length(&self) -> u64 {
        self.length
    }

    pub fn entry_type(&self) -> EntryType {
        self.entry_type
    }
}

#[repr(C)]
struct MemoryMapTagRaw {
    header: raw::Tag,
    entry_size: u32,
    entry_version: u32,
}