//! Map resolver.

use super::*;

impl<P: Copy, R: Copy, const D: Dep> I<ValidH<P, R>, D> {
    /// Maps the egress resolver into the ingress resolver.
    ///
    /// - Payload: Preserved.
    /// - Resolver: Mapped by `f`.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `R`          | `ER`         |
    pub fn map_resolver<ER: Copy>(self, f: impl Fn(ER) -> R) -> I<ValidH<P, ER>, D> {
        unsafe { self.fsm::<(), D, ValidH<P, ER>>((), |ip, er, s| (ip, f(er), s)) }
    }
}

impl<P: Copy, R: Copy> I<ValidH<P, R>, { Dep::Helpful }> {
    /// A variation of [`map_resolver`] that allows `f` to consider the ingress payload in addition to the egress
    /// resolver while calculating the ingress resolver.
    ///
    /// This is possible because the ingress interface is [`Dep::Helpful`].
    ///
    /// - Payload: Preserved.
    /// - Resolver: Mapped by `f`.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `R`          | `ER`         |
    pub fn map_resolver_with_p<ER: Copy>(self, f: impl Fn(HOption<P>, ER) -> R) -> I<ValidH<P, ER>, { Dep::Helpful }> {
        unsafe { self.fsm::<(), { Dep::Helpful }, ValidH<P, ER>>((), |ip, er, s| (ip, f(ip, er), s)) }
    }
}

impl<P: Copy, R: Copy, const D: Dep> I<VrH<P, R>, D> {
    /// Maps the egress resolver into the ingress resolver.
    ///
    /// - Payload: Preserved.
    /// - Resolver: The egress resolver `Ready<ER>` is mapped to the inner value `R` of the ingress resolver by `f`.
    ///     Note that this disallows changing the ready signal.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<ER>`  |
    pub fn map_resolver<ER: Copy>(self, f: impl Fn(Ready<ER>) -> R) -> I<VrH<P, ER>, D> {
        unsafe { self.fsm::<(), D, VrH<P, ER>>((), |ip, er, s| (ip, Ready::new(er.ready, f(er)), s)) }
    }

    /// A variation of [`map_resolver`] that does not use the egress ready signal.
    ///
    /// - Payload: Preserved.
    /// - Resolver: The inner value `ER` of the egress resolver is mapped into the inner value `R` of the ingress
    ///     resolver by `f`.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<ER>`  |
    pub fn map_resolver_inner<ER: Copy>(self, f: impl Fn(ER) -> R) -> I<VrH<P, ER>, D> {
        self.map_resolver(|er| f(er.inner))
    }
}

impl<P: Copy, R: Copy> I<VrH<P, R>, { Dep::Helpful }> {
    /// A variation of [`map_resolver`] that allows `f` to consider the ingress payload in addition to the egress
    /// resolver while calculating the ingress resolver.
    ///
    /// This is possible because the ingress interface is [`Dep::Helpful`].
    ///
    /// - Payload: Preserved.
    /// - Resolver: The egress resolver `Ready<ER>` is mapped to the inner value `R` of the ingress resolver by `f`.
    ///     Note that this disallows changing the ready signal.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<ER>`  |
    pub fn map_resolver_with_p<ER: Copy>(
        self,
        f: impl Fn(HOption<P>, Ready<ER>) -> R,
    ) -> I<VrH<P, ER>, { Dep::Helpful }> {
        unsafe {
            self.fsm::<(), { Dep::Helpful }, VrH<P, ER>>((), |ip, er, s| (ip, Ready::new(er.ready, f(ip, er)), s))
        }
    }
}

impl<H: Hazard, const D: Dep> I<H, D> {
    /// Maps the egress resolver into the ingress resolver.
    ///
    /// - Payload: Droppped if `H::ready(ip, ir)` or `EH::ready(ep, er)` is false.
    ///     ([why?](super#notes-on-dropping-combinators))
    /// - Resolver: Mapped by `f`.
    ///
    /// | Interface | Ingress         | Egress          |
    /// | :-------: | --------------- | --------------- |
    /// |  **Fwd**  | `HOption<H::P>` | `HOption<H::P>` |
    /// |  **Bwd**  | `H::R`          | `EH::R`         |
    pub fn map_resolver_drop<EH: Hazard<P = H::P>>(self, f: impl Fn(EH::R) -> H::R) -> I<EH, { Dep::Demanding }> {
        unsafe {
            self.fsm::<(), { Dep::Demanding }, EH>((), |ip, er, s| {
                let ir = f(er);
                let ep = if ip.is_some_and(|p| H::ready(p, ir) && EH::ready(p, er)) { ip } else { None };
                (ep, ir, s)
            })
        }
    }
}

impl<H: Hazard> I<H, { Dep::Helpful }> {
    /// A variation of [`I::map_resolver_drop`] that allows `f` to consider the ingress payload in addition to the
    /// egress resolver while calculating the ingress resolver.
    ///
    /// This is possible because the ingress interface is [`Dep::Helpful`].
    ///
    /// - Payload: The same behavior as [`I::map_resolver_drop`]
    /// - Resolver: The same behavior as [`I::map_resolver_drop`]
    ///
    /// | Interface | Ingress         | Egress          |
    /// | :-------: | --------------- | --------------- |
    /// |  **Fwd**  | `HOption<H::P>` | `HOption<H::P>` |
    /// |  **Bwd**  | `H::R`          | `EH::R`         |
    pub fn map_resolver_drop_with_p<EH: Hazard<P = H::P>>(
        self,
        f: impl Fn(HOption<H::P>, EH::R) -> H::R,
    ) -> I<EH, { Dep::Demanding }> {
        unsafe {
            self.fsm::<(), { Dep::Demanding }, EH>((), |ip, er, s| {
                let ir = f(ip, er);
                let ep = if ip.is_some_and(|p| H::ready(p, ir) && EH::ready(p, er)) { ip } else { None };
                (ep, ir, s)
            })
        }
    }
}

impl<H: Hazard> I<AndH<H>, { Dep::Helpful }> {
    /// Maps the egress resolver into the ingress resolver with an additional ready signal for blocking.
    ///
    /// - Payload: Preserved. Technically, the payload is dropped if `H::ready(ip, ir)` or `EH::ready(ep, er)` is false.
    ///     But since the added ready signal informs the ingress interface that a transfer did not happen (i.e. blocks
    ///     the transfer) in such a case, no payload will be lost.
    /// - Resolver: An additional ready signal is attached to the ingress resolver, which will be turned off if the
    ///     egress ready condition `EH::ready(ep, er)` is false. The egress resolver `EH::R` is mapped to the inner
    ///     ingress resolver `H::R` by `f`.
    ///
    /// | Interface | Ingress         | Egress          |
    /// | :-------: | --------------- | --------------- |
    /// |  **Fwd**  | `HOption<H::P>` | `HOption<H::P>` |
    /// |  **Bwd**  | `Ready<H::R>`   | `EH::R`         |
    pub fn map_resolver_block<EH: Hazard<P = H::P>>(self, f: impl Fn(EH::R) -> H::R) -> I<EH, { Dep::Demanding }> {
        self.map_resolver_block_with_p(|_, er| f(er))
    }

    /// A variation of [`I::map_resolver_block`] that allows `f` to consider the ingress payload in addition to the
    /// egress resolver while calculating the ingress resolver.
    ///
    /// This is possible because the ingress interface is [`Dep::Helpful`].
    ///
    /// - Payload: The same behavior as [`I::map_resolver_block`].
    /// - Resolver: The same behavior as [`I::map_resolver_block`].
    ///
    /// | Interface | Ingress         | Egress          |
    /// | :-------: | --------------- | --------------- |
    /// |  **Fwd**  | `HOption<H::P>` | `HOption<H::P>` |
    /// |  **Bwd**  | `Ready<H::R>`   | `EH::R`         |
    pub fn map_resolver_block_with_p<EH: Hazard<P = H::P>>(
        self,
        f: impl Fn(HOption<H::P>, EH::R) -> H::R,
    ) -> I<EH, { Dep::Demanding }> {
        unsafe {
            self.fsm::<(), { Dep::Demanding }, EH>((), |ip, er, s| {
                let ir_inner = f(ip, er);
                let xfer = ip.is_some_and(|p| H::ready(p, ir_inner) && EH::ready(p, er));
                let ir = Ready::new(xfer, ir_inner);
                let ep = if xfer { ip } else { None };
                (ep, ir, s)
            })
        }
    }
}
