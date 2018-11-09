//! Implements low-level utility function that should already exist, but don't.

pub unsafe fn memset(mem: *mut u8, size: usize, value: u8) {
    for i in 0..size {
        mem.add(i).write(value)
    }
}
