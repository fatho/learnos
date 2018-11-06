#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), feature(global_asm))]
#![feature(naked_functions)]
#![feature(link_args)]
#![feature(asm)]
#![feature(get_type_id)]
#![feature(const_fn)]
#![feature(alloc)]
#![feature(format_args_nl)] // needed for debugln! macro
#![feature(extern_crate_item_prelude)]
#![feature(alloc_error_handler)]

// built-in crates
#[macro_use]
extern crate core;
extern crate alloc;

// crates from crates.io
#[macro_use]
extern crate static_assertions;
#[macro_use]
extern crate bitflags;

// other crates from this workspace
extern crate bare_metal;
extern crate acpi;
extern crate multiboot2;
extern crate spinlock;

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
use bare_metal::*;
#[macro_use]
pub mod diagnostics;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga;

// kernel specific part
mod kernel;

/// Arguments passed to the kernel by the loader.
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct KernelArgs {
    kernel_start: PhysAddr,
    kernel_end: PhysAddr,
    multiboot_start: PhysAddr,
    multiboot_end: PhysAddr,
}

/// Must be initialized before it can actually allocate things.
/// Must only be initialized once, by the BSPs. All kernel threads run in the same address space.
#[global_allocator]
static KERNEL_ALLOCATOR: memory::heap::KernelAllocator = memory::heap::KernelAllocator::new();


// For now, this kernel is 64 bit only. Ensure that `usize` has the right size.
assert_eq_size!(ptr_size; usize, u64);

/// This is the Rust entry point that is called by the assembly boot code after switching to long mode.
#[no_mangle]
pub extern "C" fn kernel_main(args: &KernelArgs) -> ! {
    kernel::main(args)
}
