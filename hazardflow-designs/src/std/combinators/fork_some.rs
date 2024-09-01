//! Fork some.

use super::*;

impl<P: Copy, R1: Copy, R2: Copy, const D: Dep> I<VrH<P, (R1, R2)>, D> {
    /// Forks into some of the two `VrH` hazard interfaces.
    ///
    /// An ingress transfer and egress transfers happen as soon as when the ingress payload is valid and at least one of
    /// the egress ready signals is true. Note that the egress transfers happen only for the ready egress interfaces.
    ///
    /// - Payload: Each egress payload becomes available when its own egress ready signal is true. The payload value `P`
    ///     is duplicated to multiple interfaces.
    /// - Resolvers: The ingress ready signal is true if any of the egress ready signals are true. The inner values
    ///     `R1`, `R2` of the resolvers are preserved, and combined into one interface.
    ///
    /// | Interface | Ingress           | Egress                     |
    /// | :-------: | ----------------- | -------------------------- |
    /// |  **Fwd**  | `HOption<P>`      | `(HOption<P>, HOption<P>)` |
    /// |  **Bwd**  | `Ready<(R1, R2)>` | `(Ready<R1>, Ready<R2>)`   |
    #[allow(clippy::type_complexity)]
    pub fn fork_some(self) -> (I<VrH<P, R1>, { Dep::Demanding }>, I<VrH<P, R2>, { Dep::Demanding }>) {
        unsafe {
            Interface::fsm(self, (), |ip, er: (Ready<R1>, Ready<R2>), ()| {
                let ep = match ip {
                    Some(p) => {
                        let ep0 = if er.0.ready { Some(p) } else { None };
                        let ep1 = if er.1.ready { Some(p) } else { None };
                        (ep0, ep1)
                    }
                    None => (None, None),
                };
                let ir = Ready::new(er.0.ready || er.1.ready, (er.0.inner, er.1.inner));
                (ep, ir, ())
            })
        }
    }
}

macro_rules! impl_i_vr_h_fork_some {
    ($($R:ident),+; $($index:tt),+) => {
        impl<P: Copy, $($R: Copy,)+ const D: Dep> I<VrH<P, ($($R,)+)>, D> {
            /// A variation of [`fork_some`] to 3-12 `VrH` hazard interfaces. See the 2-tuple version for more
            /// information.
            #[allow(clippy::type_complexity)]
            pub fn fork_some(self) -> ($(I<VrH<P, $R>, { Dep::Demanding }>,)+) {
                unsafe {
                    Interface::fsm(self, (), |ip, er: ($(Ready<$R>,)+), ()| {
                        let ep = match ip {
                            Some(p) => ($(if er.$index.ready { Some(p) } else { None },)+),
                            None => ($(replace!($index, None),)+),
                        };
                        let ir = Ready::new($(er.$index.ready)||+, ($(er.$index.inner,)+));
                        (ep, ir, ())
                    })
                }
            }
        }
    };
}

impl_i_vr_h_fork_some! { R1, R2, R3; 0, 1, 2 }
impl_i_vr_h_fork_some! { R1, R2, R3, R4; 0, 1, 2, 3 }
impl_i_vr_h_fork_some! { R1, R2, R3, R4, R5; 0, 1, 2, 3, 4 }
impl_i_vr_h_fork_some! { R1, R2, R3, R4, R5, R6; 0, 1, 2, 3, 4, 5 }
impl_i_vr_h_fork_some! { R1, R2, R3, R4, R5, R6, R7; 0, 1, 2, 3, 4, 5, 6 }
impl_i_vr_h_fork_some! { R1, R2, R3, R4, R5, R6, R7, R8; 0, 1, 2, 3, 4, 5, 6, 7 }
impl_i_vr_h_fork_some! { R1, R2, R3, R4, R5, R6, R7, R8, R9; 0, 1, 2, 3, 4, 5, 6, 7, 8 }
impl_i_vr_h_fork_some! { R1, R2, R3, R4, R5, R6, R7, R8, R9, R10; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 }
impl_i_vr_h_fork_some! { R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 }
impl_i_vr_h_fork_some! { R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 }

impl<P: Copy, R: Copy, const N: usize, const D: Dep> I<VrH<P, Array<R, N>>, D> {
    /// Forks into some of the `N` `VrH` hazard interfaces.
    ///
    /// An ingress transfer and egress transfers happen as soon as when the ingress payload is valid and at least one of
    /// the egress ready signals are true. Note that the egress transfers happen only for the ready egress interfaces.
    ///
    /// - Payload: Each egress payload becomes available when its own egress ready signal is true. The payload value `P`
    ///     is duplicated to multiple interfaces.
    /// - Resolvers: The ingress ready signal is true if any of the egress ready signals are true. The inner values
    ///     `R` of the resolvers are preserved, and combined into one interface.
    ///
    /// | Interface | Ingress              | Egress                 |
    /// | :-------: | -------------------- | ---------------------- |
    /// |  **Fwd**  | `HOption<P>`         | `Array<HOption<P>, N>` |
    /// |  **Bwd**  | `Ready<Array<R, N>>` | `Array<Ready<R>, N>`   |
    pub fn fork_some(self) -> [I<VrH<P, R>, { Dep::Demanding }>; N] {
        unsafe {
            Interface::fsm(self, (), |ip, er: Array<Ready<R>, N>, ()| {
                let ep = match ip {
                    Some(p) => er.map(|r| if r.ready { Some(p) } else { None }),
                    None => None.repeat(),
                };
                let ir = Ready::new(er.any(|r| r.ready), er.map(|r| r.inner));
                (ep, ir, ())
            })
        }
    }
}

impl<P: Copy, const D: Dep> Vr<P, D> {
    /// A variation of [`fork_some`] for a valid-ready interface, that has the correct resolver type.
    ///
    /// - Payload: Each egress payload becomes available when its own egress ready signal is true. The payload value `P`
    ///     is duplicated to multiple interfaces.
    /// - Resolvers: The ingress ready signal is true if any of the egress ready signals are true.
    ///
    /// | Interface | Ingress      | Egress                     |
    /// | :-------: | ------------ | -------------------------- |
    /// |  **Fwd**  | `HOption<P>` | `(HOption<P>, HOption<P>)` |
    /// |  **Bwd**  | `Ready<()>`  | `(Ready<()>, Ready<()>)`   |
    pub fn fork_some(self) -> (Vr<P, { Dep::Demanding }>, Vr<P, { Dep::Demanding }>) {
        self.map_resolver_inner::<((), ())>(|_| ()).fork_some()
    }
}
