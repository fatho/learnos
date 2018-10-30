#![cfg_attr(not(test), no_std)]
//#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), feature(global_asm))]
#![feature(naked_functions)]
#![feature(link_args)]

#[cfg(not(test))]
use core::panic::PanicInfo;

#[repr(C, packed)]
struct VgaBufferEntry {
    vga_char: u8,
    vga_color: u8,
}

const VGA_BUFFER_ADDR: u64 = 0xB8000;

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn rust_main() -> ! {
    unsafe {
        let vga_buffer_ptr = VGA_BUFFER_ADDR as *mut VgaBufferEntry;
        let mut vga_buffer = core::slice::from_raw_parts_mut(vga_buffer_ptr, 2000);
        vga_buffer[0] = VgaBufferEntry {
            vga_color: 0x0F,
            vga_char: 'T' as u8
        }
    }
    loop {}
}

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
