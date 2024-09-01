//! Fir filter implementation

use crate::prelude::*;
use crate::std::*;

impl<P: Copy + Default> Valid<P> {
    /// Window combinator
    /// It takes a stream of input value P and return the latest N values.
    fn window<const N: usize>(self) -> Valid<Array<P, N>>
    where [(); N + 1]: {
        self.fsm_map(P::default().repeat::<N>(), |ip, s| {
            let ep = s.append(ip.repeat::<1>()).clip_const::<N>(0);
            let s_next = ep;
            (ep, s_next)
        })
    }
}

impl<const N: usize> Valid<Array<u32, N>> {
    /// Weight combinator
    fn weight(self, weight: [u32; N]) -> Valid<Array<u32, N>> {
        self.map(|ip| ip.zip(Array::from(weight)).map(|(ele, weight)| ele * weight))
    }

    /// Sum combinator
    /// It will add up all the elements within an array.
    fn sum(self) -> Valid<u32> {
        self.map(|ip| ip.fold_assoc(|e1, e2| e1 + e2))
    }
}

/// FIR filter implementation
#[synthesize]
pub fn fir_filter(input: Valid<u32>) -> Valid<u32> {
    input.window::<3>().weight([4, 2, 3]).sum()
}
