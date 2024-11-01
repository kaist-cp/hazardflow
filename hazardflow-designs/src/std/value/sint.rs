//! Signed integer.

use super::*;

/// An signed integer with bitwidth `N`.
#[derive(Debug, Default, Clone, Copy)]
pub struct S<const N: usize>(U<N>);

impl<const N: usize> S<N> {
    /// Sign extends `S<N>` to `S<M>`.
    ///
    /// It panics when `M < N`.
    #[allow(clippy::identity_op)]
    pub fn sext<const M: usize>(self) -> S<M>
    where [(); N + M]: {
        if M >= N {
            let msb = self.0[N - 1];
            let inner = self.0.append(msb.repeat::<M>());
            S::from(inner.resize::<M>())
        } else {
            panic!("M should be larger than N")
        }
    }

    /// Resizes the bitwidth.
    ///
    /// It does not preserves the signedness.
    pub fn resize<const M: usize>(self) -> S<M> {
        S::from(U::from(self).resize())
    }

    /// Returns the maximum value of an `N` bit signed value. (i.e., 2^(`N` - 1) - 1)
    pub fn signed_max() -> S<N>
    where
        [(); N - 1]:,
        [(); (N - 1) + 1]:,
    {
        S::from(U::<N>::unsigned_max().clip_const::<{ N - 1 }>(0).append(U::<1>::from(0)).resize::<N>())
    }

    /// Returns the minimum value of an `N` bit unsigned value. (i.e., -2^(`N` - 1))
    pub fn signed_min() -> S<N>
    where
        [(); N - 1]:,
        [(); (N - 1) + 1]:,
    {
        S::from(U::<{ N - 1 }>::from(0).append(U::<1>::from(1)).resize::<N>())
    }
}

impl<const N: usize> From<U<N>> for S<N> {
    fn from(value: U<N>) -> S<N> {
        S(value)
    }
}

impl<const N: usize> From<S<N>> for U<N> {
    fn from(value: S<N>) -> U<N> {
        value.0
    }
}
