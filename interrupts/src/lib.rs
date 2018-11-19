#![feature(asm)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate static_assertions;
#[macro_use]
extern crate log;
extern crate bare_metal;

pub mod idt;
pub mod pic;
pub mod apic;

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

#[macro_export]
macro_rules! push_scratch_registers {
    () => {{
        asm!("push rax" : : : : "intel");
        asm!("push rdi" : : : : "intel");
        asm!("push rsi" : : : : "intel");
        asm!("push rdx" : : : : "intel");
        asm!("push rcx" : : : : "intel");
        asm!("push r8"  : : : : "intel");
        asm!("push r9"  : : : : "intel");
        asm!("push r10" : : : : "intel");
        asm!("push r11" : : : : "intel");
    }};
}

#[macro_export]
macro_rules! pop_scratch_registers {
    () => {{
        asm!("push rax" : : : : "intel");
        asm!("push rdi" : : : : "intel");
        asm!("push rsi" : : : : "intel");
        asm!("push rdx" : : : : "intel");
        asm!("push rcx" : : : : "intel");
        asm!("push r8"  : : : : "intel");
        asm!("push r9"  : : : : "intel");
        asm!("push r10" : : : : "intel");
        asm!("push r11" : : : : "intel");
    }};
}

#[macro_export]
macro_rules! interrupt_handler_raw {
    (fn $name:ident () $body:tt) => {
        #[naked]
        unsafe extern "C" fn $name() -> ! {
            // stack frame is 16 byte aligned when interrupt handler is called by CPU
            push_scratch_registers!();
            // after pushing 9 registers, align stack to 16 bytes again (stack grows downwards)
            asm!("sub rsp, 8" : : : : "intel");
            {
                $body
            }
            // TODO: sigal EOI to APIC

            // This could be unreachable when the interrupt handler panics
            #[allow(unreachable_code)]
            {
                asm!("add rsp, 8" : : : : "intel"); // undo alignment
                pop_scratch_registers!();

                asm!("iretq" : : : : "intel", "volatile");
                unreachable!()
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

// TODO: write macro for interrupts with error codes