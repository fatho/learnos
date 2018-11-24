use amd64::{PhysAddr};
use amd64::interrupts::apic::ApicId;
use amd64::interrupts::ioapic::IoApicId;

use super::{AnySdt, SdtHeader, AcpiTable};
use super::util;

/// The Multiple APIC Description Table.
#[repr(C, packed)]
pub struct Madt {
    header: SdtHeader,
    local_apic_address: u32,
    flags: u32,
    records: [MadtEntryHeader; 0]
}

impl AcpiTable for Madt {
    fn is_valid(&self) -> bool {
        unsafe { util::acpi_table_checksum(self) == 0 }
    }

    fn length(&self) -> usize {
        self.header.length()
    }

    fn from_any(any: &AnySdt) -> Option<&Self> {
        if any.signature() == Self::SIGNATURE {
            let this = unsafe { &*(any as *const AnySdt as *const Madt) };
            Some(this)
        } else {
            None
        }
    }
}

impl Madt {
    pub const SIGNATURE: &'static [u8; 4] = b"APIC";

    /// Returns the physical address at which the local APIC is mapped.
    /// If a local APIC address override is specified, that address is returned,
    /// otherwise, the 32 bit address from the header is returned.
    pub fn local_apic_address(&self) -> PhysAddr {
        let default_addr = PhysAddr(self.local_apic_address as usize);
        self.iter()
            .find_map(|r| r.local_apic_address_override())
            .map_or(default_addr, |r| r.local_apic_address())
    }

    /// Returns an iterator over the headers of all entries in this MADT.
    pub fn entry_headers(&self) -> MadtHeaderIter {
        unsafe {
            let first = self.records.as_ptr();
            let last = ((self as *const Madt) as *const u8).add(self.length()) as *const MadtEntryHeader;
            MadtHeaderIter {
                current: first,
                last: last
            }
        }
    }

    /// Iterate over all MADT entries.
    pub fn iter(&self) -> impl Iterator<Item=MadtEntry> {
        self.entry_headers().map(MadtEntry::from_header)
    }

    /// Returns an iterator over all local APICS.
    pub fn processor_local_apics(&self) -> impl Iterator<Item=&ProcessorLocalApic> {
        self.iter()
            .filter_map(|f| f.processor_local_apic())
    }

    /// Returns an iterator over all IO apics.
    pub fn io_apics(&self) -> impl Iterator<Item=&IoApic> {
        self.iter()
            .filter_map(|f| f.io_apic())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MadtHeaderIter {
    current: *const MadtEntryHeader,
    last: *const MadtEntryHeader,
}

impl Iterator for MadtHeaderIter {
    type Item = &'static MadtEntryHeader;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.last {
            assert!(self.current == self.last, "Entry sizes didn't add up");
            None
        } else {
            unsafe {
                let header = &*self.current;
                let offset = header.record_length as usize;
                self.current = (self.current as *const u8).add(offset) as *const MadtEntryHeader;
                Some(header)
            }
        }
    }
}
impl core::iter::FusedIterator for MadtHeaderIter {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MadtEntry {
    ProcessorLocalApic(&'static ProcessorLocalApic),
    InterruptSourceOverride(&'static InterruptSourceOverride),
    IoApic(&'static IoApic),
    LocalApicAddressOverride(&'static LocalApicAddressOverride),
    NonMaskableInterrupt(&'static NonMaskableInterrupt),
    Unknown(&'static MadtEntryHeader),
}

impl MadtEntry {
    pub fn from_header(header: &'static MadtEntryHeader) -> MadtEntry {
        unsafe {
            match header.entry_type() {
                ProcessorLocalApic::ENTRY_TYPE => MadtEntry::ProcessorLocalApic(header.cast()),
                IoApic::ENTRY_TYPE => MadtEntry::IoApic(header.cast()),
                LocalApicAddressOverride::ENTRY_TYPE => MadtEntry::LocalApicAddressOverride(header.cast()),
                InterruptSourceOverride::ENTRY_TYPE => MadtEntry::InterruptSourceOverride(header.cast()),
                NonMaskableInterrupt::ENTRY_TYPE => MadtEntry::NonMaskableInterrupt(header.cast()),
                _ => MadtEntry::Unknown(header),
            }
        }
    }

    pub fn processor_local_apic(&self) -> Option<&'static ProcessorLocalApic> {
        match self {
            MadtEntry::ProcessorLocalApic(this) => Some(this),
            _ => None
        }
    }

    pub fn local_apic_address_override(&self) -> Option<&'static LocalApicAddressOverride> {
        match self {
            MadtEntry::LocalApicAddressOverride(this) => Some(this),
            _ => None
        }
    }

    pub fn io_apic(&self) -> Option<&'static IoApic> {
        match self {
            MadtEntry::IoApic(this) => Some(this),
            _ => None
        }
    }

    pub fn interrupt_source_override(&self) -> Option<&'static InterruptSourceOverride> {
        match self {
            MadtEntry::InterruptSourceOverride(this) => Some(this),
            _ => None
        }
    }

    pub fn non_maskable_interrupt(&self) -> Option<&'static NonMaskableInterrupt> {
        match self {
            MadtEntry::NonMaskableInterrupt(this) => Some(this),
            _ => None
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C, packed)]
pub struct MadtEntryHeader {
    entry_type: u8,
    record_length: u8,
}

impl MadtEntryHeader {
    pub fn entry_type(&self) -> u8 {
        self.entry_type
    }

    pub unsafe fn checked_cast<T>(&self, expected_type: u8) -> Option<&T> {
        if self.entry_type() == expected_type {
            Some(self.cast())
        } else {
            None
        }
    }

    pub unsafe fn cast<T>(&self) -> &T {
        &*(self as *const MadtEntryHeader as *const T)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C, packed)]
pub struct ProcessorLocalApic {
    record_header: MadtEntryHeader,
    processor_id: u8,
    apic_id: u8,
    /// bit 1 = processor enabled
    flags: u32,
}

impl ProcessorLocalApic {
    pub const ENTRY_TYPE: u8 = 0;

    /// Return the ACPI processor ID of the CPU that this APIC belongs to.
    #[inline(always)]
    pub fn processor_id(&self) -> u8 {
        self.processor_id
    }

    /// Return the id of this APIC.
    pub fn apic_id(&self) -> ApicId {
        ApicId(self.apic_id)
    }

    /// Check whether the CPU belonging to this APIC is enabled.
    pub fn processor_enabled(&self) -> bool {
        self.flags & 1 != 0
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C, packed)]
pub struct IoApic {
    record_header: MadtEntryHeader,
    io_apic_id: u8,
    reserved: u8,
    io_apic_address: u32,
    global_system_interrupt_base: u32,
}

impl IoApic {
    pub const ENTRY_TYPE: u8 = 1;

    /// The I/O APIC’s ID.
    pub fn id(&self) -> IoApicId {
        IoApicId(self.io_apic_id)
    }

    /// The 32-bit physical address to access this I/O APIC. Each I/O APIC resides at a unique address.
    pub fn address(&self) -> PhysAddr {
        PhysAddr(self.io_apic_address as usize)
    }

    /// The global system interrupt number where this I/O APIC’s interrupt
    /// inputs start. The number of interrupt inputs is determined by the I/O
    /// APIC’s Max Redir Entry register.
    pub fn global_system_interrupt_base(&self) -> u32 {
        self.global_system_interrupt_base
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C, packed)]
pub struct InterruptSourceOverride {
    record_header: MadtEntryHeader,
    bus_source: u8,
    irq_source: u8,
    global_system_interrupt: u32,
    /// Consists of
    ///   - polarity (0-2), valid values are `00` (conforms to bus), `01` (active high), `11` (active low)
    ///   - trigger mode (2-4), valid values are `00` (conforms to bus), `01` (edge-triggered), `11` (level-triggered)
    ///   - reserved (4-16)
    flags: u16,
}

impl InterruptSourceOverride {
    pub const ENTRY_TYPE: u8 = 2;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C, packed)]
pub struct LocalApicAddressOverride {
    record_header: MadtEntryHeader,
    reserved: u16,
    local_apic_address: u64,
}

impl LocalApicAddressOverride {
    pub const ENTRY_TYPE: u8 = 5;

    pub fn local_apic_address(&self) -> PhysAddr {
        PhysAddr(self.local_apic_address as usize)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C, packed)]
pub struct NonMaskableInterrupt {
    record_header: MadtEntryHeader,
    /// ACPI Processor ID (0xFF means all processors)
    processor_id: u8,
    /// Same flags as for InterruptSourceOverride.
    flags: u16,
    /// Local APIC interrupt input `LINTn` to which NMI is connected
    lint: u8
}

impl NonMaskableInterrupt {
    pub const ENTRY_TYPE: u8 = 4;

}