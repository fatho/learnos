//! KaaL - Kernel as a Library

pub mod cmdline;

pub struct Kernel {

}

impl Kernel {
    /// Initialize the kernel on the bootstrap processor (BSP).
    /// 
    /// This must be called first, before any AP initializations are run.
    pub fn init_bsp() {
        // TODO: initialize logging

        // TODO: read multiboot

        // TODO: initialize memory allocation
    }


    // TODO: parse ACPI

    // TODO: initialize interrupts

    // TODO: calibrate timers
}
