//! Implements a simple spin-lock based mutex.

#![cfg_attr(not(test), no_std)]

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
        loop {
            if let Some(success) = self.try_lock() {
                return success;
            }
        }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        if self.locked.compare_and_swap(false, true, Ordering::Acquire) {
            None
        } else {
            Some(MutexGuard {
                mutex: self
            })
        }
    }

    pub fn with_lock<F, R>(&self, callback: F) -> R where F: FnOnce(&T) -> R {
        let guard = self.lock();
        callback(&*guard)
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

#[cfg(test)]
mod test {
    use super::Mutex;

    #[test]
    fn test_mutex() {
        let mutex = Mutex::new(0_u32);

        // can always lock in the beginning
        {
            let guard = mutex.try_lock();
            assert!(guard.is_some(), "Unlocked mutex must be lockable");
        }

        // Mutex guard should release it due to the ending scope above
        {
            let guard = mutex.try_lock();
            assert!(guard.is_some(), "Mutex should have been unlocked by guard");

            let guard2 = mutex.try_lock();
            assert!(guard2.is_none(), "Mutex acquired twice");
        }
    }
}