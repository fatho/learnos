//! Parts that are specific to this kernel and cannot be easily reused.

mod layout;
mod diagnostics;

#[cfg(not(test))]
use core::panic::PanicInfo;
use core::fmt::{Write};

use crate::vga;
use crate::multiboot2;
use crate::memory::bump::BumpAllocator;

/// 
pub fn main(args: &super::KernelArgs) -> ! {
    // Initialize VGA buffer. Besides panics, this is the only place where this should happen.
    vga::init(layout::low_phys_to_virt(vga::VGA_PHYS_ADDR));

    writeln!(vga::writer(), "{:?}", args);

    // parse multiboot info
    let mb2: &multiboot2::Multiboot2Info = unsafe { &*layout::low_phys_to_virt(args.multiboot_start).as_ptr() };
    diagnostics::print_multiboot(&mb2);

    // find memory map
    let memory_map = mb2.memory_map().expect("Bootloader did not provide memory map.");

    // use it for building the page frame allocator
    let mut pfa = unsafe { BumpAllocator::new(memory_map.regions()) };
    // reserve everything up to the highest used address
    pfa.reserve_until_address(args.kernel_end);
    pfa.reserve_until_address(args.multiboot_end);
    for module in mb2.modules() {
        pfa.reserve_until_address(module.mod_start());
    }

    let total = pfa.total_available_frames();
    let remaining = pfa.remaining_frames();
    writeln!(vga::writer(), "Total: {} frames  Remaining: {} frames ({}%)", total, remaining, remaining * 100 / total);
    
    halt!();
}

#[panic_handler]
#[cfg(not(test))]
fn panic(panic_info: &PanicInfo) -> ! {
    fn write_panic(writer: &mut vga::Writer, panic_info: &PanicInfo) {
        writeln!(writer, "{}", panic_info);
    }

    fn extreme_panic(panic_info: &PanicInfo) {
        // Extreme panic is for when the VGA system is currently locked or
        // has never been initialized. System is FUBAR anyway, just grab a
        // new instance of VGA buffer and hope we get some info out
        let vga_addr = layout::low_phys_to_virt(vga::VGA_PHYS_ADDR);
        let vgabuf = unsafe { vga::VgaMem::from_addr(vga_addr) };
        let mut temp_console = vga::Writer::with_colors(vgabuf, vga::Color::White, vga::Color::Red);
        write_panic(&mut temp_console, panic_info);
    }

    // try to grab the global VGA writer first, so that the panic doesn't erase previously logged info.
    // That info could be very valuable for debugging.
    match vga::GLOBAL_WRITER.try_lock() {
        None => extreme_panic(panic_info),
        Some(mut optwriter) => match *optwriter {
            None => extreme_panic(panic_info),
            Some(ref mut writer) => write_panic(writer, panic_info)
        }
    };

    halt!();
}