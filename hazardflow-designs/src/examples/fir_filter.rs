//! Fir filter implementation

use crate::prelude::*;
use crate::std::*;

impl<P: Copy + Default> Valid<P> {
    /// Window combinator.
    ///
    /// It takes a stream of input value P and return the latest N values.
    fn window<const N: usize>(self) -> Valid<Array<P, N>>
    where
        [(); N - 1]:,
        [(); 1 + (N - 1)]:,
    {
        self.fsm_map(P::default().repeat::<{ N - 1 }>(), |ip, s| {
            let ep = ip.repeat::<1>().append(s).resize::<N>();
            let s_next = ep.clip_const::<{ N - 1 }>(0);
            (ep, s_next)
        })
    }
}

impl<const N: usize> Valid<Array<u32, N>> {
    /// Sum combinator.
    ///
    /// It adds up all the elements within an array.
    fn sum(self) -> Valid<u32> {
        self.map(|ip| ip.fold(0, |acc, e| acc + e))
    }
}

/// FIR filter implementation
#[synthesize]
pub fn fir_filter(input: Valid<u32>) -> Valid<u32> {
    input.window::<3>().map(|ip| ip.zip(Array::from([4, 2, 3])).map(|(e, wt)| e * wt)).sum()
}
