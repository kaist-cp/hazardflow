//! Map.

use super::*;

impl<P: Copy, R: Copy, const D: Dep> I<ValidH<P, R>, D> {
    /// Maps the ingress payload into the egress payload.
    ///
    /// - Payload: Mapped by `f`.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `R`          | `R`           |
    pub fn map<EP: Copy>(self, f: impl Fn(P) -> EP) -> I<ValidH<EP, R>, D> {
        self.filter_map(|p| Some(f(p)))
    }
}

impl<P: Copy, R: Copy, const D: Dep> I<VrH<P, R>, D> {
    /// Maps the ingress payload into the egress payload.
    ///
    /// - Payload: Mapped by `f`.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<R>`    |
    pub fn map<EP: Copy>(self, f: impl Fn(P) -> EP) -> I<VrH<EP, R>, D> {
        self.filter_map(|p| Some(f(p)))
    }
}

impl<H: Hazard, const D: Dep> I<H, D> {
    /// Maps the ingress payload to the egress payload.
    ///
    /// - Payload: Mapped by `f`. The payload is dropped if `H::ready(ip, ir)` or `EH::ready(ep, er)` is false.
    ///     ([why?](super#notes-on-dropping-combinators))
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress         | Egress           |
    /// | :-------: | --------------- | ---------------- |
    /// |  **Fwd**  | `HOption<H::P>` | `HOption<EH::P>` |
    /// |  **Bwd**  | `H::R`          | `H::R`           |
    pub fn map_drop<EH: Hazard<R = H::R>>(self, f: impl Fn(H::P) -> EH::P) -> I<EH, { Dep::Demanding }> {
        self.filter_map_drop_with_r(|p, _| Some(f(p)))
    }
}
