//! Parser for the Multiboot2 information structures provided by the bootloader.

use crate::addr;
use crate::addr::{PhysAddr};

use core::iter::{Iterator};
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

    pub fn length(&self) -> usize {
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
        let offset = addr::align_up(self.size(), 8);
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

#[repr(C, packed)]
pub struct ModuleTag {
    common: Tag,
    mod_start: u32,
    mod_end: u32,
    cmd_line_start: u8,
}

impl ModuleTag {
    pub fn mod_start(&self) -> PhysAddr {
        PhysAddr(self.mod_start as usize)
    }

    pub fn mod_end(&self) -> PhysAddr {
        PhysAddr(self.mod_end as usize)
    }

    pub fn cmd_line(&self) -> &str {
        let cmd_line_ptr = &self.cmd_line_start as *const u8;
        assert!(self.common.size as usize >= core::mem::size_of::<ModuleTag>());
        let cmd_line_length = self.common.size as usize - core::mem::size_of::<ModuleTag>();
        unsafe {
            str::from_utf8(slice::from_raw_parts(cmd_line_ptr, cmd_line_length)).unwrap()
        }
    }
}


#[repr(C, packed)]
pub struct BootLoaderTag {
    common: Tag,
    name_start: u8,
}

impl BootLoaderTag {
    pub fn name(&self) -> &str {
        let name_ptr = &self.name_start as *const u8;
        assert!(self.common.size as usize >= core::mem::size_of::<BootLoaderTag>());
        let name_length = self.common.size as usize - core::mem::size_of::<BootLoaderTag>();
        unsafe {
            str::from_utf8(slice::from_raw_parts(name_ptr, name_length)).unwrap()
        }
    }
}


#[repr(C, packed)]
pub struct BootCommandLineTag {
    common: Tag,
    cmd_line_start: u8,
}

impl BootCommandLineTag {
    pub fn cmd_line(&self) -> &str {
        let cmd_line_ptr = &self.cmd_line_start as *const u8;
        assert!(self.common.size as usize >= core::mem::size_of::<BootCommandLineTag>());
        let cmd_line_length = self.common.size as usize - core::mem::size_of::<BootCommandLineTag>();
        unsafe {
            str::from_utf8(slice::from_raw_parts(cmd_line_ptr, cmd_line_length)).unwrap()
        }
    }
}