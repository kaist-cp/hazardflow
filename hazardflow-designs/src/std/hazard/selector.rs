//! Selector hazard specification.

use super::*;

/// Wraps `H` with additional selector bit in payload.
///
/// Selector bit represents the value in range [0, N).
#[derive(Debug, Clone, Copy)]
pub struct SelH<H: Hazard, const N: usize> {
    _marker: PhantomData<H>,
}

impl<H: Hazard, const N: usize> Hazard for SelH<H, N>
where [(); clog2(N)]:
{
    type P = (H::P, BoundedU<N>);
    type R = H::R;

    fn ready(p: Self::P, r: Self::R) -> bool {
        H::ready(p.0, r)
    }
}

impl<const N: usize, H: Hazard, const D: Dep> I<SelH<H, N>, D>
where [(); clog2(N)]:
{
    /// Transforms the muxed hazard to the inner hazard.
    pub fn into_inner(self) -> I<H, D> {
        unsafe { self.fsm((), |p, er, ()| (p.map(|p| p.0), er, ())) }
    }
}
