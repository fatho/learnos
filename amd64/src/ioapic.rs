use crate::util::Bits;
use crate::apic::{DeliveryStatus, DeliveryMode, TriggerMode, Polarity};

/// The identifier of an IOAPIC.
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct IoApicId(pub u8);

pub struct IoApicRegisters(*mut u32);

impl IoApicRegisters {
    pub const ID_REG: u32 = 0;
    pub const VER_REG: u32 = 1;
    pub const ARB_REG: u32 = 2;
    pub const REDIRECTION_ENTRY_REG_BASE: u32 = 0x10;

    pub fn new(base: *mut u32) -> IoApicRegisters {
        IoApicRegisters(base)
    }

    pub unsafe fn id(&self) -> IoApicId {
        IoApicId(self.read_reg(Self::ID_REG).get_bits(24..=27) as u8)
    }

    pub unsafe fn version(&self) -> u32 {
        self.read_reg(Self::VER_REG).get_bits(0..=7)
    }

    /// The maximum number of IRQ redirection enties in this IOAPIC.
    pub unsafe fn max_redirection_entries(&self) -> u32 {
        self.read_reg(Self::VER_REG).get_bits(16..=23)
    }

    pub unsafe fn arbitration_priority(&self) -> u32 {
        self.read_reg(Self::ARB_REG).get_bits(24..=27)
    }

    pub unsafe fn redirection_entry(&self, index: u32) -> RedirectionEntry {
        let reg = Self::REDIRECTION_ENTRY_REG_BASE + index * 2;
        let lo = self.read_reg(reg) as u64;
        let hi = self.read_reg(reg + 1) as u64;
        RedirectionEntry((hi << 32) | lo)
    }

    pub unsafe fn set_redirection_entry(&mut self, index: u32, entry: RedirectionEntry) {
        let reg = Self::REDIRECTION_ENTRY_REG_BASE + index * 2;
        let lo = (entry.0 & 0xFFFF_FFFF) as u32;
        let hi = (entry.0 >> 32) as u32;
        self.write_reg(reg, lo);
        self.write_reg(reg, hi);
    }

    #[inline(always)]
    pub unsafe fn write_reg(&mut self, register_index: u32, value: u32) {
        self.address().write_volatile(register_index);
        self.data().write_volatile(value);
    }

    #[inline(always)]
    pub unsafe fn read_reg(&self, register_index: u32) -> u32 {
        self.address().write_volatile(register_index);
        self.data().read_volatile()
    }

    #[inline(always)]
    fn address(&self) -> *mut u32 {
        self.0
    }

    #[inline(always)]
    unsafe fn data(&self) -> *mut u32 {
        self.0.add(4)
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct RedirectionEntry(u64);

/// This value determines the interpretation of the destination field in a redirect entry.
/// When the destination mode is "physical", a destination APIC is identified by its ID.
/// Bits 56 through 59 of the Destination field specify the 4 bit APIC ID. When the destination mode
/// is "logical", destinations are identified by matching on the logical destination under the control of the
/// Destination Format Register and Logical Destination Register in each Local APIC.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum DestinationMode {
    Physical = 0,
    Logical = 1
}

impl RedirectionEntry {
    /// The vector field is an 8 bit field containing the interrupt
    /// vector for this interrupt. Vector values range from 10h to FEh
    pub fn vector(&self) -> u8 {
        self.0.get_bits(0..=7) as u8
    }

    pub fn set_vector(&mut self, vector: u8) {
        self.0.set_bits(0..=7, vector as u64)
    }

    pub fn delivery_mode(&self) -> DeliveryMode {
        DeliveryMode::parse(self.0.get_bits(8..=10) as u8).unwrap()
    }

    pub fn set_delivery_mode(&mut self, mode: DeliveryMode) {
        self.0.set_bits(8..=10, mode as u64)
    }

    pub fn destination_mode(&self) -> DestinationMode {
        if self.0.get_bit(11) { DestinationMode::Logical } else { DestinationMode::Physical }
    }

    pub fn set_destination_mode(&mut self, mode: DestinationMode) {
        self.0.set_bit(11, mode == DestinationMode::Logical);
    }

    pub fn delivery_status(&self) -> DeliveryStatus {
        if self.0.get_bit(12) { DeliveryStatus::Idle } else { DeliveryStatus::SendPending }
    }

    pub fn input_polarity(&self) -> Polarity {
        if self.0.get_bit(13) { Polarity::LowActive } else { Polarity::HighActive }
    }

    pub fn set_input_polarity(&mut self, mode: Polarity) {
        self.0.set_bit(13, mode == Polarity::LowActive);
    }

    /// TODO: implement getter for Remote IRR

    pub fn trigger_mode(&self) -> TriggerMode {
        if self.0.get_bit(15) { TriggerMode::LevelTriggered } else { TriggerMode::EdgeTriggered }
    }

    pub fn set_trigger_mode(&mut self, mode: TriggerMode) {
        self.0.set_bit(15, mode == TriggerMode::LevelTriggered);
    }

    pub fn masked(&self) -> bool {
        self.0.get_bit(16)
    }

    pub fn set_masked(&mut self, masked: bool) {
        self.0.set_bit(16, masked);
    }

    pub fn destination(&self) -> u8 {
        self.0.get_bits(56..=63) as u8
    }

    pub fn set_destination(&mut self, dest: u8) {
        self.0.set_bits(56..=63, dest as u64)
    }

}