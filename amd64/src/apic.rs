use crate::util::Bits;
use crate::cpuid;
use crate::msr;
use crate::{Alignable, PhysAddr};

use core::sync::atomic::{AtomicPtr, Ordering};

/// The identifier of an APIC.
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct ApicId(pub u8);

pub fn supported() -> bool {
    let (_, _, _, edx) = cpuid::cpuid(1);
    edx & (1 << 9) != 0
}

pub fn local_apic_id() -> ApicId {
    let (_, ebx, _, _) = cpuid::cpuid(1);
    ApicId(((ebx >> 24) & 0xFF) as u8)
}


const APIC_MSR_ENABLED: u64 = 1 << 11;

/// Check whether the APIC is enabled.
pub fn is_enabled() -> bool {
    unsafe {
        let apic_msr = msr::APIC_BASE.read();
        (apic_msr & APIC_MSR_ENABLED) != 0
    }
}

/// Return the base address of the memory mapped APIC registers.
pub fn base_address() -> PhysAddr {
    unsafe {
        let apic_msr = msr::APIC_BASE.read();
        PhysAddr((apic_msr & 0xFFFFF000) as usize)
    }
}

/// Enable or disable the APIC. Usually, it is already enabled.
/// Warning: After disabling the APIC, it usually can only be enabled again after a system reset.
pub unsafe fn set_enabled(enabled: bool) {
    let mut apic_msr = msr::APIC_BASE.read();
    if enabled {
        apic_msr |= APIC_MSR_ENABLED
    } else {
        apic_msr &= ! APIC_MSR_ENABLED
    }
    msr::APIC_BASE.write(apic_msr)
}

/// Interface to the local APIC via the memory mapped registers.
pub struct ApicRegisters(AtomicPtr<u32>);


macro_rules! lvt_accessor {
    ($getter:ident, $setter:ident, $reg:expr, $entry:tt) => {
        #[inline(always)]
        pub unsafe fn $setter(&self, lvt: $entry) {
            self.write_reg($reg, lvt.0)
        }

        #[inline(always)]
        pub unsafe fn $getter(&self) -> $entry {
            $entry(self.read_reg($reg))
        }
    };
}

impl ApicRegisters {
    pub const SPURIOUS_INTERRUPT_VECTOR_REG: usize = 0xF0;
    pub const EOI_REG: usize = 0xB0;
    pub const LVT_TIMER_REG: usize = 0x320;
    pub const LVT_CMCI_REG: usize = 0x2F0;
    pub const LVT_LINT0_REG: usize = 0x350;
    pub const LVT_LINT1_REG: usize = 0x360;
    pub const LVT_ERROR_REG: usize = 0x370;
    pub const LVT_PERF_REG: usize = 0x340;
    pub const LVT_THERMAL_REG: usize = 0x330;
    pub const DIVISOR_CONFIG_REG: usize = 0x3E0;
    pub const INITIAL_COUNT_REG: usize = 0x380;
    pub const CURRENT_COUNT_REG: usize = 0x390;
    pub const ERROR_STATUS_REG: usize = 0x280;
    pub const TASK_PRIORITY_REG: usize = 0x80;

    #[inline(always)]
    pub const fn new(base_addr: *mut u32) -> ApicRegisters {
        ApicRegisters(AtomicPtr::new(base_addr))
    }

    #[inline(always)]
    pub unsafe fn set_base_address(&self, new_base: *mut u32) {
        self.0.store(new_base, Ordering::Release);
    }

    #[inline(always)]
    pub fn base_address_valid(&self) -> bool {
        (self.0.load(Ordering::Acquire) as usize).is_aligned(4096)
    }

    /// Software-enable or disable the local APIC.
    #[inline(always)]
    pub unsafe fn set_software_enable(&self, enabled: bool) {
        let mut value = self.read_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG);
        value.set_bit(8, enabled);
        self.write_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG, value)
    }

    /// Return the current software-enabled state of the APIC.
    #[inline(always)]
    pub unsafe fn software_enabled(&self) -> bool {
        self.read_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG).get_bit(8)
    }


    /// Set the interrupt vector where spurious interrupts are delivered to.
    #[inline(always)]
    pub unsafe fn set_spurious_interrupt_vector(&self, interrupt_vector: u8) {
        let mut value = self.read_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG);
        value.set_bits(0..=7, interrupt_vector as u32);
        self.write_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG, value);
    }

    /// Get the interrupt vector where spurious interrupts are delivered to.
    #[inline(always)]
    pub unsafe fn spurious_interrupt_vector(&self) -> u8 {
        self.read_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG).get_bits(0..=7) as u8
    }

    /// Signal the end of the current interrupt handler by writing to the EOI register.
    #[inline(always)]
    pub unsafe fn signal_eoi(&self) {
        self.write_reg(Self::EOI_REG, 0);
    }

    lvt_accessor!(lvt_lint0, set_lvt_lint0, Self::LVT_LINT0_REG, LvtLintEntry);
    lvt_accessor!(lvt_lint1, set_lvt_lint1, Self::LVT_LINT1_REG, LvtLintEntry);
    lvt_accessor!(lvt_timer, set_lvt_timer, Self::LVT_TIMER_REG, LvtTimerEntry);
    lvt_accessor!(lvt_perf, set_lvt_perf, Self::LVT_PERF_REG, LvtTimerEntry);
    lvt_accessor!(lvt_error, set_lvt_error, Self::LVT_ERROR_REG, LvtTimerEntry);
    lvt_accessor!(lvt_thermal, set_lvt_thermal, Self::LVT_THERMAL_REG, LvtTimerEntry);
    lvt_accessor!(lvt_cmci, set_lvt_cmci, Self::LVT_CMCI_REG, LvtTimerEntry);

    #[inline(always)]
    pub unsafe fn set_timer_divisor(&self, divisor: TimerDivisor) {
        let mut value = self.read_reg(Self::DIVISOR_CONFIG_REG);
        let divisor_bits = divisor as u32;
        value.set_bit(3, divisor_bits.get_bit(2));
        value.set_bits(0..=1, divisor_bits.get_bits(0..=1));
        self.write_reg(Self::DIVISOR_CONFIG_REG, value)
    }

    #[inline(always)]
    pub unsafe fn timer_divisor(&self) -> TimerDivisor {
        let divisor_config = self.read_reg(Self::DIVISOR_CONFIG_REG);
        let mut value = divisor_config.get_bits(0..=1);
        value.set_bit(2, divisor_config.get_bit(3));
        TimerDivisor::parse(value as u8).unwrap()
    }

    #[inline(always)]
    pub unsafe fn set_timer_initial_count(&self, count: u32) {
        self.write_reg(Self::INITIAL_COUNT_REG, count)
    }

    #[inline(always)]
    pub unsafe fn timer_initial_count(&self) -> u32 {
        self.read_reg(Self::INITIAL_COUNT_REG)
    }

    #[inline(always)]
    pub unsafe fn timer_current_count(&self) -> u32 {
        self.read_reg(Self::CURRENT_COUNT_REG)
    }

    // TODO: better wrapper around APIC error status
    pub unsafe fn error_status(&self) -> u32 {
        self.read_reg(Self::ERROR_STATUS_REG)
    }

    pub unsafe fn set_task_priority(&self, priority: u8) {
        let mut value = self.read_reg(Self::TASK_PRIORITY_REG);
        value.set_bits(0..=7, priority as u32);
        self.write_reg(Self::TASK_PRIORITY_REG, value);
    }

    /// Write to the given APIC register. The index must be 16 byte aligned, as mandated by the APIC specification.
    #[inline(always)]
    pub unsafe fn write_reg(&self, reg_index: usize, reg_value: u32) {
        assert!(reg_index.is_aligned(16), "misaligned APIC register index");
        let reg_addr = self.0.load(Ordering::Acquire).add(reg_index >> 2);
        reg_addr.write_volatile(reg_value);
    }

    /// Read the given APIC register. The index must be 16 byte aligned, as mandated by the APIC specification.
    #[inline(always)]
    pub unsafe fn read_reg(&self, reg_index: usize) -> u32 {
        assert!(reg_index.is_aligned(16), "misaligned APIC register index");
        let reg_addr = self.0.load(Ordering::Acquire).add(reg_index >> 2);
        reg_addr.read_volatile()
    }
}

/// The Delivery Mode is a 3 bit field that specifies how the
/// APICs listed in the destination field should act upon reception of this signal. Note that certain
/// Delivery Modes only operate as intended when used in conjunction with a specific trigger Mode.
/// These restrictions are indicated in the following table for each Delivery Mode.
#[repr(u8)]
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum DeliveryMode {
    /// Deliver  the  signal  on  the  INTR  signal  of  all  processor  cores  listed  in  the
    /// destination. Trigger Mode for "fixed" Delivery Mode can be edge or level.
    Fixed = 0b000,
    /// Deliver the signal on the INTR signal of the processor core that is
    /// executing at the lowest priority among all the processors listed in the
    /// specified destination. Trigger Mode for "lowest priority". Delivery Mode
    /// can be edge or level. This setting is only valid in the I/O APIC and reserved
    /// in the local vector table of an APIC.
    LowestPriority = 0b001,
    /// System Management Interrupt. A delivery mode equal to SMI requires an
    /// edge trigger mode. The vector information is ignored but must be
    /// programmed to all zeroes for future compatibility
    SMI = 0b010,
    /// Deliver the signal on the NMI signal of all processor cores listed in the
    /// destination. Vector information is ignored. NMI is treated as an edge
    /// triggered interrupt, even if it is programmed as a level triggered interrupt.
    /// For proper operation, this redirection table entry must be programmed to
    /// "edge" triggered interrupt.
    NMI = 0b100,
    /// Deliver the signal to all processor cores listed in the destination by
    /// asserting the INIT signal. All addressed local APICs will assume their
    /// INIT state. INIT is always treated as an edge triggered interrupt, even if
    /// programmed otherwise. For proper operation, this redirection table entry
    /// must be programmed to "edge" triggered interrupt.
    INIT = 0b101,
    /// Deliver the signal to the INTR signal of all processor cores listed in the
    /// destination as an interrupt that originated in an externally connected
    /// (8259A-compatible) interrupt controller. The INTA cycle that corresponds
    /// to this ExtINT delivery is routed to the external controller that is expected
    /// to supply the vector. A Delivery Mode of "ExtINT"  requires an edge trigger mode.
    ExtInit = 0b111
}

impl DeliveryMode {
    pub fn parse(value: u8) -> Option<DeliveryMode> {
        match value {
            0 => Some(DeliveryMode::Fixed),
            1 => Some(DeliveryMode::LowestPriority),
            2 => Some(DeliveryMode::SMI),
            4 => Some(DeliveryMode::NMI),
            5 => Some(DeliveryMode::INIT),
            7 => Some(DeliveryMode::ExtInit),
            _ => None
        }
    }
}

#[repr(u8)]
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Polarity {
    HighActive = 0,
    LowActive = 1
}

#[repr(u8)]
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum TriggerMode {
    EdgeTriggered = 0,
    LevelTriggered = 1
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum TimerDivisor {
    Divisor2 = 0b000,
    Divisor4 = 0b001,
    Divisor8 = 0b010,
    Divisor16 = 0b011,
    Divisor32 = 0b100,
    Divisor64 = 0b101,
    Divisor128 = 0b110,
    Divisor1 = 0b111,
}

impl TimerDivisor {
    pub fn parse(value: u8) -> Option<TimerDivisor> {
        match value {
            0b000 => Some(TimerDivisor::Divisor2),
            0b001 => Some(TimerDivisor::Divisor4),
            0b010 => Some(TimerDivisor::Divisor8),
            0b011 => Some(TimerDivisor::Divisor16),
            0b100 => Some(TimerDivisor::Divisor32),
            0b101 => Some(TimerDivisor::Divisor64),
            0b110 => Some(TimerDivisor::Divisor128),
            0b111 => Some(TimerDivisor::Divisor1),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[repr(u32)]
pub enum DeliveryStatus {
    Idle = 0,
    SendPending = 1
}

pub trait LvtEntry {
    unsafe fn new_unchecked(value: u32) -> Self;

    fn raw(&self) -> &u32;

    unsafe fn raw_mut(&mut self) -> &mut u32;

    fn disabled() -> Self where Self: Sized {
        unsafe { Self::new_unchecked(0x0001_0000) }
    }

    fn set_vector(&mut self, vec: u8) {
        unsafe { self.raw_mut().set_bits(0..=7, vec as u32); }
    }

    fn vector(&self) -> u8 {
        self.raw().get_bits(0..=7) as u8
    }

    fn set_masked(&mut self, masked: bool) {
        unsafe { self.raw_mut().set_bit(16, masked); }
    }

    fn masked(&self) -> bool {
        self.raw().get_bit(16)
    }

    fn delivery_status(&self) -> DeliveryStatus {
        if self.raw().get_bit(12) {
            DeliveryStatus::Idle
        } else {
            DeliveryStatus::SendPending
        }
    }
}

macro_rules! impl_LvtEntry {
    ($entry:tt) => {
        impl LvtEntry for $entry {
            #[inline(always)]
            unsafe fn new_unchecked(value: u32) -> Self { $entry(value) }
            #[inline(always)]
            fn raw(&self) -> &u32 { &self.0 }
            #[inline(always)]
            unsafe fn raw_mut(&mut self) -> &mut u32 { &mut self.0 }
        }
    };
}

pub trait DeliverableLvtEntry : LvtEntry {
    fn delivery_mode(&self) -> DeliveryMode {
        DeliveryMode::parse(self.raw().get_bits(8..=10) as u8).unwrap()
    }

    fn set_delivery_mode(&mut self, mode: DeliveryMode) {
        unsafe { self.raw_mut().set_bits(8..=10, mode as u32) }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct LvtTimerEntry(u32);

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[repr(u32)]
pub enum TimerMode {
    OneShot = 0,
    Periodic = 0b01,
    TscDeadline = 0b10,
}

impl TimerMode {
    pub fn parse(value: u32) -> Option<TimerMode> {
        match value {
            0 => Some(TimerMode::OneShot),
            0b01 => Some(TimerMode::Periodic),
            0b10 => Some(TimerMode::TscDeadline),
            _ => None,
        }
    }
}

impl LvtTimerEntry {
    pub fn one_shot(vector: u8) -> LvtTimerEntry {
        let mut lvt = Self::disabled();
        lvt.set_vector(vector);
        lvt.set_timer_mode(TimerMode::OneShot);
        lvt.set_masked(false);
        lvt
    }

    pub fn periodic(vector: u8) -> LvtTimerEntry {
        let mut lvt = Self::disabled();
        lvt.set_vector(vector);
        lvt.set_timer_mode(TimerMode::Periodic);
        lvt.set_masked(false);
        lvt
    }

    pub fn set_timer_mode(&mut self, mode: TimerMode) {
        self.0.set_bits(17..=18, mode as u32)
    }

    pub fn timer_mode(&self) -> TimerMode {
        TimerMode::parse(self.0.get_bits(17..=18)).unwrap()
    }
}
impl_LvtEntry!(LvtTimerEntry);

pub struct LvtLintEntry(u32);

impl LvtLintEntry {
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
}

impl_LvtEntry!(LvtLintEntry);
impl DeliverableLvtEntry for LvtLintEntry {}

pub struct LvtErrorEntry(u32);
impl_LvtEntry!(LvtErrorEntry);

pub struct LvtCmciEntry(u32);
impl_LvtEntry!(LvtCmciEntry);
impl DeliverableLvtEntry for LvtCmciEntry {}

pub struct LvtPerfEntry(u32);
impl_LvtEntry!(LvtPerfEntry);
impl DeliverableLvtEntry for LvtPerfEntry {}

pub struct LvtThermalEntry(u32);
impl_LvtEntry!(LvtThermalEntry);
impl DeliverableLvtEntry for LvtThermalEntry {}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lvt_timer() {
        let t = LvtTimerEntry::periodic(33);
        assert_eq!(t.0, 0b010_0000_0000_0010_0001);
    }
}