use std::cmp::Ordering;

use crate::{interleave_bits, Interleavable, bit_count, mask, mask_pow2_and_under};

/// HilbertPrecompute
#[derive(Eq, Hash, Clone, Copy)]
pub struct HilbertPrecomputeData<T> where T: Interleavable {
    flip: T,
    swap: T,
}

impl<T> HilbertPrecomputeData<T> where T: Interleavable {
    pub fn new(x: T, y: T) -> Self {
        let bits_wide = bit_count::<T>();
        let swap_pattern = mask::<T>(1);
        let zeros = (x | y).leading_zeros() as usize;
    
        // if x == y
        if zeros == bits_wide {
            return Self { flip: T::zero(), swap: y };
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
    
        Self { flip, swap }
    }
}

impl<T> PartialEq for HilbertPrecomputeData<T> where T: Interleavable {
    #[inline]
    fn eq(&self, that: &Self) -> bool {
        self.flip == that.flip && self.swap == that.swap
    }
}

/// HilbertPrecompute
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct HilbertPrecompute<T> where T: Interleavable {
    x: T,
    y: T,
    data: HilbertPrecomputeData<T>,
}

impl<T> HilbertPrecompute<T> where T: Interleavable {
    #[inline]
    pub fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
            data: HilbertPrecomputeData::new(x, y),
        }
    }

    #[inline]
    pub fn new_with_data(x: T, y: T, data: HilbertPrecomputeData<T>) -> Self {
        debug_assert!(HilbertPrecomputeData::new(x, y) == data);
        Self {
            x,
            y,
            data,
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
        let diff = (xy_diff & self.data.swap) ^ self.data.flip;
        interleave_bits(self.x ^ diff, xy_diff)
    }
}

impl<T> PartialEq<(T, T)> for HilbertPrecompute<T> where T: Interleavable {
    #[inline]
    fn eq(&self, that: &(T, T)) -> bool {
        self.x == that.0 && self.y == that.1
    }
}

fn hilbert_cmp<T>(lhs: &HilbertPrecompute<T>, (rhs_x, rhs_y): (T, T)) -> Ordering
    where T: Interleavable
{
    let matching_prefix = ((lhs.x ^ rhs_x) | (lhs.y ^ rhs_y)).leading_zeros() as usize;
    if matching_prefix == bit_count::<T>() {
        Ordering::Equal
    }
    else {
        let loc_lhs = xy_local_dist(matching_prefix, lhs.x, lhs.y, lhs.data.flip, lhs.data.swap);
        let loc_rhs = xy_local_dist(matching_prefix, rhs_x, rhs_y, lhs.data.flip, lhs.data.swap);
        loc_lhs.cmp(&loc_rhs)
    }
}

impl<T> PartialOrd<(T, T)> for HilbertPrecompute<T> where T: Interleavable {
    #[inline]
    fn partial_cmp(&self, that: &(T, T)) -> Option<Ordering> {
        Some(hilbert_cmp(self, *that))
    }
}

impl<T> PartialOrd for HilbertPrecompute<T> where T: Interleavable {
    #[inline]
    fn partial_cmp(&self, that: &Self) -> Option<Ordering> {
        Some(hilbert_cmp(self, (that.x, that.y)))
    }
}

impl<T> Ord for HilbertPrecompute<T> where T: Interleavable {
    #[inline]
    fn cmp(&self, that: &Self) -> Ordering {
        hilbert_cmp(self, (that.x, that.y))
    }
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
