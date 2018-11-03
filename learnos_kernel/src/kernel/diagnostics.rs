//! Module responsible for displaying diagnostic messages on startup.

use super::vga;

use crate::multiboot2;

use core::fmt::{Write};

pub fn print_multiboot(mb2: &multiboot2::Multiboot2Info) {
    writeln!(vga::writer(), "MB2 info at {:p} length {}", mb2 as *const multiboot2::Multiboot2Info, mb2.length());

    for tag in mb2.tags() {
        writeln!(vga::writer(), "Multiboot tag: type={:?} size={}", tag.tag_type(), tag.size());
    }

    for tag in mb2.modules() {
        writeln!(vga::writer(), "Module: start={:?} end={:?} cmd_line", tag.mod_start(), tag.mod_end());
    }

    for mmap in mb2.memory_map() {
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
    }

    writeln!(vga::writer(), "CmdLine: {:?}", mb2.boot_cmd_line());
    writeln!(vga::writer(), "Bootloader: {:?}", mb2.bootloader_name());
}
