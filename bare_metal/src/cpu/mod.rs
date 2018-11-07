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