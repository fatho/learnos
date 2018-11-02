//! Parts that are specific to this kernel and cannot be easily reused.

mod layout;
mod diagnostics;

#[cfg(not(test))]
use core::panic::PanicInfo;
use core::fmt::{Write};
use core::ops::DerefMut;

use crate::addr::{PhysAddr};
use crate::vga;
use crate::multiboot2;
use crate::memory;

/// 
pub fn main(args: &super::KernelArgs) -> ! {
    // Initialize VGA buffer. Besides panics, this is the only place where this should happen.
    vga::init(layout::low_phys_to_virt(vga::VGA_PHYS_ADDR));

    writeln!(vga::writer(), "{:?}", args);

    // prepare multiboot info parsing
    let mb2 = unsafe { multiboot2::Multiboot2Info::from_virt(layout::low_phys_to_virt(args.multiboot_info)) };

    diagnostics::print_multiboot(&mb2);

    // only consider addresses above the kernel as free
    // below the kernel lies the bootcode (which we could recover at this point)
    // and the stack and page tables (which we cannot recover yet)
    let heap_start = args.kernel_end.align_up(12);
    
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
        Some(mut optwriter) => match optwriter.deref_mut() {
            None => extreme_panic(panic_info),
            Some(ref mut writer) => write_panic(writer, panic_info)
        }
    };

    halt!();
}