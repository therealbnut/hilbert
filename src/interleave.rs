use crate::{bit_count, mask, zero_high_bits};
use num_traits::{cast::FromPrimitive, int::PrimInt};

pub trait Interleavable: PrimInt + private::Sealed {
    type Wider: PrimInt + FromPrimitive;
    fn as_wider(self) -> Self::Wider;
}

macro_rules! impl_interleavable {
    // `()` indicates that the macro takes no argument.
    ($lo: ty, $hi: ty) => {
        impl private::Sealed for $lo {}
        impl Interleavable for $lo {
            type Wider = $hi;
            fn as_wider(self) -> Self::Wider {
                self as $hi
            }
        }
    };
}

impl_interleavable!(u8, u16);
impl_interleavable!(u16, u32);
impl_interleavable!(u32, u64);
#[cfg(has_i128)]
impl_interleavable!(u64, u128);

mod private {
    pub trait Sealed {}
}

#[inline]
pub fn interleave_bits<T: Interleavable>(lhs: T, rhs: T) -> T::Wider {
    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
        use core::arch::x86_64::*;
        match bit_count::<T::Wider>() {
            32 => {
                let (mask_lhs, mask_rhs) = (mask::<u32>(1) << 1, mask::<u32>(1));
                let (lhs, rhs) = (lhs.to_u32().unwrap(), rhs.to_u32().unwrap());
                let output = unsafe { _pdep_u32(lhs, mask_lhs) | _pdep_u32(rhs, mask_rhs) };
                return <T::Wider as FromPrimitive>::from_u32(output).unwrap();
            }
            64 => {
                let (mask_lhs, mask_rhs) = (mask::<u64>(1) << 1, mask::<u64>(1));
                let (lhs, rhs) = (lhs.to_u64().unwrap(), rhs.to_u64().unwrap());
                let output = unsafe { _pdep_u64(lhs, mask_lhs) | _pdep_u64(rhs, mask_rhs) };
                return <T::Wider as FromPrimitive>::from_u64(output).unwrap();
            }
            _ => {}
        }
    }

    (interleave_with_zero(zero_high_bits(lhs.as_wider())) << 1)
        | interleave_with_zero(zero_high_bits(rhs.as_wider()))
}

#[inline]
fn interleave_with_zero<T: PrimInt>(input: T) -> T {
    let bits_wide = bit_count::<T>();
    let mut output = input;
    let mut mask_half_len = 1;
    let mut shift = bits_wide >> 2;
    while shift > 0 {
        output = (output ^ (output << shift)) & mask(mask_half_len);
        mask_half_len <<= 1;
        shift >>= 1;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interleave_bits() {
        assert_eq!(interleave_bits(0b0000u32, 0b0000u32), 0b00000000u64);
        assert_eq!(interleave_bits(0b1111u32, 0b1111u32), 0b11111111u64);
        assert_eq!(interleave_bits(0b0000u32, 0b1111u32), 0b01010101u64);
        assert_eq!(interleave_bits(0b1111u32, 0b0000u32), 0b10101010u64);
        assert_eq!(interleave_bits(0b1010u32, 0b0101u32), 0b10011001u64);
    }
}
