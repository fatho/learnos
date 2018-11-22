use crate::cpu;
use crate::{Alignable, PhysAddr};

use core::sync::atomic::{AtomicPtr, Ordering};

/// The identifier of an APIC.
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct ApicId(pub u8);

pub fn supported() -> bool {
    let (_, _, _, edx) = cpu::cpuid(1);
    edx & (1 << 9) != 0
}

pub fn local_apic_id() -> ApicId {
    let (_, ebx, _, _) = cpu::cpuid(1);
    ApicId(((ebx >> 24) & 0xFF) as u8)
}


const APIC_MSR_ENABLED: u64 = 1 << 11;

/// Check whether the APIC is enabled.
pub fn is_enabled() -> bool {
    unsafe {
        let apic_msr = cpu::read_msr(cpu::MSR_APIC_BASE);
        (apic_msr & APIC_MSR_ENABLED) != 0
    }
}

/// Return the base address of the memory mapped APIC registers.
pub fn base_address() -> PhysAddr {
    unsafe {
        let apic_msr = cpu::read_msr(cpu::MSR_APIC_BASE);
        PhysAddr((apic_msr & 0xFFFFF000) as usize)
    }
}

/// Enable or disable the APIC. Usually, it is already enabled.
/// Warning: After disabling the APIC, it usually can only be enabled again after a system reset.
pub unsafe fn set_enabled(enabled: bool) {
    let mut apic_msr = cpu::read_msr(cpu::MSR_APIC_BASE);
    if enabled {
        apic_msr |= APIC_MSR_ENABLED
    } else {
        apic_msr &= ! APIC_MSR_ENABLED
    }
    cpu::write_msr(cpu::MSR_APIC_BASE, apic_msr)
}

/// Interface to the local APIC via the memory mapped registers.
pub struct Apic(AtomicPtr<u32>);

impl Apic {
    pub const SPURIOUS_INTERRUPT_VECTOR_REG: usize = 0xF0;
    pub const EOI_REG: usize = 0xB0;
    pub const LVT_TIMER_REG: usize = 0x320;
    pub const DIVISOR_CONFIG_REG: usize = 0x3E0;
    pub const INITIAL_COUNT_REG: usize = 0x380;
    pub const CURRENT_COUNT_REG: usize = 0x390;
    pub const ERROR_STATUS_REG: usize = 0x280;
    pub const TASK_PRIORITY_REG: usize = 0x80;

    pub const fn new(base_addr: *mut u32) -> Apic {
        Apic(AtomicPtr::new(base_addr))
    }

    pub unsafe fn set_base_address(&self, new_base: *mut u32) {
        self.0.store(new_base, Ordering::Release);
    }

    pub fn base_address_valid(&self) -> bool {
        (self.0.load(Ordering::Acquire) as usize).is_aligned(4096)
    }

    /// Software-enable or disable the local APIC.
    pub unsafe fn set_software_enable(&self, enabled: bool) {
        let mut value = self.read_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG);
         if enabled {
            value |= 0x100;
        } else {
            value &= ! 0x100;
        }
        self.write_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG, value)
    }

    /// Return the current software-enabled state of the APIC.
    pub unsafe fn software_enabled(&self) -> bool {
        self.read_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG) & 0x100 != 0
    }


    /// Set the interrupt vector where spurious interrupts are delivered to.
    pub unsafe fn set_spurious_interrupt_vector(&self, interrupt_vector: u8) {
        let mut value = self.read_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG);
        value &= ! 0xFF;
        value |= interrupt_vector as u32;
        self.write_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG, value);
    }

    /// Get the interrupt vector where spurious interrupts are delivered to.
    pub unsafe fn spurious_interrupt_vector(&self) -> u8 {
        (self.read_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG) & 0xFF) as u8
    }


    /// Signal the end of the current interrupt handler by writing to the EOI register.
    #[inline(always)]
    pub unsafe fn signal_eoi(&self) {
        self.write_reg(Self::EOI_REG, 0);
    }

    /// Set the timer entry of the local vector table.
    #[inline(always)]
    pub unsafe fn set_lvt_timer(&self, lvt: LvtTimer) {
        self.write_reg(Self::LVT_TIMER_REG, lvt.0)
    }

    /// Read the timer entry of the local vector table.
    pub unsafe fn lvt_timer(&self) -> LvtTimer {
        LvtTimer(self.read_reg(Self::LVT_TIMER_REG))
    }

    #[inline(always)]
    pub unsafe fn set_timer_divisor(&self, divisor: TimerDivisor) {
        let divisor_hi = ((divisor as u32) & 0b100) << 1;
        let divisor_lo = (divisor as u32) & 0b011;
        let divisor_old = self.read_reg(Self::DIVISOR_CONFIG_REG);
        let divisor_new = (divisor_old & !0b1111) | divisor_lo | divisor_hi;
        self.write_reg(Self::DIVISOR_CONFIG_REG, divisor_new)
    }

    pub unsafe fn timer_divisor(&self) -> TimerDivisor {
        let divisor_config = self.read_reg(Self::DIVISOR_CONFIG_REG);
        let divisor_hi = (divisor_config & 0b1000) >> 1;
        let divisor_lo = divisor_config & 0b011;
        TimerDivisor::parse((divisor_hi | divisor_lo) as u8).unwrap()
    }

    #[inline(always)]
    pub unsafe fn set_timer_initial_count(&self, count: u32) {
        self.write_reg(Self::INITIAL_COUNT_REG, count)
    }

    pub unsafe fn timer_initial_count(&self) -> u32 {
        self.read_reg(Self::INITIAL_COUNT_REG)
    }

    pub unsafe fn timer_current_count(&self) -> u32 {
        self.read_reg(Self::CURRENT_COUNT_REG)
    }

    // TODO: better wrapper around APIC error status
    pub unsafe fn error_status(&self) -> u32 {
        self.read_reg(Self::ERROR_STATUS_REG)
    }

    pub unsafe fn set_task_priority(&self, priority: u8) {
        let old = self.read_reg(Self::TASK_PRIORITY_REG);
        let new = (old & ! 0xFF) | (priority as u32);
        self.write_reg(Self::TASK_PRIORITY_REG, new);
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
pub struct LvtTimer(u32);

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

impl LvtTimer {
    pub fn disabled() -> LvtTimer {
        LvtTimer(0x0001_0000)
    }

    pub fn one_shot(vector: u8) -> LvtTimer {
        let mut lvt = Self::disabled();
        lvt.set_vector(vector);
        lvt.set_timer_mode(TimerMode::OneShot);
        lvt.set_masked(false);
        lvt
    }

    pub fn periodic(vector: u8) -> LvtTimer {
        let mut lvt = Self::disabled();
        lvt.set_vector(vector);
        lvt.set_timer_mode(TimerMode::Periodic);
        lvt.set_masked(false);
        lvt
    }

    pub fn set_vector(&mut self, vec: u8) {
        self.0 &= ! 0xFF;
        self.0 |= vec as u32;
    }

    pub fn vector(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn set_masked(&mut self, masked: bool) {
        if masked {
            self.0 |= 1 << 16;
        } else {
            self.0 &= !(1 << 16);
        }
    }

    pub fn masked(&self) -> bool {
        self.0 & (1 << 16) != 0
    }

    pub fn set_timer_mode(&mut self, mode: TimerMode) {
        self.0 = (self.0 & !(0b11 << 17)) | ((mode as u32) << 17);
    }

    pub fn timer_mode(&self) -> TimerMode {
        TimerMode::parse((self.0 & (0b11 << 17)) >> 17).unwrap()
    }

    // TODO: implement delivery status getter
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lvt_timer() {
        let t = LvtTimer::periodic(33);
        assert_eq!(t.0, 0b010_0000_0000_0010_0001);
    }
}