#![cfg_attr(not(test), no_std)]

pub mod util;
mod rsdp;
mod rsdt;
mod xsdt;
mod madt;

pub use self::rsdp::*;
pub use self::rsdt::*;
pub use self::xsdt::*;
pub use self::madt::*;

use bare_metal::{VirtAddr};

pub trait AcpiTable {
    fn is_valid(&self) -> bool;
    fn length(&self) -> usize;
    fn from_any(any: &AnySdt) -> Option<&Self>;
}

/// Header of an ACPI system description table.
#[repr(C, packed)]
pub struct SdtHeader {
  signature: [u8; 4],
  length: u32,
  revision: u8,
  checksum: u8,
  oem_id: [u8; 6],
  oem_table_id: [u8; 8],
  oem_revision: u32,
  creator_id: u32,
  creator_revision: u32,
}

impl SdtHeader {
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    pub fn length(&self) -> usize {
        self.length as usize
    }
}

/// A generic ACPI table that provides only access to the header that is common to all ACPI tables.
pub struct AnySdt {
    header: SdtHeader,
}

impl AcpiTable for AnySdt {
    fn is_valid(&self) -> bool {
        unsafe { util::acpi_table_checksum(self) == 0 }
    }

    fn length(&self) -> usize {
        self.header.length()
    }

    fn from_any(any: &AnySdt) -> Option<&Self> {
        Some(any)
    }
}

impl AnySdt {
    pub fn signature(&self) -> &[u8] {
        self.header.signature()
    }
}

/// Acquire a reference to an ACPI table from a raw virtual address.
/// This function ensures that the memory area pointed to contains a valid ACPI table of the requested type.
pub unsafe fn table_from_raw<T: AcpiTable>(table_addr: VirtAddr) -> Option<&'static T> {
    let table: *const T = table_addr.as_ptr();
    if (*table).is_valid() {
        Some(&*table)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
