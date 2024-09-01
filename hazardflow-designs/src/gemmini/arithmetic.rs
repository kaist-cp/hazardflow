//! Arithmetic functions.

use super::*;

/// Rounding shift (round-to-nearest-even)
/// <https://github.com/ucb-bar/gemmini/blob/be2e9f26181658895ebc7ca7f7d6be6210f5cdef/src/main/scala/gemmini/Arithmetic.scala#L97C7-L97C22>
/// <https://github.com/riscv/riscv-v-spec/blob/master/v-spec.adoc#38-vector-fixed-point-rounding-mode-register-vxrm>
pub fn rounding_shift(val: U<32>, shamt: U<5>) -> U<32> {
    let val_i32 = u32::from(val) as i32; // $signed(c1)
    let shamt_usize = u32::from(shamt) as usize;
    let round_down_shifted = val_i32 >> u32::from(shamt);

    // d != 0
    let nonzero_shamt = shamt.any(|x| x);

    // v[d-2:0] != 0
    let zeros = if shamt_usize < 2 {
        false
    } else {
        let mask = (1 << (shamt_usize - 1)) - 1;
        (val_i32 & mask) != 0
    };

    // d != 0 && v[d-1] && (v[d-2:0]!=0 | v[d])
    let r = (nonzero_shamt & val[shamt_usize - 1] & (zeros | val[shamt_usize])) as i32;

    (round_down_shifted + r).into_u()
}

/// Same as `clippedToWidthOf` function.
/// <https://github.com/ucb-bar/gemmini/blob/be2e9f26181658895ebc7ca7f7d6be6210f5cdef/src/main/scala/gemmini/Arithmetic.scala#L122C20-L126>
pub fn clip_with_saturation<const N: usize, const M: usize>(val: U<N>) -> U<M>
where
    [(); M - 1]:,
    [(); (M - 1) + 1]:,
{
    let val = u32::from(val) as i32;

    let sat_max = u32::from(U::<M>::signed_max()) as i32;
    let sat_min = u32::from(U::<M>::signed_min().resize::<20>().sext::<32>()) as i32;
    let clipped = if val > sat_max {
        sat_max
    } else if val < sat_min {
        sat_min
    } else {
        val
    };

    clipped.into_u()
}
