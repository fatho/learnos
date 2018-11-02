//! Parser for the Multiboot2 information structures provided by the bootloader.

use crate::addr::{VirtAddr};
use crate::mem_util;

use core::iter::{Iterator};
use core::str;

pub mod memmap;

/// Read-only pointer to the multiboot2 info data. Care must be taken not to overwrite
/// that memory region when the multiboot data is still needed.
#[derive(Debug)]
pub struct Multiboot2Info {
    header: *const raw::Header,
}

impl Multiboot2Info {
    pub unsafe fn from_virt(addr: VirtAddr) -> Multiboot2Info {
        Multiboot2Info {
            header: addr.as_ptr(),
        }
    }

    pub fn start_addr(&self) -> VirtAddr {
        VirtAddr(self.header as u64)
    }

    pub fn length(&self) -> usize {
        unsafe { (*self.header).total_size as usize }
    }

    pub fn tags(&self) -> Tags {
        Tags {
            current: self.start_addr().add(core::mem::size_of::<raw::Header>() as u64),
            end: self.start_addr().add(self.length() as u64),
        }
    }
}

pub struct Tags {
    current: VirtAddr,
    end: VirtAddr,
}

impl Iterator for Tags {
    type Item = Tag;

    fn next(&mut self) -> Option<Self::Item> {
        // tags must be terminated by a tag of type 0,
        // so we should never exceed the end address
        assert!(self.current < self.end);
        unsafe {
            let tag_start = self.current;
            // parse tag header
            let tag_header_ptr = tag_start.0 as *const raw::Tag;
            let size = (*tag_header_ptr).size;
            let tag_type = (*tag_header_ptr).tag_type;

            // sanity check that size covers at least the two header fields
            assert!(size >= 8);

            // goto next tag starting on 8 byte alignment
            self.current = tag_start.add(size as u64).align_up(3);

            if tag_type == 0 {
                None
            } else {
                let tag = match tag_type {
                    1 => {
                        assert!(size >= 9); // tag must contain at least the 0 terminator
                        Tag::BootCommandLine(mem_util::str_from_addr(tag_start.add(8), (size - 9) as usize).unwrap())
                    },
                    2 => {
                        assert!(size >= 9); // tag must contain at least the 0 terminator
                        Tag::BootLoaderName(mem_util::str_from_addr(tag_start.add(8), (size - 9) as usize).unwrap())
                    },
                    6 => Tag::MemoryMap(memmap::MemoryMap::from_raw(tag_header_ptr)),
                    // 10: APM table, uninteresting
                    _ => Tag::Other(tag_type, tag_start)
                };
                Some(tag)
            }
        }
    }
}

#[derive(Debug)]
pub enum Tag {
    /// Comand line that was passed to the kernel by the bootloader.
    BootCommandLine(&'static str),
    /// Name of the Multiboot2 compliant bootloader that loaded the kernel.
    BootLoaderName(&'static str),
    /// TODO: provide means for iterating through memory map
    MemoryMap(memmap::MemoryMap),
    /// Some other tag with the given type, starting at the given address.
    Other(u32, VirtAddr)
}

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