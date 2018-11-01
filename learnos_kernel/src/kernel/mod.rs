//! Parts that are specific to this kernel and cannot be easily reused.

mod layout;

#[cfg(not(test))]
use core::panic::PanicInfo;
use core::fmt::{Write};

use crate::addr;
use crate::vga;
use crate::console;

use crate::multiboot2;

/// 
pub fn main(multiboot_info: addr::PhysAddr32) -> ! {
    // Initialize VGA buffer. Besides panics, this is the only place where this should happen.
    let vga_addr = layout::low_phys_to_virt(vga::VGA_PHYS_ADDR);
    let vgabuf = unsafe { vga::VgaMem::with_addr(vga_addr) };
    let mut console = console::Console::new(vgabuf);

    writeln!(console, "Multiboot info structures @ {:p}", multiboot_info);

    // sanity check about virtual addresses
    let where_am_i: u64;
    unsafe {
        asm!("lea rax, [rip]"
             : "={rax}"(where_am_i)
             : 
             : 
             : "intel"
             );
    }
    writeln!(console, "main @ {:p}", (main as *const u8));
    writeln!(console, "RIP  @ {:p}", (where_am_i as *const u8));
    let stack_addr: u64;
    unsafe {
        asm!("mov rax, rsp"
             : "={rax}"(stack_addr)
             : 
             : 
             : "intel"
             );
    }
    writeln!(console, "RSP  @ {:p}", (stack_addr as *const u8));

    let mb2 = unsafe { multiboot2::Multiboot2Info::from_virt(layout::low_phys_to_virt(multiboot_info.extend())) };
    writeln!(console, "Multiboot2");
    for tag in mb2.tags() {
        match tag {
            multiboot2::Tag::MemoryMap(mmap) => {
                writeln!(console, "Memory map:");
                writeln!(console, "{: ^6} {: ^23} {: ^18}", "Type", "Physical Address", "Length");
                let mut total_available = 0;
                for e in mmap.entries() {
                    let type_ch = match e.entry_type {
                        multiboot2::memmap::EntryType::Available => 'A',
                        multiboot2::memmap::EntryType::AvailableACPI => 'C',
                        multiboot2::memmap::EntryType::ReservedHibernation => 'H',
                        multiboot2::memmap::EntryType::Defective => 'X',
                        multiboot2::memmap::EntryType::Reserved => 'R',
                    };
                    writeln!(console, "{: ^6} {: ^23p} {:016x}", type_ch, e.base_addr, e.length);
                    if e.is_available() {
                        total_available += e.length;
                    }
                }
                writeln!(console, " Available: {} MiB", total_available / 1024 / 1024);
            },
            multiboot2::Tag::BootCommandLine(cmdline) => {
                writeln!(console, "Command: {:?}", cmdline);
            },
            multiboot2::Tag::BootLoaderName(loader) => {
                writeln!(console, "Loader: {:?}", loader);
            }
            multiboot2::Tag::Other(id, _) => {
                writeln!(console, "Unknown tag: type={}", id);
            }
        }
    }

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