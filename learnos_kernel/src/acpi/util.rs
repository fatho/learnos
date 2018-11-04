use super::AcpiTable;

use core::slice;

/// Compute the ACPI checksum of the given table based on the self-reported length.
pub unsafe fn acpi_table_checksum<T: AcpiTable + ?Sized>(table: &T) -> u8 {
    acpi_table_checksum_with_length(table, table.length())
}

/// Compute the ACPI checksum of the given table with an explicitly given length.
pub unsafe fn acpi_table_checksum_with_length<T: ?Sized>(table: &T, length: usize) -> u8 {
    let data = slice::from_raw_parts(table as *const T as *const u8, length);
    acpi_checksum(data)
}


/// Compute the ACPI checksum over the given slice of ACPI data.
/// 
/// The checksum is calculated by adding all bytes (wrapping around on overflow).
/// The result must be zero for the checksum to be valid.
pub fn acpi_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0_u8, |acc, b| acc.wrapping_add(*b))
}