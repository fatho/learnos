use bare_metal::cpu;
use bare_metal::{Alignable, PhysAddr};

pub fn supported() -> bool {
    let (_, _, _, edx) = cpu::cpuid(1);
    edx & (1 << 9) != 0
}

pub fn local_apic_id() -> u8 {
    let (_, ebx, _, _) = cpu::cpuid(1);
    ((ebx >> 24) & 0xFF) as u8
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
pub struct Apic(*mut u32);

impl Apic {
    pub const SPURIOUS_INTERRUPT_VECTOR_REG: usize = 0xF0;
    pub const EOI_REG: usize = 0xB0;

    pub fn new(base_addr: *mut u32) -> Apic {
        assert!((base_addr as usize).is_aligned(4096), "APIC register base address not aligned");
        Apic(base_addr)
    }

    /// Software-enable or disable the local APIC.
    pub unsafe fn set_software_enable(&mut self, enabled: bool) {
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
    pub unsafe fn set_spurious_interrupt_vector(&mut self, interrupt_vector: u8) {
        let mut value = self.read_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG);
        value &= ! 0xFF;
        value |= interrupt_vector as u32;
        self.write_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG, value);
    }

    /// Get the interrupt vector where spurious interrupts are delivered to.
    pub unsafe fn spurious_interrupt_vector(&mut self) -> u8 {
        (self.read_reg(Self::SPURIOUS_INTERRUPT_VECTOR_REG) & 0xFF) as u8
    }


    /// Signal the end of the current interrupt handler by writing to the EOI register.
    pub unsafe fn signal_eoi(&mut self) {
        self.write_reg(Self::EOI_REG, 0);
    }


    /// Write to the given APIC register. The index must be 16 byte aligned, as mandated by the APIC specification.
    pub unsafe fn write_reg(&mut self, reg_index: usize, reg_value: u32) {
        assert!(reg_index.is_aligned(16), "misaligned APIC register index");
        let reg_addr = self.0.add(reg_index >> 4);
        reg_addr.write_volatile(reg_value);
    }

    /// Read the given APIC register. The index must be 16 byte aligned, as mandated by the APIC specification.
    pub unsafe fn read_reg(&self, reg_index: usize) -> u32 {
        assert!(reg_index.is_aligned(16), "misaligned APIC register index");
        let reg_addr = self.0.add(reg_index >> 2);
        debug!("Reading APIC register: {:x} at {:p}", reg_index, reg_addr);
        reg_addr.read_volatile()
    }
}