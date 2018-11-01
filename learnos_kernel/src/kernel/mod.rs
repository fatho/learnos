//! Parts that are specific to this kernel and cannot be easily reused.

mod layout;
mod diagnostics;

#[cfg(not(test))]
use core::panic::PanicInfo;
use core::fmt::{Write};

use crate::addr::{PhysAddr, PhysAddr32};
use crate::vga;
use crate::console;

use crate::multiboot2;

use crate::paging;

/// 
pub fn main(multiboot_info: PhysAddr32) -> ! {
    // Initialize VGA buffer. Besides panics, this is the only place where this should happen.
    let vga_addr = layout::low_phys_to_virt(vga::VGA_PHYS_ADDR);
    let vgabuf = unsafe { vga::VgaMem::with_addr(vga_addr) };
    let mut console = console::Console::new(vgabuf);

    // create page frame allocator for low 2 GiB
    let mut pfa = unsafe { paging::alloc::PageFrameAllocator::new(PhysAddr(0), layout::LOW_PHYS_MAX, layout::KERNEL_VIRTUAL_BASE) };

    // prepare multiboot info parsing
    let mb2 = unsafe { multiboot2::Multiboot2Info::from_virt(layout::low_phys_to_virt(multiboot_info.extend())) };
    diagnostics::print_multiboot(&mut console, &mb2);

    diagnostics::print_heap_info(&mut console);

    // only consider addresses above the heap start as free
    let heap_start = layout::heap_start().align_up(12);

    for tag in mb2.tags() {
        match tag {
            multiboot2::Tag::MemoryMap(mmap) => {
                for entry in mmap.entries() {
                    if entry.is_available() {
                        unsafe {
                            if entry.base_addr >= heap_start {
                                pfa.add_space(entry.base_addr, (entry.length >> 12) as u32);
                            } else if entry.base_addr.add(entry.length) >= heap_start {
                                let unused = heap_start.0 - entry.base_addr.0;
                                let adj_len = (entry.length - unused) >> 12;
                                pfa.add_space(heap_start, adj_len as u32);
                            }
                        }
                    }
                }
            },
            _ => {}
        }
    }
    diagnostics::print_pfa_info(&mut console, &pfa);


    let a = pfa.alloc(4).unwrap();
    let b = pfa.alloc(3).unwrap();
    pfa.free(a);
    let c = pfa.alloc(2).unwrap();
    pfa.free(b);

    diagnostics::print_pfa_info(&mut console, &pfa);


    halt!();
}

#[panic_handler]
#[cfg(not(test))]
fn panic(panic_info: &PanicInfo) -> ! {
    // System is FUBAR anyway, just grab a new instance of VGA buffer and hope we get some info out
    let vga_addr = layout::low_phys_to_virt(vga::VGA_PHYS_ADDR);
    let vgabuf = unsafe { vga::VgaMem::with_addr(vga_addr) };
    let mut console = console::Console::with_colors(vgabuf, vga::Color::White, vga::Color::Red);

    writeln!(console, "{}", panic_info);

    halt!();
}