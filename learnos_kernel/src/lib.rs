#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), feature(global_asm))]
#![feature(naked_functions)]
#![feature(link_args)]
#![feature(asm)]
#![feature(get_type_id)]
#![feature(const_fn)]

#[macro_use]
extern crate core;

macro_rules! halt {
    () => {
        loop {
            unsafe {
                asm!("hlt" : /* no outputs */ : /* no inputs */ : /* no clobbers */ : "volatile");
            }
        }
    };
}

// reusable parts
pub mod addr;
pub mod vga;
pub mod multiboot2;
pub mod mem_util;
pub mod memory;
pub mod spin;

mod kernel;

/// This is the Rust entry point that is called by the assembly boot code after switching to long mode.
#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: addr::PhysAddr32) -> ! {
    kernel::main(multiboot_info)
}