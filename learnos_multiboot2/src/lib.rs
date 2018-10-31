#![cfg_attr(not(test), no_std)]
//#![cfg_attr(not(test), no_main)]
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

/// This is the Rust entry point that is called by the assembly boot code after switching to long mode.
#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn rust_main() -> ! {
    // Initialize VGA buffer. Besides panics, this is the only place where this should happen.
    let vgabuf = unsafe { vga::VgaMem::with_addr(vga::VGA_PHYS_ADDR.identity_mapping()) };
    let mut console = console::Console::new(vgabuf);

    // Some test output
    console.write_bytes(b"Hello World, it works!\n");
    console.write_bytes(b"Even with newlines\nIt's fantastic");
    console.write_bytes(b", really.\n");
    for _i in 0..30 {
        console.write_bytes(b"This is repeated a few times and should wrap around\n");
    }
    console.write_bytes(b"A long text spanning more than eighty characters - which is not a lot I must note, as you can easily reach these lengths - should wrap around at the end of the line.\n");

    // Rust can format stuff without std library, that's cool!
    writeln!(console, "The int {}", 42);

    // Panics are properly handled as well.
    panic!("TODO: implement rest of OS!");
}

#[panic_handler]
#[cfg(not(test))]
fn panic(panic_info: &PanicInfo) -> ! {
    // System is FUBAR anyway, just grab a new instance of VGA buffer and hope we get some info out
    let vgabuf = unsafe { vga::VgaMem::with_addr(vga::VGA_PHYS_ADDR.identity_mapping()) };
    let mut console = console::Console::with_colors(vgabuf, vga::Color::White, vga::Color::Red);

    writeln!(console, "{}", panic_info);

    loop {
        unsafe {
            asm!("hlt" : /* no outputs */ : /* no inputs */ : /* no clobbers */ : "volatile");
        }
    }
}
