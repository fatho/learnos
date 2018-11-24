pub const APIC_BASE: Msr = Msr(0x1B);

/// A model-specific register.
pub struct Msr(pub u32);

impl Msr {
    /// Read the value of a model specific register
    #[inline(always)]
    pub unsafe fn read(&self) -> u64 {
        let lo: u32;
        let hi: u32;
        asm!("rdmsr" : "={eax}"(lo), "={edx}"(hi) : "{ecx}"(self.0));
        (lo as u64) | ((hi as u64) << 32)
    }

    /// Write the value of a model specific register
    #[inline(always)]
    pub unsafe fn write(&self, val: u64) {
        let lo = (val & 0xFFFFFFFF) as u32;
        let hi = ((val >> 32) & 0xFFFFFFFF) as u32;
        asm!("wrmsr" : : "{ecx}"(self.0), "{eax}"(lo), "{edx}"(hi));
    }
}