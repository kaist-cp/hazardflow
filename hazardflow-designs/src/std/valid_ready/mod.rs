//! Valid-ready protocol.

use super::hazard::*;
use super::interface::*;
use super::valid::*;
use crate::prelude::*;
use crate::std::*;

/// Hazard for valid-ready hazard interface.
///
/// - `Hazard::P` = `P`
/// - `Hazard::R` = `Ready<R>`
pub type VrH<P, R = ()> = AndH<ValidH<P, R>>;

/// Valid-ready interface.
///
/// - `Interface::Fwd` = `HOption<P>`
/// - `Interface::Bwd` = `Ready<()>`
pub type Vr<P, const D: Dep = { Dep::Helpful }> = I<VrH<P>, D>;

/// Attaches a ready signal to the module `m`'s egress interface.
///
/// The returned module's ingress ready signal is calculated as "`m`'s ingress ready signal" AND "attached ready
/// signal".
///
/// The returned module `attach_ready(m)` looks like the following:
///
/// ```text
///   (Ingress)                                  (Egress)
///                                   +-----+
/// HOption<P1> --------------------->|     |--> HOption<P2>
///          R1 <---------------------|  m  |<-- R2
///                +-----+            |     |
///                |     |<-- bool <--|     |
///        bool <--| AND |            +-----+
///                |     |<--------------------- bool
///                +-----+
/// ```
pub fn attach_ready<P1: Copy, R1: Copy, P2: Copy, R2: Copy>(
    m: impl FnOnce(I<VrH<P1, R1>, { Dep::Helpful }>) -> I<ValidH<P2, R2>, { Dep::Helpful }>,
) -> impl FnOnce(I<VrH<P1, R1>, { Dep::Helpful }>) -> I<VrH<P2, R2>, { Dep::Helpful }> {
    |i: I<VrH<P1, R1>, { Dep::Helpful }>| -> I<VrH<P2, R2>, { Dep::Helpful }> {
        // `dummy` is used for additional ready signal.
        let (i, dummy) = unsafe {
            Interface::fsm::<(I<VrH<P1, R1>, { Dep::Helpful }>, I<ValidH<(), bool>, { Dep::Helpful }>), ()>(
                i,
                (),
                |ip, (er1, er2), s| ((ip, None), Ready::new(er1.ready & er2, er1.inner), s),
            )
        };

        unsafe {
            (i.comb(m), dummy)
                .fsm::<I<VrH<P2, R2>, { Dep::Helpful }>, ()>((), |(ip, _), er, s| (ip, (er.inner, er.ready), s))
        }
    }
}

/// Attaches an additional resolver to the valid-ready circuit `m`.
///
/// The returned module `attach_resolver(m)` looks like the following:
///
/// ```text
///  (Ingress)               (Egress)
///               +-----+
/// HOption<P> -->|  m  |--> HOption<EP>
///       bool <--|     |<-- bool
///               +-----+
///          R <------------ R
/// ```
pub fn attach_resolver<P: Copy, EP: Copy, R: Copy>(
    m: impl FnOnce(Vr<P>) -> Vr<EP>,
) -> impl FnOnce(I<VrH<P, R>, { Dep::Helpful }>) -> I<VrH<EP, R>, { Dep::Helpful }> {
    |i: I<VrH<P, R>, { Dep::Helpful }>| -> I<VrH<EP, R>, { Dep::Helpful }> {
        // Always transferred to the first interface, `dummy` is used for additional resolver.
        let (i, dummy) = i.map(|p| (p, BoundedU::new(0.into_u()))).map_resolver_inner::<((), R)>(|(_, r)| r).branch();

        // Always transferred from the first interface.
        (i.comb(m), dummy.map(|_| unsafe { x::<EP>() }))
            .mux(Valid::constant(0.into_u()))
            .map_resolver_inner::<R>(|r| ((), r))
    }
}

/// Attaches an additional payload to the valid-ready circuit `m`.
///
/// The returned module `attach_payload(m)` looks like the following:
///
/// ```text
///        (Ingress)                                                    (Egress)
///                    +--> AP -----------------------------> AP --+
///                    |                  +-----+                  |
/// HOption<(P, AP)> --+--> HOption<P> -->|  m  |--> HOption<EP> --+--> HOption<(EP, AP)>
///             bool <--------------------|     |<--------------------- bool
///                                       +-----+
/// ```
///
/// NOTE: Current implementation considered only when `m` returns its egress at the same cycle as takes its ingress.
// TODO: Consider when `m` takes one or multi cycles.
pub fn attach_payload<P: Copy, EP: Copy, AP: Copy>(
    m: impl FnOnce(Vr<P>) -> Vr<EP>,
) -> impl FnOnce(Vr<(P, AP)>) -> Vr<(EP, AP)> {
    |i: Vr<(P, AP)>| -> Vr<(EP, AP)> {
        // `dummy` is used for additional payload.
        let (i, dummy) = i.lfork_uni();

        let i = i.map(|(p, _)| p);
        let dummy = dummy.map(|(_, ap)| ap); // TODO: Add `reg_fwd` with flow property to consider `m` takes one or multi cycles.

        (i.comb(m), dummy).join()
    }
}
