#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), feature(global_asm))]
#![feature(naked_functions)]
#![feature(link_args)]
#![feature(asm)]
#![feature(get_type_id)]
#![feature(const_fn)]
#![feature(format_args_nl)] // needed for debugln! macro

#[macro_use]
extern crate core;
#[macro_use]
extern crate static_assertions;
#[macro_use]
extern crate bitflags;

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
pub mod acpi;
pub mod addr;
#[macro_use]
pub mod diagnostics;
pub mod vga;
pub mod multiboot2;
pub mod memory;
pub mod serial;
pub mod spin;
pub mod interrupts;
pub mod portio;

// kernel specific part
mod kernel;

/// Arguments passed to the kernel by the loader.
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct KernelArgs {
    kernel_start: addr::PhysAddr,
    kernel_end: addr::PhysAddr,
    multiboot_start: addr::PhysAddr,
    multiboot_end: addr::PhysAddr,
}

/// This is the Rust entry point that is called by the assembly boot code after switching to long mode.
#[no_mangle]
pub extern "C" fn kernel_main(args: &KernelArgs) -> ! {
    kernel::main(args)
}