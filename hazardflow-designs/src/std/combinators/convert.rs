//! Convert.

use super::*;

impl<P: Copy> I<ValidH<P, ()>, { Dep::Demanding }> {
    /// Converts the dependency type of a valid interface back into [`Dep::Helpful`].
    ///
    /// When applying some combinator, the dependency type of a valid interface could be changed to [`Dep::Demanding`].
    /// This function is used to convert the dependency type back to [`Dep::Helpful`].
    ///
    /// - Payload: Preserved.
    /// - Resolver: The resolver carries no information.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `()`         | `()`         |
    pub fn into_helpful(self) -> Valid<P> {
        // # Safety
        //
        // `Valid<P, D>` interface has unit(`()`) resolver signal by definition.
        // Hence, the `Valid` interface doesn't refer to the resolver signal to determine the valid signal, which is identical to `Dep::Helpful`.
        unsafe { self.fsm::<(), { Dep::Helpful }, ValidH<P, ()>>((), |ip, (), ()| (ip, (), ())) }
    }
}

impl<P: Copy, R: Copy> I<ValidH<P, R>, { Dep::Helpful }> {
    /// Converts a `ValidH` interface into a `VrH` interface, by allowing the payload to be discarded.
    ///
    /// Note that the ingress ready condition `ValidH::ready` is always `true`, but the egress ready condition
    /// `VrH::ready` is `er.ready`. This means if `er.ready` is false, the payload will be discarded/ignored by
    /// combinators after this one even if it is valid.
    ///
    /// - Payload: Preserved, but may be discarded by combinators after this one.
    /// - Resolver: The ready signal is stripped.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `R`          | `Ready<R>`   |
    pub fn discard_into_vr(self) -> I<VrH<P, R>, { Dep::Helpful }> {
        unsafe { self.fsm((), |ip, er: Ready<R>, ()| (ip, er.inner, ())) }
    }
}

impl<P: Copy, R: Copy, const D: Dep> I<VrH<P, R>, D> {
    /// Converts a `VrH` hazard interface into a `ValidH` hazard interface, by setting the ingress ready signal to be
    /// always true.
    ///
    /// - Payload: Preserved.
    /// - Resolver: Wrapped in an always ready [`Ready`].
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `Ready<R>`   | `R`          |
    pub fn always_into_valid(self) -> I<ValidH<P, R>, D> {
        unsafe { self.fsm((), |ip, er, ()| (ip, Ready::valid(er), ())) }
    }
}

impl<H: Hazard> I<H, { Dep::Helpful }> {
    /// Converts a [`Dep::Helpful`] hazard interface into a [`Dep::Demanding`] one, by dropping the payload if
    /// `H::ready` is false.
    ///
    /// - Payload: Dropped if `H::ready` is false.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress         | Egress          |
    /// | :-------: | --------------- | --------------- |
    /// |  **Fwd**  | `HOption<H::P>` | `HOption<H::P>` |
    /// |  **Bwd**  | `H::R`          | `H::R`          |
    pub fn drop_into_demanding(self) -> I<H, { Dep::Demanding }> {
        self.drop_into_hazard::<H>()
    }
}

impl<H: Hazard, const D: Dep> I<H, D> {
    /// Converts a hazard interface into another one with the same payload/resolver types.
    ///
    /// In effect, this changes the ready condition from the ingress side `H::ready` to the egress side `EH::ready`.
    ///
    /// - Payload: Dropped if `H::ready` or `EH::ready` is false.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress         | Egress          |
    /// | :-------: | --------------- | --------------- |
    /// |  **Fwd**  | `HOption<H::P>` | `HOption<H::P>` |
    /// |  **Bwd**  | `H::R`          | `H::R`          |
    pub fn drop_into_hazard<EH: Hazard<P = H::P, R = H::R>>(self) -> I<EH, { Dep::Demanding }> {
        self.map_drop(|p| p)
    }
}

impl<H: Hazard> I<AndH<H>, { Dep::Helpful }> {
    /// Converts a hazard interface wrapped in an `AndH` into another one with the same payload/resolver types.
    ///
    /// In effect, this changes the ready condition from the ingress side `<AndH<H>>::ready` to the egress side
    /// `EH::ready`.
    ///
    /// - Payload: Preserved.
    /// - Resolver: An additional ready signal is attached to the ingress resolver, which will be turned off to block
    ///     ingress transfers if the egress ready condition `EH::ready` is false. The egress resolver `H::R` is
    ///     preserved.
    ///
    /// | Interface | Ingress         | Egress          |
    /// | :-------: | --------------- | --------------- |
    /// |  **Fwd**  | `HOption<H::P>` | `HOption<H::P>` |
    /// |  **Bwd**  | `Ready<H::R>`   | `H::R`          |
    pub fn block_into_hazard<EH: Hazard<P = H::P, R = H::R>>(self) -> I<EH, { Dep::Demanding }> {
        self.map_resolver_block::<EH>(|er| er)
    }
}

impl<P: Copy> Vr<P> {
    /// A variation of [`I::block_into_hazard`] for a valid-ready interface that drops the resolver.
    ///
    /// - Payload: Preserved.
    /// - Resolver: The ingress ready signal will be turned off to block ingress transfers if the egress ready condition
    ///     `EH::ready` is false. The egress resolver `EH::R` is dropped.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `Ready<()>`  | `EH::R`      |
    pub fn block_into_hazard_vr<EH: Hazard<P = P>>(self) -> I<EH, { Dep::Demanding }> {
        self.map_resolver::<EH::R>(|_| ()).block_into_hazard::<EH>()
    }
}
