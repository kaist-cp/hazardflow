//! Filter.

use super::*;

impl<P: Copy, R: Copy, const D: Dep> I<ValidH<P, R>, D> {
    /// Filters the ingress payload.
    ///
    /// - Payload: Filtered by `f`.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `R`          | `R`          |
    pub fn filter(self, f: impl Fn(P) -> bool) -> I<ValidH<P, R>, D> {
        self.filter_map(|p| if f(p) { Some(p) } else { None })
    }
}

impl<P: Copy, R: Copy, const D: Dep> I<VrH<P, R>, D> {
    /// Filters the ingress payload.
    ///
    /// - Payload: Filtered by `f`.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<R>`   |
    pub fn filter(self, f: impl Fn(P) -> bool) -> I<VrH<P, R>, D> {
        self.filter_map(|p| if f(p) { Some(p) } else { None })
    }
}

impl<H: Hazard, const D: Dep> I<H, D> {
    /// Filters the ingress payload.
    ///
    /// - Payload: Filtered by `f`. The payload is dropped if `H::ready(ip, ir)` or `EH::ready(ep, er)` is false, even
    ///     if `f` returns `true`. ([why?](super#notes-on-dropping-combinators))
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress         | Egress          |
    /// | :-------: | --------------- | --------------- |
    /// |  **Fwd**  | `HOption<H::P>` | `HOption<H::P>` |
    /// |  **Bwd**  | `H::R`          | `H::R`          |
    pub fn filter_drop<EH: Hazard<P = H::P, R = H::R>>(self, f: impl Fn(H::P) -> bool) -> I<EH, { Dep::Demanding }> {
        self.filter_map_drop_with_r(|p, _| if f(p) { Some(p) } else { None })
    }
}
