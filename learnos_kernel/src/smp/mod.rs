use core::iter;
use core::mem;
use core::ops;

use amd64::PhysAddr;
use amd64::interrupts::apic::{ApicId, Polarity, TriggerMode};
use amd64::interrupts::ioapic::IoApicId;

/// Architectural limit for the number of CPUs in a system.
pub const MAX_CPU_COUNT: usize = 256;

/// Architectural limit for the number of IO APICs in a system.
pub const MAX_IOAPIC_COUNT: usize = 256;

/// Maximum number of ISA IRQs.
pub const MAX_ISA_IRQ_COUNT: usize = 32;

/// Stores information about a CPU.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct CpuInfo {
    pub acpi_id: u8,
    pub apic_id: ApicId,
    pub is_bsp: bool,
}

/// Stores information about an IOAPIC.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct IoApicInfo {
    pub id: IoApicId,
    /// The physical address to access this I/O APIC. Each I/O APIC resides at a unique address.
    pub addr: PhysAddr,
    /// The global system interrupt number where this I/O APIC’s interrupt
    /// inputs start. The number of interrupt inputs is determined by the I/O
    /// APIC’s Max Redir Entry register.
    pub irq_base: u32,
    /// Maximum number of input inerrupts of this I/O APIC.
    pub max_redir_count: u32,
    /// The I/O APIC version
    pub version: u32,
}

/// Sotres information about an IRQ.
pub struct IrqInfo {
    /// The global system interrupt that this IRQ is mapped to. This determines the I/O APIC which receives
    /// the interrupt, and the index of the redirection table entry.
    pub global_system_interrupt: u32,
    /// Polarity of the interrupt. This is needed for correctly setting up the I/O APIC entry for this IRQ.
    pub polarity: Polarity,
    /// Trigger mode of the interrupt. This is needed for correctly setting up the I/O APIC entry for this IRQ.
    pub trigger_mode: TriggerMode,
}

macro_rules! info_table {
    ($name:ident, $entry_type:ty, $entry_count:expr, { $($extra_fn:tt)* }) => {
        /// A table for keeping track of all (at most 256) CPUs in the system.
        pub struct $name {
            entries: [mem::ManuallyDrop<$entry_type>; $entry_count],
            count: usize,
        }

        impl $name {
            pub fn new() -> $name {
                $name {
                    entries: unsafe { mem::uninitialized() },
                    count: 0
                }
            }

            /// Return the number of entries.
            pub fn count(&self) -> usize {
                self.count
            }

            /// Insert an entry into the table and return its internal ID.
            ///
            /// # Panics
            ///
            /// Panics when trying to insert more than 256 entries.
            pub fn insert(&mut self, entry: $entry_type) -> usize {
                assert!(self.count < $entry_count, "too many entries");
                let index = self.count;
                self.count += 1;
                self.entries[index] = mem::ManuallyDrop::new(entry);
                index
            }

            pub fn iter(&self) -> impl Iterator<Item=&$entry_type> {
                self.entries[0..self.count].iter().map(|c| &**c)
            }

            $($extra_fn)*
        }

        impl iter::FromIterator<$entry_type> for $name {
            fn from_iter<T: IntoIterator<Item = $entry_type>>(iter: T) -> Self {
                let mut table = $name::new();
                for entry in iter.into_iter().take(256) {
                    table.insert(entry);
                }
                table
            }
        }

        impl iter::Extend<$entry_type> for $name {
            fn extend<T: IntoIterator<Item = $entry_type>>(&mut self, iter: T) {
                for entry in iter.into_iter().take(256) {
                    self.insert(entry);
                }
            }
        }


        impl ops::Index<u8> for $name {
            type Output = $entry_type;

            fn index(&self, idx: u8) -> &$entry_type {
                let uidx = idx as usize;
                assert!(uidx < self.count(), "index out of range");
                &*self.entries[uidx]
            }
        }

        impl ops::IndexMut<u8> for $name {
            fn index_mut(&mut self, idx: u8) -> &mut $entry_type {
                let uidx = idx as usize;
                assert!(uidx < self.count(), "tindex out of range");
                &mut *self.entries[uidx]
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                while self.count > 0 {
                    self.count -= 1;
                    unsafe {
                        mem::ManuallyDrop::drop(&mut self.entries[self.count])
                    }
                }
            }
        }
    };
}

info_table!(IoApicTable, IoApicInfo, MAX_IOAPIC_COUNT, {
    pub fn by_id(&self, ioapic_id: IoApicId) -> Option<&IoApicInfo> {
        self.iter().find(|ioapic| ioapic.id == ioapic_id)
    }

    pub fn by_gsi(&self, gsi: u32) -> Option<&IoApicInfo> {
        self.iter().find(|ioapic| gsi >= ioapic.irq_base && gsi < ioapic.irq_base + ioapic.max_redir_count)
    }
});

info_table!(CpuTable, CpuInfo, MAX_CPU_COUNT, {
    pub fn by_apic_id(&self, apic_id: ApicId) -> Option<&CpuInfo> {
        self.iter().find(|cpu| cpu.apic_id == apic_id)
    }

    pub fn by_acpi_id(&self, acpi_id: u8) -> Option<&CpuInfo> {
        self.iter().find(|cpu| cpu.acpi_id == acpi_id)
    }

    pub fn bsp(&self) -> Option<&CpuInfo> {
        self.iter().find(|cpu| cpu.is_bsp)
    }

    pub fn aps(&self) -> impl Iterator<Item=&CpuInfo> {
        self.iter().filter(|cpu| ! cpu.is_bsp)
    }
});

pub struct IsaIrqTable([IrqInfo; MAX_ISA_IRQ_COUNT]);

impl IsaIrqTable {
    pub fn new() -> IsaIrqTable {
        let mut table = unsafe { IsaIrqTable(mem::uninitialized()) };
        // setup the default identity mapping
        for irq in 0..MAX_ISA_IRQ_COUNT {
            let entry = IrqInfo {
                global_system_interrupt: irq as u32,
                polarity: Polarity::HighActive,
                trigger_mode: TriggerMode::EdgeTriggered,
            };
            table.0[irq] = entry;
        }
        table
    }

    pub fn iter(&self) -> impl Iterator<Item=&IrqInfo> {
        self.0.iter()
    }
}

impl ops::Index<usize> for IsaIrqTable {
    type Output = IrqInfo;

    fn index(&self, idx: usize) -> &IrqInfo {
        &self.0[idx]
    }
}

impl ops::IndexMut<usize> for IsaIrqTable {
    fn index_mut(&mut self, idx: usize) -> &mut IrqInfo {
        &mut self.0[idx]
    }
}
