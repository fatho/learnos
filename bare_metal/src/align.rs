
/// Something (usually addresses or sizes) that is alignable to a certain alignment
/// represented in the same type and usually a power of two.
pub trait Alignable {
    type Alignment;

    /// Returns the smallest `x` that is a multiple of `alignment` such that `x >= num`.
    fn align_up(self, alignment: Self::Alignment) -> Self;

    /// Returns the largest `x` that is a multiple of `alignment` such that `x <= num`.
    fn align_down(self, alignment: Self::Alignment) -> Self;

    /// Returns whether the value is aligned to the given alignment.
    fn is_aligned(self, alignment: Self::Alignment) -> bool;
}


macro_rules! align_impl {
    ($numtype:tt) => {
        impl Alignable for $numtype {
            type Alignment = $numtype;
            fn align_up(self, alignment: Self) -> Self {
                if alignment == 0 {
                    self
                } else {
                    let mask = alignment - 1;
                    assert!(alignment & mask == 0, "alignment must be power of two");
                    let padding = alignment - (self & mask);
                    self + (padding & mask)
                }   
            }
            fn align_down(self, alignment: Self) -> Self {
                if alignment == 0 {
                    self
                } else {
                    let mask = alignment - 1;
                    assert!(alignment & mask == 0, "alignment must be power of two");
                    let padding = self & mask;
                    self - padding
                }
            }
            fn is_aligned(self, alignment: Self) -> bool {
                self.align_down(alignment) == self
            }
        }
    };
}

align_impl!(usize);
align_impl!(u64);
align_impl!(u32);
align_impl!(u16);
align_impl!(u8);

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