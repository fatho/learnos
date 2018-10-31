#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), feature(global_asm))]
#![feature(naked_functions)]
#![feature(link_args)]
#![feature(asm)]
#![feature(get_type_id)]

#[macro_use]
extern crate core;
#[cfg(not(test))]
use core::panic::PanicInfo;
use core::fmt::{Write};

pub mod addr;
pub mod vga;
pub mod console;

macro_rules! halt {
    () => {
        loop {
            unsafe {
                asm!("hlt" : /* no outputs */ : /* no inputs */ : /* no clobbers */ : "volatile");
            }
        }
    };
}

/// This is the Rust entry point that is called by the assembly boot code after switching to long mode.
#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn rust_main(multiboot_info: addr::PhysAddr32) -> ! {
    // Initialize VGA buffer. Besides panics, this is the only place where this should happen.
    let vgabuf = unsafe { vga::VgaMem::with_addr(vga::VGA_PHYS_ADDR.identity_mapping()) };
    let mut console = console::Console::new(vgabuf);

    writeln!(console, "Multiboot info structures @ {:?}", multiboot_info);

    halt!();
}

#[panic_handler]
#[cfg(not(test))]
fn panic(panic_info: &PanicInfo) -> ! {
    // System is FUBAR anyway, just grab a new instance of VGA buffer and hope we get some info out
    let vgabuf = unsafe { vga::VgaMem::with_addr(vga::VGA_PHYS_ADDR.identity_mapping()) };
    let mut console = console::Console::with_colors(vgabuf, vga::Color::White, vga::Color::Red);

    writeln!(console, "{}", panic_info);

    halt!();
}