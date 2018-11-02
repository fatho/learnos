//! Module responsible for displaying diagnostic messages on startup.

use super::vga;

use crate::multiboot2;

use core::fmt::{Write};

pub fn print_multiboot(mb2: &multiboot2::Multiboot2Info) {
    writeln!(vga::writer(), "MB2 info at {:p} length {}", mb2.start_addr(), mb2.length());

    for tag in mb2.tags() {
        match tag {
            multiboot2::Tag::MemoryMap(mmap) => {
                writeln!(vga::writer(), "Memory map:");
                writeln!(vga::writer(), "{: ^6} {: ^23} {: ^18}", "Type", "Physical Address", "Length");
                let mut total_available = 0;
                for e in mmap.regions() {
                    let type_ch = match e.entry_type() {
                        multiboot2::memmap::EntryType::AVAILABLE => 'A',
                        multiboot2::memmap::EntryType::AVAILABLE_ACPI => 'C',
                        multiboot2::memmap::EntryType::RESERVED_HIBERNATION => 'H',
                        multiboot2::memmap::EntryType::DEFECTIVE => 'X',
                        _ => 'R',
                    };
                    writeln!(vga::writer(), "{: ^6} {: ^23p} {:016x}", type_ch, e.base_addr(), e.length());
                    if e.is_available() {
                        total_available += e.length();
                    }
                }
                writeln!(vga::writer(), " Available: {} MiB", total_available / 1024 / 1024);
            },
            multiboot2::Tag::BootCommandLine(cmdline) => {
                writeln!(vga::writer(), "Command: {:?}", cmdline);
            },
            multiboot2::Tag::BootLoaderName(loader) => {
                writeln!(vga::writer(), "Loader: {:?}", loader);
            }
            multiboot2::Tag::Other(id, _) => {
                writeln!(vga::writer(), "Unknown tag: type={}", id);
            }
        }
    }
}
