//! Parser for the Multiboot2 information structures provided by the bootloader.

use crate::addr::{VirtAddr};
use crate::mem_util;

use core::iter::{Iterator};
use core::str;
use core::marker;

/// Handle to the multiboot2 info data. While this is active, care must be taken not to overwrite the data in memory.
/// The liftetime of the data extracted from a `Multiboot2Info` value is tied to the lifetime of the `Multiboot2Info`
/// value itself.
#[derive(Debug)]
pub struct Multiboot2Info {
    start: VirtAddr,
    end: VirtAddr
}

impl Multiboot2Info {
    pub unsafe fn from_virt(addr: VirtAddr) -> Multiboot2Info {
        let header = addr.0 as *const HeaderRaw;
        Multiboot2Info {
            start: addr,
            end: addr.add((*header).total_size as u64)
        }
    }

    pub fn tags(&self) -> Tags {
        Tags {
            current: self.start.add(8),
            end: self.end,
            _lifetime: marker::PhantomData
        }
    }
}

pub struct Tags<'a> {
    current: VirtAddr,
    end: VirtAddr,
    _lifetime: marker::PhantomData<&'a u8>
}

impl<'a> Iterator for Tags<'a> {
    type Item = Tag<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // tags must be terminated by a tag of type 0,
        // so we should never exceed the end address
        assert!(self.current < self.end);
        unsafe {
            let tag_start = self.current;
            // parse tag header
            let tag_header_ptr = tag_start.0 as *const TagRaw;
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
                    6 => Tag::MemoryMap,
                    // 10: APM table, uninteresting
                    _ => Tag::Other(tag_type, tag_start)
                };
                Some(tag)
            }
        }
    }
}

#[derive(Debug)]
pub enum Tag<'a> {
    /// Comand line that was passed to the kernel by the bootloader.
    BootCommandLine(&'a str),
    /// Name of the Multiboot2 compliant bootloader that loaded the kernel.
    BootLoaderName(&'a str),
    /// TODO: provide means for iterating through memory map
    MemoryMap,
    /// Some other tag with the given type, starting at the given address.
    Other(u32, VirtAddr)
}


#[repr(C, packed)]
struct HeaderRaw {
    total_size: u32,
    reserved: u32
}

#[repr(C, packed)]
struct TagRaw {
    tag_type: u32,
    size: u32
}
