use crate::cmos;
use crate::util::{Bits};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ClockTime {
    pub format: HourFormat,
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub day_of_month: u8,
    pub month: u8,
    pub year: u32,
}

/// Distinguishes between 12/24 hour clock.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum HourFormat {
    Hour12,
    Hour24,
}

/// Wait for the next update of the RTC to happen.
/// It is only safe to read the RTC after an update,
/// otherwise it is likely that the data is inconsistent.
/// 
/// Accesses CMOS registers, therefore care must be taken
/// that no concurrent CMOS accesses happen.
pub unsafe fn wait_for_update() {
    // wait for update to start
    while ! cmos::read_register(registers::STATUS_A).get_bit(7) {}
    // wait for update to finish
    while cmos::read_register(registers::STATUS_A).get_bit(7) {}
}

/// Read the current time from the CMOS RTC.
/// 
/// Accesses CMOS registers, therefore care must be taken
/// that no concurrent CMOS accesses happen.
pub unsafe fn read_clock() -> ClockTime {
    let has_century = false;
    let (hour_format, value_format) = read_rtc_format();
    let raw = read_rtc_raw(has_century);
    interpret_raw_data(raw, hour_format, value_format)
}

/// Distinguishes the two encoding possiblities for the RTC.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum ValueFormat {
    BCD,
    Binary
}

/// Raw data read from the RTC.
pub struct RawData {
    seconds: u8,
    minutes: u8,
    hours: u8,
    day_of_month: u8,
    month: u8,
    century: u8,
    year: u8,
}

fn interpret_raw_data(raw: RawData, hours: HourFormat, format: ValueFormat) -> ClockTime {
    let century = if raw.century == 0 { 20 } else { decode_field(format, raw.century) } as u32;
    ClockTime {
        format: hours,
        seconds: decode_field(format, raw.seconds),
        minutes: decode_field(format, raw.minutes),
        hours: decode_field(format, raw.hours),
        day_of_month: decode_field(format, raw.day_of_month),
        month: decode_field(format, raw.month),
        year: century * 100 + decode_field(format, raw.year) as u32,
    }
}

fn decode_field(format: ValueFormat, value: u8) -> u8 {
    match format {
        ValueFormat::BCD => decode_bcd(value),
        ValueFormat::Binary => value,
    }
}

fn decode_bcd(bcd: u8) -> u8 {
    let low = bcd.get_bits(0..=3);
    let high = bcd.get_bits(4..=7);
    low + high * 10
}

unsafe fn read_rtc_format() -> (HourFormat, ValueFormat) {
    let status = cmos::read_register(registers::STATUS_B);
    let hour_format = if status.get_bit(1) { HourFormat::Hour24 } else { HourFormat::Hour12 };
    let value_format = if status.get_bit(2) { ValueFormat::Binary } else { ValueFormat::BCD };
    (hour_format, value_format)
}

unsafe fn read_rtc_raw(has_century: bool) -> RawData {
    RawData {
        seconds: cmos::read_register(registers::SECONDS),
        minutes: cmos::read_register(registers::MINUTES),
        hours: cmos::read_register(registers::HOURS),
        day_of_month: cmos::read_register(registers::DAY_OF_MONTH),
        month: cmos::read_register(registers::MONTH),
        century: if has_century { cmos::read_register(registers::CENTURY) } else { 0 },
        year: cmos::read_register(registers::YEAR),
    }
}

/// The RTC related CMOS registers
mod registers {
    use crate::cmos::CmosRegister;

    pub static SECONDS: CmosRegister = CmosRegister(0x00);
    pub static MINUTES: CmosRegister = CmosRegister(0x02);
    pub static HOURS: CmosRegister = CmosRegister(0x04);
    pub static DAY_OF_MONTH: CmosRegister = CmosRegister(0x07);
    pub static MONTH: CmosRegister = CmosRegister(0x08);
    pub static YEAR: CmosRegister = CmosRegister(0x09);
    /// The century register might not exist, check ACPI FADT
    pub static CENTURY: CmosRegister = CmosRegister(0x32);
    pub static STATUS_A: CmosRegister = CmosRegister(0x0A);
    pub static STATUS_B: CmosRegister = CmosRegister(0x0B);

    // According to the OSDev wiki, the weekday register is unreliable
    // pub static WEEKDAY: CmosRegister = CmosRegister(0x06);
}