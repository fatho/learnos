
/// Something (usually addresses or sizes) that is alignable to a certain alignment
/// represented in the same type and usually a power of two.
pub trait Alignable {
    type Alignment;

    /// Return the smallest `x` that is a multiple of `alignment` such that `x >= num`.
    fn align_up(self, alignment: Self::Alignment) -> Self;

    /// Return the largest `x` that is a multiple of `alignment` such that `x <= num`.
    fn align_down(self, alignment: Self::Alignment) -> Self;
}


macro_rules! align_up_impl {
    ($num:ident, $alignment:ident) => {
        if $alignment == 0 {
            $num
        } else {
            let mask = $alignment - 1;
            assert!($alignment & mask == 0, "alignment must be power of two");
            let padding = $alignment - ($num & mask);
            $num + (padding & mask)
        }   
    };
}

macro_rules! align_down_impl {
    ($num:ident, $alignment:ident) => {
        if $alignment == 0 {
            $num
        } else {
            let mask = $alignment - 1;
            assert!($alignment & mask == 0, "alignment must be power of two");
            let padding = $num & mask;
            $num - padding
        }
    };
}

impl Alignable for usize {
    type Alignment = usize;
    fn align_up(self, alignment: Self) -> Self { align_up_impl!(self, alignment)  }
    fn align_down(self, alignment: Self) -> Self { align_down_impl!(self, alignment) }
}

impl Alignable for u64 {
    type Alignment = u64;
    fn align_up(self, alignment: Self) -> Self { align_up_impl!(self, alignment)  }
    fn align_down(self, alignment: Self) -> Self { align_down_impl!(self, alignment) }
}

impl Alignable for u32 {
    type Alignment = u32;
    fn align_up(self, alignment: Self) -> Self { align_up_impl!(self, alignment)  }
    fn align_down(self, alignment: Self) -> Self { align_down_impl!(self, alignment) }
}

impl Alignable for u16 {
    type Alignment = u16;
    fn align_up(self, alignment: Self) -> Self { align_up_impl!(self, alignment)  }
    fn align_down(self, alignment: Self) -> Self { align_down_impl!(self, alignment) }
}

impl Alignable for u8 {
    type Alignment = u8;
    fn align_up(self, alignment: Self) -> Self { align_up_impl!(self, alignment)  }
    fn align_down(self, alignment: Self) -> Self { align_down_impl!(self, alignment) }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn align_down_test() {
        assert_eq!(23_usize.align_down(8), 16);
        assert_eq!(24_usize.align_down(8), 24);
        assert_eq!(25_usize.align_down(8), 24);

        // edge cases
        assert_eq!(23_usize.align_down(0), 23);
        assert_eq!(0_usize.align_down(0), 0);
        assert_eq!(0xFFFF_FFFF_FFFF_FFFF_usize.align_down(0), 0xFFFF_FFFF_FFFF_FFFF);
    }

    #[test]
    fn align_up_test() {
        assert_eq!(23_usize.align_up(8), 24);
        assert_eq!(24_usize.align_up(8), 24);
        assert_eq!(25_usize.align_up(8), 32);

        // edge cases
        assert_eq!(23_usize.align_up(0), 23);
        assert_eq!(0_usize.align_up(0), 0);
        assert_eq!(0xFFFF_FFFF_FFFF_FFFF_usize.align_up(0), 0xFFFF_FFFF_FFFF_FFFF);
    }
}