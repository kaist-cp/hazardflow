//! Valid protocol.

use super::hazard::*;
use super::*;

/// Hazard for hazard interfaces whose transfers can always happen.
#[derive(Debug, Clone, Copy)]
pub struct ValidH<P: Copy, R: Copy> {
    _marker: PhantomData<(P, R)>,
}

impl<P: Copy, R: Copy> Hazard for ValidH<P, R> {
    type P = P;
    type R = R;

    fn ready(_p: P, _h: R) -> bool {
        true
    }
}

/// Valid interface.
///
/// A transfer always happens for a valid payload.
///
/// - `Interface::Fwd` = `HOption<P>`
/// - `Interface::Bwd` = `()`
pub type Valid<P> = I<ValidH<P, ()>, { Dep::Helpful }>;
