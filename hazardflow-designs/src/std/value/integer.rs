//! Integer.

use core::cmp::Ordering;
use core::ops::*;

use hazardflow_macro::magic;

use super::Array;
use crate::prelude::*;

/// An integer with bitwidth `N`.
///
/// The lower bits of the integer are represented by the lower index of the array, and vice versa. In other words, the
/// least significant bit of the integer is the 0th element of the array, and the most significant bit is the
/// (`N` - 1)-th element.
pub type U<const N: usize> = Array<bool, N>;

impl<const N: usize> From<U<N>> for u32 {
    #[magic(int::convert)]
    fn from(_value: U<N>) -> Self {
        compiler_magic!()
    }
}

impl<const N: usize> From<U<N>> for u8 {
    #[magic(int::convert)]
    fn from(_value: U<N>) -> Self {
        compiler_magic!()
    }
}

impl<const N: usize> From<i32> for U<N> {
    #[magic(int::convert)]
    fn from(_value: i32) -> U<N> {
        compiler_magic!()
    }
}

impl<const N: usize> From<u32> for U<N> {
    #[magic(int::convert)]
    fn from(_value: u32) -> U<N> {
        compiler_magic!()
    }
}

impl<const N: usize> From<usize> for U<N> {
    #[magic(int::convert)]
    fn from(_value: usize) -> U<N> {
        compiler_magic!()
    }
}

impl<const N: usize> From<u128> for U<N> {
    #[magic(int::convert)]
    fn from(_value: u128) -> U<N> {
        compiler_magic!()
    }
}

impl From<bool> for U<1> {
    #[magic(int::convert)]
    fn from(_value: bool) -> U<1> {
        compiler_magic!()
    }
}

impl<const N: usize> From<U<N>> for bool {
    #[magic(int::convert)]
    fn from(_value: U<N>) -> bool {
        compiler_magic!()
    }
}

impl<const N: usize> Not for U<N> {
    type Output = Self;

    #[magic(int::not)]
    fn not(self) -> Self::Output {
        compiler_magic!()
    }
}

impl<const N: usize, const M: usize> Shr<U<M>> for U<N> {
    type Output = Self;

    #[magic(int::shr)]
    fn shr(self, _rhs: U<M>) -> Self::Output {
        compiler_magic!()
    }
}

impl<const N: usize> Shr<usize> for U<N> {
    type Output = Self;

    #[magic(int::shr)]
    fn shr(self, _rhs: usize) -> Self::Output {
        compiler_magic!()
    }
}

impl<const N: usize, const M: usize> Shl<U<M>> for U<N> {
    type Output = Self;

    #[magic(int::shl)]
    fn shl(self, _lhs: U<M>) -> Self::Output {
        compiler_magic!()
    }
}

impl<const N: usize> Shl<usize> for U<N> {
    type Output = Self;

    #[magic(int::shl)]
    fn shl(self, _lhs: usize) -> Self::Output {
        compiler_magic!()
    }
}

impl<const N: usize> Add<U<N>> for U<N>
where [(); N + 1]:
{
    type Output = U<{ N + 1 }>;

    #[magic(int::add)]
    fn add(self, _rhs: U<N>) -> U<{ N + 1 }> {
        compiler_magic!()
    }
}

#[allow(clippy::identity_op)]
impl<const N: usize> U<N>
where [(); N + 1]:
{
    /// Adds two `U<N>`s and truncate the result to `U<N>`.
    pub fn trunk_add(self, rhs: Self) -> Self {
        (self + rhs).resize()
    }

    /// Sign extends `U<N>` to `U<M>`.
    pub fn sext<const M: usize>(self) -> U<M>
    where
        [(); (M - N) * 1]:,
        [(); M * N]:,
        [(); N + (M - N)]:,
    {
        if M >= N {
            let msb_arr: Array<bool, { M - N }> = self.clip_const::<1>(N - 1).repeat::<{ M - N }>().concat().resize();
            self.append(msb_arr).resize::<M>()
        } else {
            panic!("M should be larger than N")
        }
    }
}

impl<const N: usize> U<N> {
    /// Returns the maximum value of an `N` bit unsigned value. (i.e., 2^`N` - 1)
    pub fn unsigned_max() -> U<N> {
        true.repeat::<N>()
    }

    /// Returns the maximum value of an `N` bit signed value. (i.e., 2^(`N` - 1) - 1)
    pub fn signed_max() -> U<N>
    where
        [(); N - 1]:,
        [(); (N - 1) + 1]:,
    {
        Self::unsigned_max().clip_const::<{ N - 1 }>(0).append(U::<1>::from(0)).resize::<N>()
    }

    /// Returns the minimum value of an `N` bit unsigned value. (i.e., -2^(`N` - 1))
    pub fn signed_min() -> U<N>
    where
        [(); N - 1]:,
        [(); (N - 1) + 1]:,
    {
        U::<{ N - 1 }>::from(0).append(U::<1>::from(1)).resize::<N>()
    }
}

impl<const N: usize> Sub<U<N>> for U<N> {
    type Output = U<N>;

    #[magic(int::sub)]
    fn sub(self, _other: U<N>) -> U<N> {
        compiler_magic!()
    }
}

impl<const N: usize, const M: usize> Mul<U<M>> for U<N>
where [(); N + M]:
{
    type Output = U<{ N + M }>;

    #[magic(int::mul)]
    fn mul(self, _other: U<M>) -> Self::Output {
        compiler_magic!()
    }
}

impl<const N: usize> PartialOrd for U<N> {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        panic!("placeholder for rust's type system")
    }

    #[magic(int::lt)]
    fn lt(&self, _other: &Self) -> bool {
        compiler_magic!()
    }

    #[magic(int::le)]
    fn le(&self, _other: &Self) -> bool {
        compiler_magic!()
    }

    #[magic(int::gt)]
    fn gt(&self, _other: &Self) -> bool {
        compiler_magic!()
    }

    #[magic(int::ge)]
    fn ge(&self, _other: &Self) -> bool {
        compiler_magic!()
    }
}

/// Trait for converting a type into `U<N>`.
pub trait IntoU {
    /// Converts `self` into `U<N>`.
    fn into_u<const N: usize>(self) -> U<N>;
}

impl IntoU for i32 {
    fn into_u<const N: usize>(self) -> U<N> {
        U::from(self)
    }
}
impl IntoU for usize {
    fn into_u<const N: usize>(self) -> U<N> {
        U::from(self)
    }
}
impl IntoU for u32 {
    fn into_u<const N: usize>(self) -> U<N> {
        U::from(self)
    }
}

impl IntoU for bool {
    fn into_u<const N: usize>(self) -> U<N> {
        U::from(self).resize()
    }
}

impl<const M: usize> IntoU for [bool; M] {
    fn into_u<const N: usize>(self) -> U<N> {
        U::from(self).resize()
    }
}
