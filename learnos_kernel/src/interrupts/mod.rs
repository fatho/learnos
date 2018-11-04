pub mod idt;
pub mod pic;

/// Enable interrupts on the current CPU.
#[inline]
pub unsafe fn enable() {
    asm!("sti" : : : : "intel", "volatile")
}

/// Disable interrupts on the current CPU.
#[inline]
pub unsafe fn disable() {
    asm!("cli" : : : : "intel", "volatile")
}