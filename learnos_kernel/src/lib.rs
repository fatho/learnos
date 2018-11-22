#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), feature(global_asm))]
#![feature(naked_functions)]
#![feature(link_args)]
#![feature(asm)]
#![feature(get_type_id)]
#![feature(const_fn)]
//#![feature(alloc)]
#![feature(format_args_nl)] // needed for debug! macro
#![feature(extern_crate_item_prelude)]
#![feature(alloc_error_handler)]
#![feature(core_intrinsics)]
#![feature(abi_x86_interrupt)]

// built-in crates
#[macro_use]
extern crate core;
#[macro_use]
extern crate log;
extern crate spin;
//extern crate alloc;

// crates from crates.io
#[macro_use]
extern crate static_assertions;
//#[macro_use]
extern crate bitflags;

// other crates from this workspace
extern crate acpi;
extern crate bare_metal;
#[macro_use]
extern crate interrupts;
extern crate kmem;
extern crate multiboot2;

use core::cmp;
use core::iter;

use acpi::AcpiTable;
use bare_metal::*;
use bare_metal::segments::Ring;
use interrupts::idt::{IdtEntry, Idt};
use interrupts::apic::{Apic, LvtTimer, TimerDivisor};
use kmem::physical::alloc as kmem_alloc;
use kmem::physical::{PageFrameRegion, PageFrame};

#[macro_use]
pub mod diagnostics;
pub mod globals;
pub mod vga;
pub mod panic;
pub mod mem;
mod kernel;

use self::mem::layout::DIRECT_MAPPING;

/// Arguments passed to the kernel by the loader.
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct KernelArgs {
    kernel_start: PhysAddr,
    kernel_end: PhysAddr,
    multiboot_start: PhysAddr,
    multiboot_end: PhysAddr,
}

// For now, this kernel is 64 bit only. Ensure that `usize` has the right size.
assert_eq_size!(ptr_size; usize, u64);

/// The IDT that is used by the kernel on all cores.
static IDT: spin::Mutex<Idt> = spin::Mutex::new(Idt::new());

static LOGGER: &'static log::Log = &diagnostics::FanOutLogger
    (diagnostics::SerialLogger, diagnostics::VgaLogger);

static APIC: Apic = Apic::new(core::ptr::null_mut());

mod selectors {
    use bare_metal::segments::Selector;

    pub const KERNEL_CODE: Selector = Selector(8);
    #[allow(dead_code)]
    pub const KERNEL_DATA: Selector = Selector(16);
}

/// This is the Rust entry point that is called by the assembly boot code after switching to long mode.
#[no_mangle]
pub extern "C" fn kernel_main(args: &KernelArgs) -> ! {
    vga::init(DIRECT_MAPPING.phys_to_virt(vga::VGA_PHYS_ADDR));
    log::set_logger(LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();

    debug!("VGA initialized");

    // parse multiboot info
    let mb2: &multiboot2::Multiboot2Info = unsafe { &*DIRECT_MAPPING.phys_to_virt(args.multiboot_start).as_ptr() };
    diagnostics::print_multiboot(&mb2);

    // find memory map
    let memory_map = mb2.memory_map().expect("Bootloader did not provide memory map.");

    // compute start of physical heap
    let heap_start = mb2.modules().map(|m| m.mod_end())
        .chain(iter::once(args.kernel_end))
        .chain(iter::once(args.multiboot_end))
        .max().unwrap_or(PhysAddr(0));

    let heap_start_frame = PageFrame::next_above(heap_start);

    debug!("[Bootmem] first frame = {:p}", heap_start_frame.start_address());

    // Compute initial allocation regions: all available RAM regions, rounded down to page sizes,
    // and above the important kernel data.
    let bootmem_regions = memory_map.regions()
        .filter(|r| r.is_available())
        .map(|r| PageFrameRegion::new_included_in(r.base_addr(), r.base_addr() + r.length()))
        .map(|r| PageFrameRegion {
            start: cmp::max(r.start, heap_start_frame),
            end: r.end
        })
        .filter(|r| ! r.is_empty());

    // Initialize page frame allocator. It can only give us chunks of 4KB.
    // Fortunately, we mostly want to allocate page tables (which conveniently are 4KB in size)
    // and metadata for the better allocators (which can be reasonably rounded up to the next 4KB).
    let _boot_pfa = kmem_alloc::BumpAllocator::new(bootmem_regions);    
    debug!("[Bootmem] page frame allocator initialized");

    // Find the root ACPI table
    let rsdp = unsafe { find_acpi_rsdp().expect("ACPI not supported") };
    let rsdt = unsafe { acpi::table_from_raw::<acpi::Rsdt>(DIRECT_MAPPING.phys_to_virt(rsdp.rsdt_address())).expect("RSDT is corrupted") };
    
    let acpi_tables = rsdt.sdt_pointers()
        .map(|acpi_table_phys| DIRECT_MAPPING.phys_to_virt(acpi_table_phys))
        .map(|acpi_table_virt| unsafe { acpi::table_from_raw::<acpi::AnySdt>(acpi_table_virt).expect("Corrupted ACPI table") });
    for tbl in acpi_tables {
        unsafe {
            debug!("[ACPI] {}", core::str::from_utf8_unchecked(tbl.signature()));
        }
        if let Some(madt) = acpi::Madt::from_any(tbl) {
            for e in madt.entries() {
                debug!("  {:?}", e);
            }
        }
    }

    // TODO: setup proper address space

    // TODO: setup proper GDT

    // Setup interrupts
    unsafe {
        {
            let mut idt = IDT.lock();
            let intgate = |handler| IdtEntry::new(interrupts::idt::GateType::INTERRUPT_GATE, selectors::KERNEL_CODE, Some(handler), Ring::RING0, true);
            idt[0] = intgate(div_by_zero_handler);
            idt[8] = intgate(df_handler);
            idt[13] = intgate(gpf_handler);
            idt[14] = intgate(pf_handler);
            for i in 32..=255 {
                idt[i] = intgate(null_handler);
            }
            idt[32] = intgate(test_timer);
            idt[33] = intgate(callable_int);
            interrupts::idt::load_idt(&*idt);
            debug!("IDT loaded");
        }

        // default mapping of PIC collides with CPU exceptions
        interrupts::pic::remap(0x20, 0x28);
        debug!("PIC IRQs remapped");

        // we do not want to receive interrupts from the PIC, because
        // we are soon going to enable the APIC.
        interrupts::pic::set_masks(0xFF, 0xFF);
        debug!("PIC IRQs masked");

        if ! interrupts::apic::supported() {
            panic!("APIC not supported")
        }

        info!("BSP APIC ID {}", interrupts::apic::local_apic_id());

        if ! interrupts::apic::is_enabled()  {
            info!("APIC support not yet enabled, enabling now");
            interrupts::apic::set_enabled(true);
            assert!(interrupts::apic::is_enabled(), "APIC support could not be enabled");
        }

        let apic_base_phys = interrupts::apic::base_address();
        let apic_base_virt = DIRECT_MAPPING.phys_to_virt(apic_base_phys);
        APIC.set_base_address(apic_base_virt.as_mut_ptr());

        info!("APIC base address is {:p}", apic_base_phys);

        APIC.set_spurious_interrupt_vector(0xFF);
        APIC.set_software_enable(true);
        APIC.set_task_priority(0);
        
        info!("APIC enabled");

        // TODO: configure IO APIC


        interrupts::enable();

        // test timer
        APIC.set_timer_divisor(TimerDivisor::Divisor16);
        APIC.set_lvt_timer(LvtTimer::periodic(32));
        APIC.set_timer_initial_count(1024);

        asm!("int 33" : : : "rax" : "intel", "volatile");

        debug!("Timer: {:x?} {:x?} {:x?} {:x?}", APIC.timer_divisor(), APIC.timer_initial_count(), APIC.timer_current_count(), APIC.lvt_timer());
        debug!("APIC error status: {:x}", APIC.error_status());
    }

    let bla = 0x1_00000000 as *const u32;
    unsafe {
        let val = *bla;
        info!("IMPORTANT: {}", val);
    }

    unsafe {
        cpu::hang()
    }
}

unsafe fn find_acpi_rsdp() -> Option<&'static acpi::Rsdp> {
    let find_phys = |start_phys, end_phys| 
            acpi::Rsdp::find(DIRECT_MAPPING.phys_to_virt(PhysAddr(start_phys)),
                             DIRECT_MAPPING.phys_to_virt(PhysAddr(end_phys)));
    find_phys(0xE0000, 0xFFFFF).or(find_phys(0, 1024))
}

#[naked]
fn retpoline() {
    info!("retpolined");
    unsafe {
        let ret: u64;
        asm!("int 33" : "={rax}"(ret) : : : "intel");
        info!("int 33 returned {}", ret);
        cpu::hang();
    }
}


exception_handler_with_code! {
    fn df_handler(_frame: &interrupts::InterruptFrame, error_code: u64) {
        unsafe { APIC.signal_eoi(); }
        panic!("Double fault: {}", error_code);
    }
}

exception_handler_with_code! {
    fn pf_handler(stack_frame: &mut interrupts::InterruptFrame, error_code: u64) {
        let addr: usize;
        unsafe {
            asm!("mov $0, cr2" : "=r"(addr) : : : "intel");
        }
        error!("Page fault: {:05b} - {:p}\n{:X?}", error_code, VirtAddr(addr), stack_frame);
        stack_frame.rip = retpoline as usize;
        unsafe { APIC.signal_eoi(); }
    }
}

exception_handler_with_code! {
    fn gpf_handler(stack_frame: &interrupts::InterruptFrame, error_code: u64) {
        unsafe { APIC.signal_eoi(); }
        panic!("Protection fault: {:32b}\n{:X?}", error_code, stack_frame);
    }
}

interrupt_handler! {
    fn div_by_zero_handler(_frame: &interrupts::InterruptFrame) {
        unsafe { APIC.signal_eoi(); }
        panic!("division by zero");
    }
}

interrupt_handler! {
    fn test_timer(_frame: &interrupts::InterruptFrame) {
        info!("timer");
        unsafe { APIC.signal_eoi(); }
    }
}

interrupt_handler_raw! {
    fn null_handler() {
        APIC.signal_eoi();
    }
}

interrupt_handler_raw! {
    fn callable_int() {
        push_scratch_registers!();
        debug!("callable interrupt called");
        APIC.signal_eoi();
        pop_scratch_registers!();
        asm!("mov rax, 42" : : : : "intel");
    }
}