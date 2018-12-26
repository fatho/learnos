use multiboot2;
use core::fmt::Write;
use log;

pub struct SerialLogger;

impl log::Log for SerialLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut com1 = crate::globals::COM1.lock();
            let lvl_char = level_prefix(record.level());
            writeln!(com1, "[{}] {}", lvl_char, record.args()).unwrap_or(());
        }
    }

    fn flush(&self) {}
}

pub struct VgaLogger;

impl log::Log for VgaLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut vga_out = crate::vga::writer();
            let lvl_char = level_prefix(record.level());
            writeln!(vga_out, "[{}] {}", lvl_char, record.args()).unwrap_or(());
        }
    }

    fn flush(&self) {}
}

pub struct FanOutLogger<A, B>(pub A, pub B);

impl<A: log::Log, B: log::Log> log::Log for FanOutLogger<A, B> {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.0.enabled(metadata) || self.1.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        self.0.log(record);
        self.1.log(record);
    }

    fn flush(&self) {
        self.0.flush();
        self.1.flush();
    }
}

fn level_prefix(level: log::Level) -> char {
    match level {
        log::Level::Trace => 'T',
        log::Level::Debug => 'D',
        log::Level::Info => 'I',
        log::Level::Warn => 'W',
        log::Level::Error => 'E',
    }
}

pub fn print_multiboot(mb2: &multiboot2::Multiboot2Info) {
    info!("MB2 info at {:p} size {}", mb2 as *const multiboot2::Multiboot2Info, mb2.size());

    for tag in mb2.modules() {
        info!("  Module: start={:p} end={:p} cmd={:?}", tag.mod_start(), tag.mod_end(), tag.cmd_line());
    }

    for mmap in mb2.memory_map() {
        info!("  Memory map:");
        info!("  {: ^6} {: ^23} {: ^18}", "Type", "Physical Address", "Length");
        let mut total_available = 0;
        for e in mmap.regions() {
            let type_ch = match e.entry_type() {
                multiboot2::memmap::EntryType::AVAILABLE => 'A',
                multiboot2::memmap::EntryType::AVAILABLE_ACPI => 'C',
                multiboot2::memmap::EntryType::RESERVED_HIBERNATION => 'H',
                multiboot2::memmap::EntryType::DEFECTIVE => 'X',
                _ => 'R',
            };
            info!("  {: ^6} {: ^23p} {:016x}", type_ch, e.base_addr(), e.length());
            if e.is_available() {
                total_available += e.length();
            }
        }
        info!("  Available: {} MiB", total_available / 1024 / 1024);
    }

    info!("  CmdLine: {:?}", mb2.boot_cmd_line());
    for tok in crate::kaal::cmdline::CmdLine::parse(mb2.boot_cmd_line().unwrap_or("")) {
        debug!("  {:?}", tok)
    }
    info!("  Bootloader: {:?}", mb2.bootloader_name());
}
