pub mod io;


/// Pause the CPU until the next interrupt arrives.
#[inline]
pub unsafe fn hlt() {
    asm!("hlt");
}

/// Pause the CPU indefintely. Interrupts may still arrive,
/// depending on the interrupt flags of the CPU.
#[inline]
pub unsafe fn hang() -> ! {
    loop {
        hlt();
    }
}

/// Execute the cpuid instruction after setting eax to the given query.
#[inline]
pub fn cpuid(eax: u32) -> (u32, u32, u32, u32) {
    let a: u32;
    let b: u32;
    let c: u32;
    let d: u32;
    unsafe {
        asm!("cpuid" : "={eax}"(a), "={ebx}"(b), "={ecx}"(c), "={edx}"(d) : "{eax}"(eax));
    }
    (a, b, c, d)
}

pub const MSR_APIC_BASE: u32 = 0x1B;

/// Read the value of a model specific register
#[inline]
pub unsafe fn read_msr(msr: u32) -> u64 {
    let lo: u32;
    let hi: u32;
    asm!("rdmsr" : "={eax}"(lo), "={edx}"(hi) : "{ecx}"(msr));
    (lo as u64) | ((hi as u64) << 32)
}

/// Write the value of a model specific register
#[inline]
pub unsafe fn write_msr(msr: u32, val: u64) {
    let lo = (val & 0xFFFFFFFF) as u32;
    let hi = ((val >> 32) & 0xFFFFFFFF) as u32;
    asm!("wrmsr" : : "{ecx}"(msr), "{eax}"(lo), "{edx}"(hi));
}