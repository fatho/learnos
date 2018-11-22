use core::iter;
use core::mem;
use core::ops;

use amd64::interrupts::apic::ApicId;
use amd64::interrupts::ioapic::IoApicId;

/// Architectural limit for the number of CPUs in a system.
pub const MAX_CPU_COUNT: usize = 256;

/// Architectural limit for the number of IO APICs in a system.
pub const MAX_IOAPIC_COUNT: usize = 256;


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