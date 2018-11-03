//! Parser for the Multiboot2 memory map.

use crate::addr::{PhysAddr};

use core::iter::{Iterator};
use core::fmt;

#[repr(C)]
pub struct MemoryMapTag {
    header: super::Tag,
    entry_size: u32,
    entry_version: u32,
    first_region: Region,
}

impl MemoryMapTag {
    pub fn regions(&self) -> Regions {
        unsafe {
            let length = (self.header.size - 16) / self.entry_size;
            let start = &self.first_region as *const Region;
            Regions {
                current: start,
                end: start.add(length as usize),
            }
        }
    }
}

/// An iterator over the entries of a multiboot2 memory map.
#[derive(Debug, Clone)]
pub struct Regions {
    current: *const Region,
    end: *const Region,
}

impl Iterator for Regions {
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

#[repr(C, packed)]
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

    pub fn length(&self) -> usize {
        self.length as usize
    }

    pub fn entry_type(&self) -> EntryType {
        self.entry_type
    }
}