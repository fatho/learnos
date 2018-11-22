use amd64::{Alignable, VirtAddr, PhysAddr};
use super::util;
use super::{AcpiTable, AnySdt};

use core::str;
use core::slice;
use core::mem;

#[repr(C, packed)]
pub struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32
}

#[repr(C, packed)]
pub struct RsdpV2 {
    v1: Rsdp,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3]
}

impl Rsdp {
    pub const SIGNATURE: &'static [u8; 8] = b"RSD PTR ";

    /// Find the Root System Description Pointer table in the given virtual memory range.
    pub unsafe fn find(start: VirtAddr, end: VirtAddr) -> Option<&'static Rsdp> {
        // the signature is guaranteed to be 16 byte aligned
        let mut current = start.align_up(16);

        // little endian representation of the signature
        const SIG: u64 = 0x20_52_54_50_20_44_53_52;

        while current < end {
            let candidate = current.as_ptr::<u64>().read();
            if candidate == SIG {
                let rsdp = current.as_ptr::<Rsdp>();
                if (*rsdp).is_valid() {
                    return Some(&*rsdp)
                }
            }
            current += 16;
        }
        None
    }

    pub fn revision(&self) -> u8 {
        self.revision
    }

    pub fn oem_id(&self) -> &'static str {
        unsafe {
            str::from_utf8(slice::from_raw_parts(self.oem_id.as_ptr(), 6))
                .expect("Invalid UTF-8 string in ACPI OEM ID")
        }
    }

    pub fn rsdt_address(&self) -> PhysAddr {
        PhysAddr(self.rsdt_address as usize)
    }

    pub fn as_v2(&self) -> Option<&RsdpV2> {
        if self.revision() != 2 {
            unsafe {
                let v2 = &*(self as *const Rsdp as *const RsdpV2);
                if util::acpi_table_checksum(v2) == 0 {
                    return Some(v2);
                }
            }
        }
        None
    }
}

impl AcpiTable for Rsdp {
    fn is_valid(&self) -> bool {
        let sig_valid = &self.signature == Self::SIGNATURE;
        let checksum_valid = unsafe { util::acpi_table_checksum(self) == 0 };

        sig_valid && checksum_valid
    }

    fn length(&self) -> usize {
        mem::size_of::<Rsdp>()
    }

    fn from_any(any: &AnySdt) -> Option<&Self> {
        if any.signature() == Self::SIGNATURE {
            let this = unsafe { &*(any as *const AnySdt as *const Rsdp) };
            Some(this)
        } else {
            None
        }
    }
}

impl RsdpV2 {
    pub fn as_v1(&self) -> &Rsdp {
        &self.v1
    }

    pub fn length(&self) -> usize {
        self.length as usize
    }

    pub fn xsdt_address(&self) -> PhysAddr {
        PhysAddr(self.xsdt_address as usize)
    }
}

impl AcpiTable for RsdpV2 {
    fn is_valid(&self) -> bool {
        let v1_valid = self.as_v1().is_valid();
        let checksum_valid = unsafe { util::acpi_table_checksum(self) == 0 };

        v1_valid && checksum_valid
    }

    fn length(&self) -> usize {
        self.length as usize
    }

    fn from_any(any: &AnySdt) -> Option<&Self> {
        Rsdp::from_any(any).and_then(|r| r.as_v2())
    }
}