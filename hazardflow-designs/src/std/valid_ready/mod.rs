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
pub fn attach_ready<P1: Copy, R1: Copy, P2: Copy, R2: Copy, const D1: Dep, const D2: Dep>(
    m: impl FnOnce(I<VrH<P1, R1>, D1>) -> I<ValidH<P2, R2>, D2>,
) -> impl FnOnce(I<VrH<P1, R1>, D1>) -> I<VrH<P2, R2>, D2> {
    |i: I<VrH<P1, R1>, D1>| -> I<VrH<P2, R2>, D2> {
        let (i, ready) = unsafe {
            Interface::fsm::<(I<VrH<P1, R1>, D1>, I<ValidH<(), bool>, { Dep::Helpful }>), ()>(
                i,
                (),
                |ip, (er1, er2), s| ((ip, None), Ready::new(er1.ready & er2, er1.inner), s),
            )
        };

        let e = i.comb(m);

        unsafe { (e, ready).fsm::<I<VrH<P2, R2>, D2>, ()>((), |(ip, _), er, s| (ip, (er.inner, er.ready), s)) }
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
pub fn attach_resolver<P: Copy, EP: Copy, R: Copy, const D: Dep, const ED: Dep>(
    m: impl FnOnce(Vr<P, D>) -> Vr<EP, ED>,
) -> impl FnOnce(I<VrH<P, R>, D>) -> I<VrH<EP, R>, ED> {
    |i: I<VrH<P, R>, D>| -> I<VrH<EP, R>, ED> {
        // TODO: Need to consider `m` need multi-cycle
        let (payload, hazard) = unsafe {
            Interface::fsm::<(Vr<P, D>, I<ValidH<(), R>, { Dep::Helpful }>), ()>(i, (), |ip, (er1, er2), s| {
                let ir = Ready::new(er1.ready, er2);
                // let ep1 = ip.filter(|ip| ReadyH::<H>::ready(ip, ir));
                let ep1 = ip;
                let ep2 = Some(());
                ((ep1, ep2), ir, s)
            })
        };

        unsafe {
            (payload.comb(m), hazard).fsm::<I<VrH<EP, R>, ED>, ()>((), |(ip1, _), er, s| {
                let ir1 = er.map(|_| ());
                let ir2 = er.inner;
                let ep = ip1;
                (ep, (ir1, ir2), s)
            })
        }
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
pub fn attach_payload<P: Copy, EP: Copy, AP: Copy, const D: Dep, const ED: Dep>(
    m: impl FnOnce(Vr<P, D>) -> Vr<EP, ED>,
) -> impl FnOnce(Vr<(P, AP), D>) -> Vr<(EP, AP), ED> {
    |i: Vr<(P, AP), D>| -> Vr<(EP, AP), ED> {
        // TODO: Need to consider `m` need multi-cycle
        let (i1, i2) = unsafe {
            Interface::fsm::<(Vr<P, D>, Vr<AP, D>), ()>(i, (), |ip, (er1, _er2), s| {
                let ep1 = ip.map(|(p, _)| p);
                let ep2 = ip.map(|(_, ap)| ap);
                // let ir = Ready::new(er1.ready & er2.ready, ());
                let ir = er1;
                ((ep1, ep2), ir, s)
            })
        };

        let e1 = i1.comb(m);
        let e2 = i2; // TODO: We should be add `reg_fwd` which have flow property.

        unsafe {
            (e1, e2).fsm::<Vr<(EP, AP), ED>, ()>((), |(ip1, ip2), er, s| {
                let ep = ip1.zip(ip2);
                let ir1 = er;
                let ir2 = er;
                (ep, (ir1, ir2), s)
            })
        }
    }
}
