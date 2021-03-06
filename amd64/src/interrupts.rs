use crate::cmos;
use crate::io;

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

/// Run a callback with interrupts disabled.
pub unsafe fn uninterruptible<F, R>(callback: F) -> R where F: FnOnce() -> R {
    disable();
    let value = callback();
    enable();
    value
}

/// Enable the non-maskable interrupt.
#[inline]
pub unsafe fn enable_nmi() {
    // NMI is controlled by the highest bit of the CMOS select register.
    io::outb(cmos::SELECT_PORT, io::inb(cmos::SELECT_PORT) & 0x7F);
}

/// Disable the non-maskable interrupt.
#[inline]
pub unsafe fn disable_nmi() {
    io::outb(cmos::SELECT_PORT, io::inb(cmos::SELECT_PORT) | 0x80);
}


/// Stack-frame layout upon entering an interrupt handler.
#[derive(Debug)]
#[repr(C)]
pub struct InterruptFrame {
    /// Saved instruction pointer, the interrupt handler jumps back to that location upon executing IRETQ.
    pub rip: usize,
    /// Saved code segment of the caller.
    pub cs: usize,
    /// Saved CPU flags.
    pub rflags: usize,
    /// Saved stack pointer from the calling stack frame.
    pub rsp: usize,
    /// Saved stack segement of the caller.
    pub ss: usize,
}

#[macro_export]
macro_rules! push_scratch_registers {
    () => {{
        asm!("push rax" : : : : "intel", "volatile");
        asm!("push rdi" : : : : "intel", "volatile");
        asm!("push rsi" : : : : "intel", "volatile");
        asm!("push rdx" : : : : "intel", "volatile");
        asm!("push rcx" : : : : "intel", "volatile");
        asm!("push r8"  : : : : "intel", "volatile");
        asm!("push r9"  : : : : "intel", "volatile");
        asm!("push r10" : : : : "intel", "volatile");
        asm!("push r11" : : : : "intel", "volatile");
    }};
}

#[macro_export]
macro_rules! pop_scratch_registers {
    () => {{
        asm!("pop r11" : : : : "intel", "volatile");
        asm!("pop r10" : : : : "intel", "volatile");
        asm!("pop r9"  : : : : "intel", "volatile");
        asm!("pop r8"  : : : : "intel", "volatile");
        asm!("pop rcx" : : : : "intel", "volatile");
        asm!("pop rdx" : : : : "intel", "volatile");
        asm!("pop rsi" : : : : "intel", "volatile");
        asm!("pop rdi" : : : : "intel", "volatile");
        asm!("pop rax" : : : : "intel", "volatile");
    }};
}

/// Generates a raw interrupt handler
#[macro_export]
macro_rules! interrupt_handler_raw {
    (fn $name:ident () $body:tt) => {
        #[naked]
        unsafe extern "C" fn $name() -> ! {
            // clear direction bit, will be restored by iretq
            asm!("cld");

            {
                $body
            }

            // This could be unreachable when the interrupt handler panics
            #[allow(unreachable_code)]
            {
                asm!("iretq" : : : : "intel", "volatile");
                unreachable!()
            }
        }
    };
}

#[macro_export]
macro_rules! interrupt_handler {
    (fn $name:ident ($frame:ident : $frame_type:ty) $body:tt) => {
        interrupt_handler_raw! {
            fn $name () {
                extern "C" fn work($frame: $frame_type) {
                    $body
                }
                assert_eq_size!($frame_type, usize);
                push_scratch_registers!();
                asm!("sub rsp, 8 // align to 16 bytes (we pushed 9 * 8)
                      lea rdi, [rsp+80]
                      call $0
                      add rsp, 8 // undo alignment
                     " : : "i"(work as extern "C" fn($frame_type)) : : "intel", "volatile");
                pop_scratch_registers!();
            }
        }
    };
}

#[macro_export]
macro_rules! exception_handler_with_code {
    (fn $name:ident ($frame:ident : $frame_type:ty, $err_code:ident : u64) $body:tt) => {
        interrupt_handler_raw! {
            fn $name () {
                extern "C" fn work($frame: $frame_type, $err_code : u64) {
                    $body
                }

                assert_eq_size!($frame_type, usize);
                push_scratch_registers!();
                asm!("lea rdi, [rsp+80]
                      mov rsi, [rsp+72]
                      call $0
                     " : : "i"(work as extern "C" fn($frame_type, u64)) : : "intel", "volatile");
                pop_scratch_registers!();
                // pop error code
                asm!("add rsp, 8" : : : : "intel", "volatile");
            }
        }
    };
}

#[macro_export]
macro_rules! interrupt_handler_wrapper {
    ($handler:ident) => {{
        interrupt_handler_raw! {
            fn wrapper() {
                asm!("call $0" : "=r"($handler as extern "C" fn()));
            }
        }
        wrapper
    }};
}
