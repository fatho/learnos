#![cfg_attr(not(test), no_std)]
#![feature(asm)]
#![feature(step_trait)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
#[macro_use]
extern crate static_assertions;

extern crate amd64;

use core::ops::{Deref, DerefMut, Index, IndexMut};

pub mod paging;
pub mod physical;
pub mod util;

/// Number of trailing zeros in a page aligned address.
pub const PAGE_ALIGN_BITS: u32 = 12;

/// Number of trailing zeros in a large page aligned address.
pub const LARGE_PAGE_ALIGN_BITS: u32 = 21;

/// Size of a normal physical page, $ KiB.
pub const PAGE_SIZE: usize = 1 << PAGE_ALIGN_BITS;

/// Size of a large physical page, 2 MiB
pub const LARGE_PAGE_SIZE: usize = 1 << LARGE_PAGE_ALIGN_BITS;

/// A pointer-wrapper around manually managed kernel memory.
/// Dropping it will leak the underlying memory, but still drop the element it contains.
#[derive(Debug)]
pub struct KBox<T> {
    ptr: *mut T,
}

impl<T> KBox<T> {
    pub unsafe fn from_raw(ptr: *mut T) -> KBox<T> {
        KBox {
            ptr: ptr
        }
    }
}

impl<T> Drop for KBox<T> {
    fn drop(&mut self) {
        unsafe {
            self.ptr.drop_in_place();
        }
    }
}

impl<T> Deref for KBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for KBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

/// A fixed size array of possibly uninitialized values in manually managed kernel memory.
pub struct KFixedVec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}

impl<T> KFixedVec<T> {
    /// Create the array from already initialized memory.
    pub unsafe fn from_raw(ptr: *mut T, len: usize) -> KFixedVec<T> {
        KFixedVec {
            ptr: ptr,
            len: len,
            cap: len,
        }
    }

    /// Create an empty array from the given memory range. The capacity is measured in the *number of elements*.
    pub unsafe fn from_raw_uninitialized(ptr: *mut T, cap: usize) -> KFixedVec<T> {
        KFixedVec {
            ptr: ptr,
            len: 0,
            cap: cap,
        }
    }

    /// The number of initialized elements in the array.
    pub fn len(&self) -> usize {
        self.len
    }

    /// The maximum number of elements in the array.
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Append a new element at the end of the initialized elements.
    pub fn push(&mut self, val: T) {
        assert!(self.len < self.cap);
        unsafe {
            self.ptr.add(self.len).write(val);
        }
        self.len += 1
    }

    /// Return the initialized part of the array as a slice.
    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Return the initialized part of the array as a mutable slice.
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    pub fn iter(&self) -> core::slice::Iter<T> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<T> {
        self.as_slice_mut().iter_mut()
    }
}

impl<T> Drop for KFixedVec<T> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe {
                self.ptr.add(i).drop_in_place()
            }
        }
    }
}

impl<T> Index<usize> for KFixedVec<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &T {
        assert!(idx < self.len);
        unsafe { &*self.ptr.add(idx) }
    }
}

impl<T> IndexMut<usize> for KFixedVec<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        assert!(idx < self.len);
        unsafe { &mut *self.ptr.add(idx) }
    }
}
