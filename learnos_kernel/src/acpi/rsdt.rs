use crate::addr::{PhysAddr};
use core::mem;

use super::{AnySdt, SdtHeader, AcpiTable};
use super::util;

/// The Root System Descriptor Table. It contains 32 bit pointers to all other ACPI tables.
#[repr(C, packed)]
pub struct Rsdt {
    header: SdtHeader,
    sdt_pointers: [u32; 0]
}

impl AcpiTable for Rsdt {
    fn is_valid(&self) -> bool {
        let checksum_valid = unsafe { util::acpi_table_checksum(self) == 0 };
        let sig_valid = self.header.signature() == Self::SIGNATURE;
        checksum_valid && sig_valid
    }

    fn length(&self) -> usize {
        self.header.length()
    }

    fn from_any(any: &AnySdt) -> Option<&Self> {
        if any.signature() == Self::SIGNATURE {
            let this = unsafe { &*(any as *const AnySdt as *const Rsdt) };
            Some(this)
        } else {
            None
        }
    }
}

impl Rsdt {
    pub const SIGNATURE: &'static [u8; 4] = b"RSDT";

    /// Returns the number of tables that are referenced by this RSDT.
    pub fn num_entries(&self) -> usize {
        (self.length() - mem::size_of::<SdtHeader>()) / mem::size_of::<u32>()
    }

    /// Returns an iterator over all pointers stored in this table.
    pub fn sdt_pointers(&self) -> RsdtPointerIter {
        unsafe {
            let first = self.sdt_pointers.as_ptr();
            RsdtPointerIter {
                current: first,
                last: first.add(self.num_entries())
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RsdtPointerIter {
    current: *const u32,
    last: *const u32,
}

impl Iterator for RsdtPointerIter {
    type Item = PhysAddr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.last {
            assert!(self.current == self.last, "Entry sizes didn't add up");
            None
        } else {
            unsafe {
                let addr = *self.current;
                self.current = self.current.add(1);
                Some(PhysAddr(addr as usize))
            }
        }
    }
}
impl core::iter::FusedIterator for RsdtPointerIter {}