//! Filter map.

use super::*;

impl<P: Copy, R: Copy, const D: Dep> I<ValidH<P, R>, D> {
    /// Filter-maps the ingress payload into the egress payload.
    ///
    /// - Payload: Filter-maped by `f`.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `R`          | `R`           |
    pub fn filter_map<EP: Copy>(self, f: impl Fn(P) -> HOption<EP>) -> I<ValidH<EP, R>, D> {
        self.fsm_filter_map((), |p, ()| (f(p), ()))
    }

    /// A variation of [`filter_map`] that allows `f` to consider the egress resolver in addition to the ingress payload
    /// while calculating the egress payload.
    ///
    /// - Payload: Filter-maped by `f`.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `R`          | `R`           |
    pub fn filter_map_with_r<EP: Copy>(self, f: impl Fn(P, R) -> HOption<EP>) -> I<ValidH<EP, R>, { Dep::Demanding }> {
        self.filter_map_drop_with_r::<ValidH<EP, R>>(f)
    }
}

impl<P: Copy, R: Copy, const D: Dep> I<VrH<P, R>, D> {
    /// Filter-maps the ingress payload into the egress payload.
    ///
    /// - Payload: Filter-maped by `f`.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<R>`    |
    pub fn filter_map<EP: Copy>(self, f: impl Fn(P) -> HOption<EP>) -> I<VrH<EP, R>, D> {
        self.fsm_filter_map((), |p, ()| (f(p), ()))
    }

    /// A variation of [`filter_map`] that allows `f` to consider the inner value of the egress resolver in addition to
    /// the ingress payload while calculating the egress payload.
    ///
    /// - Payload: Filter-mapped by `f`. The payload is dropped if `er.ready` is false, even if `f` returns `Some`.
    ///     ([why?](super#notes-on-dropping-combinators))
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<R>`    |
    pub fn filter_map_drop_with_r_inner<EP: Copy>(
        self,
        f: impl Fn(P, R) -> HOption<EP>,
    ) -> I<VrH<EP, R>, { Dep::Demanding }> {
        self.filter_map_drop_with_r::<VrH<EP, R>>(|ip, er| if er.ready { f(ip, er.inner) } else { None })
    }
}

impl<H: Hazard, const D: Dep> I<H, D> {
    /// Filter-maps the ingress payload and the egress resolver to the egress payload.
    ///
    /// - Payload: Filter-mapped by `f`. The payload is dropped if `H::ready(ip, ir)` or `EH::ready(ep, er)` is false,
    ///     even if `f` returns `Some`. ([why?](super#notes-on-dropping-combinators))
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress         | Egress           |
    /// | :-------: | --------------- | ---------------- |
    /// |  **Fwd**  | `HOption<H::P>` | `HOption<EH::P>` |
    /// |  **Bwd**  | `H::R`          | `H::R`           |
    pub fn filter_map_drop_with_r<EH: Hazard<R = H::R>>(
        self,
        f: impl Fn(H::P, H::R) -> HOption<EH::P>,
    ) -> I<EH, { Dep::Demanding }> {
        unsafe {
            self.fsm::<(), { Dep::Demanding }, EH>((), |ip, er, s| {
                let ir = er;
                let ep = ip.filter(|ip| H::ready(ip, ir)).and_then(|ip| f(ip, er)).filter(|ep| EH::ready(ep, er));
                (ep, ir, s)
            })
        }
    }
}
