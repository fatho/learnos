use multiboot2;

macro_rules! debugln {
    ($($arg:tt)*) => {
        {
            let mut com1 = crate::globals::COM1.lock();
            core::fmt::Write::write_fmt(&mut *com1, format_args_nl!($($arg)*)).unwrap_or(());
        }
        {
            let mut writer = crate::vga::writer();
            core::fmt::Write::write_fmt(&mut writer, format_args_nl!($($arg)*)).unwrap_or(());
        }
    };
}

pub fn print_multiboot(mb2: &multiboot2::Multiboot2Info) {
    debugln!("MB2 info at {:p} size {}", mb2 as *const multiboot2::Multiboot2Info, mb2.size());

    for tag in mb2.modules() {
        debugln!("  Module: start={:?} end={:?} cmd_line", tag.mod_start(), tag.mod_end());
    }

    for mmap in mb2.memory_map() {
        debugln!("  Memory map:");
        debugln!("  {: ^6} {: ^23} {: ^18}", "Type", "Physical Address", "Length");
        let mut total_available = 0;
        for e in mmap.regions() {
            let type_ch = match e.entry_type() {
                multiboot2::memmap::EntryType::AVAILABLE => 'A',
                multiboot2::memmap::EntryType::AVAILABLE_ACPI => 'C',
                multiboot2::memmap::EntryType::RESERVED_HIBERNATION => 'H',
                multiboot2::memmap::EntryType::DEFECTIVE => 'X',
                _ => 'R',
            };
            debugln!("  {: ^6} {: ^23p} {:016x}", type_ch, e.base_addr(), e.length());
            if e.is_available() {
                total_available += e.length();
            }
        }
        debugln!("  Available: {} MiB", total_available / 1024 / 1024);
    }

    debugln!("  CmdLine: {:?}", mb2.boot_cmd_line());
    debugln!("  Bootloader: {:?}", mb2.bootloader_name());
}
