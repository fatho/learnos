//! Module responsible for displaying diagnostic messages on startup.

use super::console;
use super::layout;

use crate::multiboot2;

use core::fmt::{Write};

pub fn print_multiboot(console: &mut console::Console, mb2: &multiboot2::Multiboot2Info) {
    writeln!(console, "Multiboot info structures @ {:p}-{:p}", mb2.start_addr(), mb2.end_addr());

    for tag in mb2.tags() {
        match tag {
            multiboot2::Tag::MemoryMap(mmap) => {
                writeln!(console, "Memory map:");
                writeln!(console, "{: ^6} {: ^23} {: ^18}", "Type", "Physical Address", "Length");
                let mut total_available = 0;
                for e in mmap.entries() {
                    let type_ch = match e.entry_type {
                        multiboot2::memmap::EntryType::Available => 'A',
                        multiboot2::memmap::EntryType::AvailableACPI => 'C',
                        multiboot2::memmap::EntryType::ReservedHibernation => 'H',
                        multiboot2::memmap::EntryType::Defective => 'X',
                        multiboot2::memmap::EntryType::Reserved => 'R',
                    };
                    writeln!(console, "{: ^6} {: ^23p} {:016x}", type_ch, e.base_addr, e.length);
                    if e.is_available() {
                        total_available += e.length;
                    }
                }
                writeln!(console, " Available: {} MiB", total_available / 1024 / 1024);
            },
            multiboot2::Tag::BootCommandLine(cmdline) => {
                writeln!(console, "Command: {:?}", cmdline);
            },
            multiboot2::Tag::BootLoaderName(loader) => {
                writeln!(console, "Loader: {:?}", loader);
            }
            multiboot2::Tag::Other(id, _) => {
                writeln!(console, "Unknown tag: type={}", id);
            }
        }
    }
}

pub fn print_heap_info(console: &mut console::Console) {
    writeln!(console, "Physical heap starts at {:p}", layout::heap_start());
}
