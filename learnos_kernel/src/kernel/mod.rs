//! Parts that are specific to this kernel and cannot be easily reused.

mod layout;

#[cfg(not(test))]
use core::panic::PanicInfo;
use core::fmt::{Write};

use crate::addr;
use crate::vga;
use crate::console;

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