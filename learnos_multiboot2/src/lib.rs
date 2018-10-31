#![cfg_attr(not(test), no_std)]
//#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), feature(global_asm))]
#![feature(naked_functions)]
#![feature(link_args)]

mod addr;
mod vga;
mod console;

#[cfg(not(test))]
use core::panic::PanicInfo;

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn rust_main() -> ! {
    // initialize VGA buffer
    let vgabuf = unsafe { vga::Vga::with_addr(vga::VGA_PHYS_ADDR.identity_mapping()) };
    let mut console = console::Console::new(vgabuf);
    console.write(b"Hello World, it works!\n");
    console.write(b"Even with newlines\nIt's fantastic");
    console.write(b", really.\n");
    for i in 0..30 {
        console.write(b"This is repeated a few times and should wrap around\n");
    }
    let msg = format!("Oh no {} ", 123);
    console.write(b"A long text spanning more than eighty characters - which is not a lot I must note, as you can easily reach these lengths - should wrap around at the end of the line.\n");
    loop {}
}

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
