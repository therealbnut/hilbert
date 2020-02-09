use num_traits::int::PrimInt;
use std::cmp::Ordering;

use crate::{interleave_bits, Interleavable, bit_count, mask, mask_pow2_and_under};

/// HilbertPrecompute
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct HilbertPrecompute<T> where T: Interleavable {
    x: T,
    y: T,
    flip: T,
    swap: T,
}

impl<T> HilbertPrecompute<T> where T: Interleavable {
    #[inline]
    pub fn new(x: T, y: T) -> Self {
        let (flip, swap) = xy_flip_swap(x, y);
        Self {
            x,
            y,
            flip,
            swap
        }
    }

    #[inline(always)]
    pub fn x(&self) -> T {
        self.x
    }

    #[inline(always)]
    pub fn y(&self) -> T {
        self.y
    }

    #[inline]
    pub fn distance(&self) -> T::Wider {
        let xy_diff = self.x ^ self.y;
        let diff = (xy_diff & self.swap) ^ self.flip;
        interleave_bits(self.x ^ diff, xy_diff)
    }
}

impl<T> PartialEq<(T, T)> for HilbertPrecompute<T> where T: Interleavable {
    #[inline]
    fn eq(&self, that: &(T, T)) -> bool {
        self.x == that.0 && self.y == that.1
    }
}

impl<T> PartialOrd<(T, T)> for HilbertPrecompute<T> where T: Interleavable {
    #[inline]
    fn partial_cmp(&self, that: &(T, T)) -> Option<Ordering> {
        let (lhs_x, lhs_y, rhs_x, rhs_y) = (self.x, self.y, that.0, that.1);
        let matching_prefix = ((lhs_x ^ rhs_x) | (lhs_y ^ rhs_y)).leading_zeros() as usize;
        if matching_prefix == bit_count::<T>() {
            Some(Ordering::Equal)
        }
        else {
            let loc_lhs = xy_local_dist(matching_prefix, lhs_x, lhs_y, self.flip, self.swap);
            let loc_rhs = xy_local_dist(matching_prefix, rhs_x, rhs_y, self.flip, self.swap);
            loc_lhs.partial_cmp(&loc_rhs)
        }
    }
}

fn xy_flip_swap<T>(x: T, y: T) -> (T, T) where T: PrimInt {
    let bits_wide = bit_count::<T>();
    let swap_pattern = mask::<T>(1);
    let zeros = (x | y).leading_zeros() as usize;

    // if x == y
    if zeros == bits_wide {
        return (T::zero(), y);
    }

    let mut bit: T = (T::one() << (bits_wide - 1)) >> zeros;
    let mut flip: T = T::zero();
    let mut swap: T = !swap_pattern & !(!T::zero() >> zeros);

    swap = swap | mask_pow2_and_under(swap & (bit << 1));

    // Consider turning this into T::Wider to combine with xy_mask.
    let diff = x ^ y;
    let (x, y) = (x, !y);

    while bit != T::zero() {
        let xy_mask = (swap & diff) ^ flip;
        swap = swap ^ mask_pow2_and_under((y ^ xy_mask) & bit);
        flip = flip ^ mask_pow2_and_under((x ^ xy_mask) & bit & diff);
        bit = bit >> 1;
    }

    (flip, swap)
}

fn xy_local_dist<T>(log2_n: usize, x: T, y: T, flip: T, swap: T) -> T::Wider where T: Interleavable {
    let bits_wide = bit_count::<T>();
    let wide_one = <T::Wider as num_traits::One>::one();

    let mask = (wide_one | (wide_one << bits_wide)) << (bits_wide - log2_n - 1);
    let xy_diff = x ^ y;
    let diff = (xy_diff & swap) ^ flip;

    let a = (x ^ diff).as_wider();
    let b = xy_diff.as_wider();

    ((a << bits_wide) | b) & mask
}
