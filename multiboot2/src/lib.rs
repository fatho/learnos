#![cfg_attr(not(test), no_std)]
//! Parser for the Multiboot2 information structures provided by the bootloader.
//! The lifetimes of the data extracted from the multiboot structures is 'static,
//! because it has been already present before Rust code is executed and it's not
//! going to be dropped.
//! 
//! However, when the physical memory where those data structures is unmapped or
//! remapped at a different address, the returned references are no longer valid.
//! If the mapping is not kept, make sure to drop all references first.
//! 
//! The safety of this parser depends on the bootloader being multiboot2 compliant.
//! If the bootloader provides bogus data, trying to parse it using this structures
//! likely ends in sadness.

use amd64::{Alignable, PhysAddr};

use core::iter::{Iterator, FusedIterator};
use core::str;
use core::slice;

pub mod memmap;

/// Root of Multiboot2 info data.
#[repr(C, packed)]
pub struct Multiboot2Info {
    total_size: u32,
    reserved: u32,
    first_tag: Tag,
}

impl Multiboot2Info {

    pub fn size(&self) -> usize {
        self.total_size as usize
    }

    pub fn tags(&self) -> TagsIter {
        TagsIter {
            current: &self.first_tag as *const Tag,
        }
    }

    pub fn modules(&self) -> impl Iterator<Item=&'static ModuleTag> {
        self.tags()
            .filter(|t| t.tag_type() == TagType::MODULE)
            .map(|t| (t as *const Tag) )
            .map(|t| unsafe { &*(t as *const ModuleTag) } )
    }

    pub fn memory_map(&self) -> Option<&'static memmap::MemoryMapTag> {
        self.tags()
            .find(|t| t.tag_type() == TagType::MEMORY_MAP)
            .map(|t| (t as *const Tag) )
            .map(|t| unsafe { &*(t as *const memmap::MemoryMapTag) } )
    }

    pub fn boot_cmd_line(&self) -> Option<&'static str> {
        self.tags()
            .find(|t| t.tag_type() == TagType::BOOT_CMD_LINE)
            .map(|t| (t as *const Tag) )
            .map(|t| unsafe { &*(t as *const BootCommandLineTag) } )
            .map(|t| t.cmd_line() )
    }

    pub fn bootloader_name(&self) -> Option<&'static str> {
        self.tags()
            .find(|t| t.tag_type() == TagType::BOOT_LOADER_NAME)
            .map(|t| (t as *const Tag) )
            .map(|t| unsafe { &*(t as *const BootLoaderTag) } )
            .map(|t| t.name() )
    }
}

#[repr(C, packed)]
pub struct Tag {
    tag_type: TagType,
    size: u32
}

impl Tag {
    pub fn tag_type(&self) -> TagType {
        self.tag_type
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    unsafe fn next(&self) -> *const Tag {
        let offset = self.size().align_up(8);
        ((self as *const Tag) as *const u8).add(offset) as *const Tag
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[repr(C)]
pub struct TagType(u32);

impl TagType {
    const END: TagType = TagType(0);
    const BOOT_CMD_LINE: TagType = TagType(1);
    const BOOT_LOADER_NAME: TagType = TagType(2);
    const MODULE: TagType = TagType(3);
    const MEMORY_MAP: TagType = TagType(6);
}

/// An iterator over the tags in the multiboot structure.
/// Construct using `Multiboot2Info::tags`.
pub struct TagsIter {
    current: *const Tag,
}

impl Iterator for TagsIter {
    type Item = &'static Tag;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let this = &*self.current;
            if this.tag_type() == TagType::END {
                None
            } else {
                self.current = this.next();
                Some(this)
            }
        }
    }
}

impl FusedIterator for TagsIter {}

#[repr(C, packed)]
pub struct ModuleTag {
    common: Tag,
    mod_start: u32,
    mod_end: u32,
    /// First byte of the command line. As the command line is specified to be a
    /// null-terminated UTF-8 string, it always consists of at least one byte.
    cmd_line_start: u8,
}

impl ModuleTag {
    /// Physical address where the module begins.
    pub fn mod_start(&self) -> PhysAddr {
        PhysAddr(self.mod_start as usize)
    }

    /// Physical address where the module ends (not included).
    pub fn mod_end(&self) -> PhysAddr {
        PhysAddr(self.mod_end as usize)
    }

    /// Return a reference to the command line of the module stored in the tag.
    /// 
    /// # Panics
    /// 
    /// This function panics when the string stored in the tag is not valid UTF-8,
    /// as it is mandated by the Multiboot2 specification.
    pub fn cmd_line(&self) -> &str {
        let cmd_line_ptr = &self.cmd_line_start as *const u8;
        assert!(self.common.size as usize >= core::mem::size_of::<ModuleTag>());
        let cmd_line_length = self.common.size as usize - core::mem::size_of::<ModuleTag>();
        unsafe {
            str::from_utf8(slice::from_raw_parts(cmd_line_ptr, cmd_line_length))
                .expect("Invalid UTF-8 string in Multiboot tag")
        }
    }
}


#[repr(C, packed)]
pub struct BootLoaderTag {
    common: Tag,
    /// First byte of the name. As the name is specified to be a
    /// null-terminated UTF-8 string, it always consists of at least one byte.
    name_start: u8,
}

impl BootLoaderTag {
    /// Return a reference to the bootloader name stored in the tag.
    /// 
    /// # Panics
    /// 
    /// This function panics when the string stored in the tag is not valid UTF-8,
    /// as it is mandated by the Multiboot2 specification.
    pub fn name(&self) -> &str {
        let name_ptr = &self.name_start as *const u8;
        assert!(self.common.size as usize >= core::mem::size_of::<BootLoaderTag>());
        let name_length = self.common.size as usize - core::mem::size_of::<BootLoaderTag>();
        unsafe {
            str::from_utf8(slice::from_raw_parts(name_ptr, name_length))
                .expect("Invalid UTF-8 string in Multiboot tag")
        }
    }
}


#[repr(C, packed)]
pub struct BootCommandLineTag {
    common: Tag,
    /// First byte of the command line. As the command line is specified to be a
    /// null-terminated UTF-8 string, it always consists of at least one byte.
    cmd_line_start: u8,
}

impl BootCommandLineTag {
    /// Return a reference to the command line stored in the tag.
    /// 
    /// # Panics
    /// 
    /// This function panics when the string stored in the tag is not valid UTF-8,
    /// as it is mandated by the Multiboot2 specification.
    pub fn cmd_line(&self) -> &str {
        let cmd_line_ptr = &self.cmd_line_start as *const u8;
        assert!(self.common.size as usize >= core::mem::size_of::<BootCommandLineTag>());
        let cmd_line_length = self.common.size as usize - core::mem::size_of::<BootCommandLineTag>();
        unsafe {
            str::from_utf8(slice::from_raw_parts(cmd_line_ptr, cmd_line_length))
                .expect("Invalid UTF-8 string in Multiboot tag")
        }
    }
}

// rust doesn't see that we're conjuring the structs from raw pointers
#[allow(dead_code)]
mod raw {
    #[repr(C, packed)]
    pub struct Header {
        pub total_size: u32,
        pub reserved: u32
    }

    #[repr(C, packed)]
    pub struct Tag {
        pub tag_type: u32,
        pub size: u32
    }
}
