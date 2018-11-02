//! Implements a simple spin-lock based mutex.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};
use core::ops::{Deref, DerefMut};

pub struct Mutex<T> {
    guarded_value: UnsafeCell<T>,
    locked: AtomicBool,
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Mutex<T> {
        Mutex {
            guarded_value: UnsafeCell::new(value),
            locked: AtomicBool::new(false)
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        while self.locked.compare_and_swap(false, true, Ordering::Acquire) { }

        MutexGuard {
            mutex: self
        }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        if self.locked.compare_and_swap(false, true, Ordering::Acquire) {
            Some(MutexGuard {
                mutex: self
            })
        } else {
            None
        }
    }
}

unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.guarded_value.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.guarded_value.get() }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}