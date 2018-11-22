
#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))]
use core::fmt::{Write};
#[cfg(not(test))]
use crate::vga;
#[cfg(not(test))]
use amd64::cpu;
#[cfg(not(test))]
use crate::mem::layout;

#[cfg(not(test))]
#[alloc_error_handler]
fn foo(layout: core::alloc::Layout) -> ! {
    panic!("Failed to allocate {:?}", layout)
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
        let vga_addr = layout::DIRECT_MAPPING.phys_to_virt(vga::VGA_PHYS_ADDR);
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

    // Also dump the panic to the serial port.
    let mut com1 = unsafe { cpu::io::com::SerialPort::new(cpu::io::com::COM1_ADDR) };
    writeln!(com1, "{}", panic_info);

    unsafe {
        amd64::interrupts::disable();
        cpu::hang()
    }
}
