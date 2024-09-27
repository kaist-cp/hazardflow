//! Fork.

use super::*;

impl<P: Copy, R1: Copy, R2: Copy, const D: Dep> I<ValidH<P, (R1, R2)>, D> {
    /// Forks into two `ValidH` hazard interfaces.
    ///
    /// - Payload: Duplicated to multiple interfaces.
    /// - Resolvers: Preserved, and combined into one interface.
    ///
    /// | Interface | Ingress      | Egress                     |
    /// | :-------: | ------------ | -------------------------- |
    /// |  **Fwd**  | `HOption<P>` | `(HOption<P>, HOption<P>)` |
    /// |  **Bwd**  | `(R1, R2)`   | `(R1, R2)`                 |
    #[allow(clippy::type_complexity)]
    pub fn lfork(self) -> (I<ValidH<P, R1>, D>, I<ValidH<P, R2>, D>) {
        unsafe { Interface::fsm(self, (), |ip, er, s| ((ip, ip), er, s)) }
    }
}

macro_rules! impl_i_valid_h_lfork {
    ($($R:ident),+) => {
        impl<P: Copy, $($R: Copy,)+ const D: Dep> I<ValidH<P, ($($R,)+)>, D> {
            /// A variation of [`lfork`](fork) to 3-12 `ValidH` hazard interfaces. See the 2-tuple version for more
            /// information.
            #[allow(clippy::type_complexity)]
            pub fn lfork(self) -> ($(I<ValidH<P, $R>, D>,)+) {
                unsafe { Interface::fsm(self, (), |ip, er, s| (($(replace!($R, ip),)+), er, s)) }
            }
        }
    };
}

impl_i_valid_h_lfork! { R1, R2, R3 }
impl_i_valid_h_lfork! { R1, R2, R3, R4 }
impl_i_valid_h_lfork! { R1, R2, R3, R4, R5 }
impl_i_valid_h_lfork! { R1, R2, R3, R4, R5, R6 }
impl_i_valid_h_lfork! { R1, R2, R3, R4, R5, R6, R7 }
impl_i_valid_h_lfork! { R1, R2, R3, R4, R5, R6, R7, R8 }
impl_i_valid_h_lfork! { R1, R2, R3, R4, R5, R6, R7, R8, R9 }
impl_i_valid_h_lfork! { R1, R2, R3, R4, R5, R6, R7, R8, R9, R10 }
impl_i_valid_h_lfork! { R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11 }
impl_i_valid_h_lfork! { R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12 }

impl<P: Copy, R: Copy, const N: usize, const D: Dep> I<ValidH<P, Array<R, N>>, D> {
    /// Forks into `N` `ValidH` hazard interfaces.
    ///
    /// - Payload: Duplicated to multiple interfaces.
    /// - Resolvers: Preserved, and combined into one interface.
    ///
    /// | Interface | Ingress       | Egress                 |
    /// | :-------: | ------------- | ---------------------- |
    /// |  **Fwd**  | `HOption<P>`  | `Array<HOption<P>, N>` |
    /// |  **Bwd**  | `Array<R, N>` | `Array<R, N>`          |
    pub fn lfork(self) -> [I<ValidH<P, R>, D>; N] {
        unsafe { Interface::fsm(self, (), |ip, er, s| (ip.repeat::<N>(), er, s)) }
    }
}

impl<P: Copy> Valid<P> {
    /// A variation of [`lfork`](fork) for a valid interface, that has the correct resolver type.
    ///
    /// - Payload: Duplicated to multiple interfaces.
    /// - Resolvers: The resolvers carry no information.
    ///
    /// | Interface | Ingress      | Egress                     |
    /// | :-------: | ------------ | -------------------------- |
    /// |  **Fwd**  | `HOption<P>` | `(HOption<P>, HOption<P>)` |
    /// |  **Bwd**  | `()`         | `((), ())`                 |
    pub fn lfork(self) -> (Valid<P>, Valid<P>) {
        self.map_resolver::<((), ())>(|_| ()).lfork()
    }
}

impl<P: Copy, R1: Copy, R2: Copy, const D: Dep> I<VrH<P, (R1, R2)>, D> {
    /// Lazy-forks into two `VrH` hazard interfaces.
    ///
    /// An ingress transfer and all egress transfers happen at once when the ingress payload is valid and all the egress
    /// ready signals are true.
    ///
    /// - Payload: All the egress payloads become available at once when all the egress ready signals are true. Note
    ///     that In the actual implementation, each egress payload does not check its own interface's ready signal. This
    ///     is fine since a transfer would not happen if the ready signal is false. It's to allow the returned
    ///     interfaces to be [`Dep::Helpful`]. The payload value `P` is duplicated to multiple interfaces.
    /// - Resolvers: The ingress ready signal is true if all the egress ready signals are true. The inner values `R1`,
    ///     `R2` of the resolvers are preserved, and combined into one interface.
    ///
    /// | Interface | Ingress           | Egress                     |
    /// | :-------: | ----------------- | -------------------------- |
    /// |  **Fwd**  | `HOption<P>`      | `(HOption<P>, HOption<P>)` |
    /// |  **Bwd**  | `Ready<(R1, R2)>` | `(Ready<R1>, Ready<R2>)`   |
    #[allow(clippy::type_complexity)]
    pub fn lfork(self) -> (I<VrH<P, R1>, D>, I<VrH<P, R2>, D>) {
        unsafe {
            Interface::fsm::<(I<VrH<P, R1>, D>, I<VrH<P, R2>, D>), ()>(self, (), |ip, (er1, er2), s| {
                let ep1 = if er2.ready { ip } else { None };
                let ep2 = if er1.ready { ip } else { None };
                let ir = Ready::new(er1.ready && er2.ready, (er1.inner, er2.inner));

                ((ep1, ep2), ir, s)
            })
        }
    }
}

// TODO: Add 4 to 12-tuple variants.
impl<P: Copy, R1: Copy, R2: Copy, R3: Copy, const D: Dep> I<VrH<P, (R1, R2, R3)>, D> {
    /// A variation of [`lfork`](fork) to 3 `VrH` hazard interfaces. See the 2-tuple version for more information.
    #[allow(clippy::type_complexity)]
    pub fn lfork(self) -> (I<VrH<P, R1>, D>, I<VrH<P, R2>, D>, I<VrH<P, R3>, D>) {
        unsafe {
            Interface::fsm::<(I<VrH<P, R1>, D>, I<VrH<P, R2>, D>, I<VrH<P, R3>, D>), ()>(
                self,
                (),
                |ip, (er1, er2, er3), s| {
                    let ep1 = if er2.ready && er3.ready { ip } else { None };
                    let ep2 = if er1.ready && er3.ready { ip } else { None };
                    let ep3 = if er1.ready && er2.ready { ip } else { None };
                    let ir = Ready::new(er1.ready && er2.ready && er3.ready, (er1.inner, er2.inner, er3.inner));

                    ((ep1, ep2, ep3), ir, s)
                },
            )
        }
    }
}

impl<P: Copy, R: Copy, const N: usize, const D: Dep> I<VrH<P, Array<R, N>>, D> {
    /// Lazy-forks into `N` `VrH` hazard interfaces.
    ///
    /// An ingress transfer and all egress transfers happen at once when the ingress payload is valid and all the egress
    /// ready signals are true.
    ///
    /// - Payload: All the egress payloads become available at once when all the egress ready signals are true. The
    ///     payload value `P` is duplicated to mulitple interfaces.
    /// - Resolvers: The ingress ready signal is true if all the egress ready signals are true. The inner values `R` of
    ///     the resolvers are preserved, and combined into one interface.
    ///
    /// | Interface | Ingress              | Egress                 |
    /// | :-------: | -------------------- | ---------------------- |
    /// |  **Fwd**  | `HOption<P>`         | `Array<HOption<P>, N>` |
    /// |  **Bwd**  | `Ready<Array<R, N>>` | `Array<Ready<R>, N>`   |
    // TODO: We may want to use `D` instead of `Demanding`. In that case, `i`-th egress payload signal should not look at `i`-th egress resolver signal.
    pub fn lfork(self) -> [I<VrH<P, R>, { Dep::Demanding }>; N] {
        unsafe {
            Interface::fsm::<[I<VrH<P, R>, { Dep::Demanding }>; N], ()>(self, (), |ip, er, s| {
                let ir = Ready::new(er.all(|r| r.ready), er.map(|r| r.inner));
                let ep = ip.filter(|_| ir.ready).repeat::<N>();

                (ep, ir, s)
            })
        }
    }
}

impl<P: Copy, const D: Dep> Vr<P, D> {
    /// A variation of [`lfork`](fork) for a valid-ready interface, that has the correct resolver type.
    ///
    /// - Payload: All the egress payloads become available at once when all the egress ready signals are true. The
    ///     payload value `P` is duplicated to mulitple interfaces.
    /// - Resolvers: The ingress ready signal is true if all the egress ready signals are true.
    ///
    /// | Interface | Ingress      | Egress                     |
    /// | :-------: | ------------ | -------------------------- |
    /// |  **Fwd**  | `HOption<P>` | `(HOption<P>, HOption<P>)` |
    /// |  **Bwd**  | `Ready<()>`  | `(Ready<()>, Ready<()>)`   |
    pub fn lfork(self) -> (Vr<P, D>, Vr<P, D>) {
        self.map_resolver_inner::<((), ())>(|_| ()).lfork()
    }
}

impl<const D: Dep, H: Hazard> I<H, D> {
    /// Lazy-forks a hazard interface unidirectionally.
    ///
    /// - Payload: The payload for `I<H, D>` is preserved. The payload for `Valid<H::P>` is valid if a transfer for
    ///     `I<H, D>` happens.
    /// - Resolvers: The resolver is preserved through `I<H, D>`.
    ///
    /// | Interface | Ingress         | Egress                           |
    /// | :-------: | --------------- | -------------------------------- |
    /// |  **Fwd**  | `HOption<H::P>` | `(HOption<H::P>, HOption<H::P>)` |
    /// |  **Bwd**  | `H::R`          | `(H::R, ())`                     |
    pub fn lfork_uni(self) -> (Self, Valid<H::P>) {
        unsafe {
            Interface::fsm::<(Self, Valid<H::P>), ()>(self, (), |ip, (er, _), s| {
                let ep1 = ip;
                let ep2 = ip.filter(|p| H::ready(p, er));
                let ir = er;
                ((ep1, ep2), ir, s)
            })
        }
    }

    /// Forks the egress resolver to an egress payload.
    ///
    /// - Payload: The payload for `I<H, D>` is preserved. The payload for `Valid<H::R>` is from the egress resolver of
    ///     `I<H, D>`, and is always valid.
    /// - Resolvers: The resolver is preserved through `I<H, D>`, and forked to the payload of `Valid<H::R>`.
    ///
    /// | Interface | Ingress         | Egress                           |
    /// | :-------: | --------------- | -------------------------------- |
    /// |  **Fwd**  | `HOption<H::P>` | `(HOption<H::P>, HOption<H::R>)` |
    /// |  **Bwd**  | `H::R`          | `(H::R, ())`                     |
    pub fn fork_r_to_p(self) -> (Self, Valid<H::R>) {
        unsafe {
            Interface::fsm::<(Self, Valid<H::R>), ()>(self, (), |ip, (er, _), s| {
                let ep1 = ip;
                let ep2 = Some(er);
                let ir = er;
                ((ep1, ep2), ir, s)
            })
        }
    }
}
