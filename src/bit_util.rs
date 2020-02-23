use num_traits::int::PrimInt;

#[inline(always)]
pub fn mask<T: PrimInt>(mask_half_len: usize) -> T {
    !T::zero() / ((T::one() << mask_half_len) + T::one())
}

#[inline]
pub fn mask_pow2_and_under<T>(x: T) -> T
where
    T: PrimInt,
{
    debug_assert!(x == T::zero() || (x & (x - T::one()) == T::zero()));
    x | (x.max(T::one()) - T::one())
}

#[inline(always)]
pub fn zero_high_bits<T: PrimInt>(value: T) -> T {
    let bits_wide = bit_count::<T>() >> 1;
    let mask = (!T::zero()) >> bits_wide;
    value & mask
}

#[inline(always)]
pub fn bit_count<T>() -> usize {
    std::mem::size_of::<T>() << 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask() {
        assert_eq!(mask::<u32>(16), 0x0000_FFFF_u32);
        assert_eq!(mask::<u32>(8), 0x00FF_00FF_u32);
        assert_eq!(mask::<u32>(4), 0x0F0F_0F0F_u32);
        assert_eq!(mask::<u32>(2), 0x3333_3333_u32);
        assert_eq!(mask::<u32>(1), 0x5555_5555_u32);
    }

    #[test]
    fn test_zero_high_bits() {
        assert_eq!(zero_high_bits(0xFEDCBA0987654321), 0x87654321u64);
        assert_eq!(zero_high_bits(0x87654321), 0x4321u32);
        assert_eq!(zero_high_bits(0x4321), 0x21u16);
        assert_eq!(zero_high_bits(0x21), 0x1u8);
    }

    #[test]
    fn test_mask_pow2_and_under() {
        assert_eq!(mask_pow2_and_under(0x100), 0x1FF);
        assert_eq!(mask_pow2_and_under(0x001), 0x001);
        assert_eq!(mask_pow2_and_under(0x000), 0x000);
    }
}
