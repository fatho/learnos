//! Parts that are specific to this kernel and cannot be easily reused.

mod layout;
mod diagnostics;

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))]
use core::fmt::{Write};
use core::iter;
use core::str;
use core::slice;

use crate::addr::{PhysAddr, VirtAddr};
use crate::vga;
use crate::multiboot2;
use crate::memory;
use crate::acpi;
use crate::acpi::AcpiTable;
use crate::spin;
use crate::interrupts;

static IDT: spin::Mutex<interrupts::idt::Idt> = spin::Mutex::new(interrupts::idt::Idt::new());

/// 
pub fn main(args: &super::KernelArgs) -> ! {
    // Initialize VGA buffer. Besides panics, this is the only place where this should happen.
    vga::init(layout::low_phys_to_virt(vga::VGA_PHYS_ADDR));

    debugln!("VGA initialized");

    // parse multiboot info
    let mb2: &multiboot2::Multiboot2Info = unsafe { &*layout::low_phys_to_virt(args.multiboot_start).as_ptr() };
    diagnostics::print_multiboot(&mb2);

    // find memory map
    let memory_map = mb2.memory_map().expect("Bootloader did not provide memory map.");

    // compute start of physical heap
    let heap_start = mb2.modules().map(|m| m.mod_end())
        .chain(iter::once(args.kernel_end))
        .chain(iter::once(args.multiboot_end))
        .max().unwrap_or(PhysAddr(0));

    // initialize pysical memory subsystem
    memory::pfa::init(heap_start, memory_map.regions());
    debugln!("Page frame allocation initialized.");

    // find ACPI table
    let start_search = layout::KERNEL_VIRTUAL_BASE.add(0x000E0000);
    let end_search = layout::KERNEL_VIRTUAL_BASE.add(0x000FFFFF);
    let rsdp = unsafe { acpi::Rsdp::find(start_search, end_search) };
    if let Some(rsdp) = rsdp {
        debugln!("ACPI revision {} found. OEM is {}", rsdp.revision(), rsdp.oem_id());
        debugln!("RSDT at {:p}", rsdp.rsdt_address());
        let rsdt: &acpi::Rsdt = unsafe { acpi::table_from_raw(layout::low_phys_to_virt(rsdp.rsdt_address())).expect("Invalid RSDT") };
        for sdt_ptr in rsdt.sdt_pointers() {
            let maybe_sdt: Option<&acpi::AnySdt> = unsafe { acpi::table_from_raw(layout::low_phys_to_virt(sdt_ptr)) };
            if let Some(sdt) = maybe_sdt {
                let sig_str = str::from_utf8(sdt.signature()).unwrap_or("XXXX");
                debugln!("  - {:p} {} {}", sdt_ptr, sig_str, sdt.length());

                if let Some(madt) = acpi::Madt::from_any(sdt) {
                    debugln!("    Local APIC: {:p}", madt.local_apic_address());
                    for r in madt.entries() {
                        debugln!("    - {:?}", r);
                    }
                }
            } else {
                debugln!("  - {:p} INVALID", sdt_ptr);
            }
        }
    } else {
        debugln!("ACPI not found");
    }

    // virtual memory test
    debugln!("Starting virtual memory test");
    // map VGA buffer again at 
    let vga_new = VirtAddr(0x0000_0010_0000_0000);
    unsafe { 
        debugln!("Map VGA buffer to different address");
        memory::vmm::mmap(vga_new, vga::VGA_PHYS_ADDR);
        let old_buffer: &[u16] = slice::from_raw_parts(layout::low_phys_to_virt(vga::VGA_PHYS_ADDR).as_ptr(), 25 * 80);
        let new_buffer: &[u16] = slice::from_raw_parts(vga_new.as_ptr(), 25 * 80);
        debugln!("Ensuring old and new buffer have the same contents");
        assert_eq!(old_buffer, new_buffer);
        debugln!("Seems like both virtual addresses point to the same physical memory");
    }


    unsafe {
        {
            let idt = IDT.lock();
            interrupts::idt::load_idt(&*idt);
        }

        // default mapping of PIC collides with CPU exceptions
        interrupts::pic::remap(0x20, 0x28);

        interrupts::pic::set_masks(0xFF, 0xFF);

        interrupts::enable();
    }

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

    // Also dump the panic to the serial port.
    let mut com1 = unsafe { crate::serial::SerialPort::new(crate::serial::COM1_ADDR) };
    writeln!(com1, "{}", panic_info);

    halt!();
}
