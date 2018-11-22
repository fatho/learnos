use core::ops::{RangeBounds, Bound};
use core::mem;

macro_rules! mask {
    ($numtype:ty, $from:expr, $to:expr) => {
        !(((1 as $numtype) << $from) - (1 as $numtype)) & (((1 as $numtype) << $to) - 1 + ((1 as $numtype) << $to))
    };
}

pub trait BitRange {
    fn first_bit(&self) -> usize;
    fn last_bit<T: Sized>(&self) -> usize;
}

impl<R: RangeBounds<usize>> BitRange for R {
    #[inline(always)]
    fn first_bit(&self) -> usize {
        match self.start_bound() {
            Bound::Included(incl) => *incl,
            Bound::Excluded(excl) => *excl + 1,
            Bound::Unbounded => 0
        }
    }

    #[inline(always)]
    fn last_bit<T: Sized>(&self) -> usize {
        match self.end_bound() {
            Bound::Included(incl) => *incl,
            Bound::Excluded(excl) => *excl - 1,
            Bound::Unbounded => mem::size_of::<T>() * 8 - 1
        }
    }
}

pub trait Bits {
    fn get_bit(&self, idx: usize) -> bool;
    fn set_bit(&mut self, idx: usize, value: bool);

    #[inline(always)]
    fn toggle_bit(&mut self, idx: usize) {
        self.set_bit(idx, ! self.get_bit(idx));
    }

    fn get_bits<R: BitRange>(&self, bits: R) -> Self;
    fn set_bits<R: BitRange>(&mut self, bits: R, value: Self);
}

macro_rules! impl_Bits {
    ($bittype:ty) => {

        impl Bits for $bittype {
            #[inline(always)]
            fn get_bit(&self, idx: usize) -> bool {
                self & (1 << idx) != 0
            }

            #[inline(always)]
            fn set_bit(&mut self, idx: usize, value: bool) {
                if value {
                    *self |= 1 << idx;
                } else {
                    *self &= !(1 << idx);
                }
            }

            #[inline(always)]
            fn get_bits<R: BitRange>(&self, range: R) -> Self {
                (*self & mask!($bittype, range.first_bit(), range.last_bit::<$bittype>())) >> range.first_bit()
            }

            #[inline(always)]
            fn set_bits<R: BitRange>(&mut self, range: R, value: Self) {
                let mask1 = mask!($bittype, range.first_bit(), range.last_bit::<$bittype>());
                let mask2 = mask!($bittype, 0, range.last_bit::<$bittype>() - range.first_bit());
                *self = (*self & !mask1) | (value & mask2) << range.first_bit();
            }
        }
    };
}

impl_Bits!(u16);
impl_Bits!(u32);
impl_Bits!(u64);
impl_Bits!(usize);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mask() {
        assert_eq!(mask!(u32, 0, 3), 0b1111);
        assert_eq!(mask!(u32, 5, 7), 0b11100000);
        assert_eq!(mask!(u32, 0, 31), 0xFFFFFFFF);
    }

    #[test]
    fn test_range() {
        assert_eq!((2..10).first_bit(), 2);
        assert_eq!((2..10).last_bit::<u32>(), 9);
        assert_eq!((2..=10).first_bit(), 2);
        assert_eq!((2..=10).last_bit::<u32>(), 10);
    }

    #[test]
    fn test_get_bits() {
        let foo: u16 = 0b1100_0110_0011_1101;
        assert!(foo.get_bit(0));
        assert!(!foo.get_bit(1));

        assert_eq!(foo.get_bits(0..=3), 0b1101);
        assert_eq!(foo.get_bits(4..=7), 0b0011);
        assert_eq!(foo.get_bits(8..=14), 0b100_0110);
        assert_eq!(foo.get_bits(0..=15), foo);

        assert_eq!(foo.get_bits(0..4), 0b1101);
        assert_eq!(foo.get_bits(4..8), 0b0011);
        assert_eq!(foo.get_bits(8..15), 0b100_0110);
        assert_eq!(foo.get_bits(0..16), foo);

        assert_eq!(foo.get_bits(14..), 0b11);
        assert_eq!(foo.get_bits(..4), 0b1101);
    }

    #[test]
    fn test_set_bits() {
        let mut foo: u16 = 0;
        foo.set_bit(2, true);
        assert_eq!(foo, 0b0000_0000_0000_0100);
        foo.toggle_bit(2);
        assert_eq!(foo, 0b0000_0000_0000_0000);

        foo.set_bits(0..=3, 0b1101);
        assert_eq!(foo, 0b0000_0000_0000_1101);
        foo.set_bits(4..=7, 0b0011);
        assert_eq!(foo, 0b0000_0000_0011_1101);
        foo.set_bits(8..=14, 0b100_0110);
        assert_eq!(foo, 0b0100_0110_0011_1101);
        foo.set_bits(0..=15, foo);
        assert_eq!(foo, 0b0100_0110_0011_1101);

        foo = 0;
        foo.set_bits(0..4, 0b1101);
        assert_eq!(foo, 0b0000_0000_0000_1101);
        foo.set_bits(4..8, 0b0011);
        assert_eq!(foo, 0b0000_0000_0011_1101);
        foo.set_bits(8..15, 0b100_0110);
        assert_eq!(foo, 0b0100_0110_0011_1101);
        foo.set_bits(0..16, foo);
        assert_eq!(foo, 0b0100_0110_0011_1101);

        foo.set_bits(14.., 0b11);
        assert_eq!(foo, 0b1100_0110_0011_1101);
        foo.set_bits(..4, 0b1001);
        assert_eq!(foo, 0b1100_0110_0011_1001);
    }
}