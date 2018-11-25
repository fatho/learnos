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
