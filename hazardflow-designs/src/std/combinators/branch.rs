//! Branch.

use super::*;

impl<P: Copy, R1: Copy, R2: Copy, const D: Dep> I<ValidH<(P, BoundedU<2>), (R1, R2)>, D> {
    /// Branches into two `ValidH` hazard interfaces based on the selector.
    ///
    /// The selector chooses which egress interface to connect to the ingress interface.
    ///
    /// - Payload: Only the selected interface's payload will be valid.
    /// - Resolvers: Preserved, and combined into one interface.
    ///
    /// | Interface | Ingress      | Egress                     |
    /// | :-------: | ------------ | -------------------------- |
    /// |  **Fwd**  | `HOption<P>` | `(HOption<P>, HOption<P>)` |
    /// |  **Bwd**  | `(R1, R2)`   | `(R1, R2)`                 |
    #[allow(clippy::type_complexity)]
    pub fn branch(self) -> (I<ValidH<P, R1>, D>, I<ValidH<P, R2>, D>) {
        unsafe {
            Interface::fsm(self, (), |ip, er, s| {
                let ep = if let Some((ip, sel)) = ip {
                    let ep1 = if sel.value() == 0.into_u() { Some(ip) } else { None };
                    let ep2 = if sel.value() == 1.into_u() { Some(ip) } else { None };
                    (ep1, ep2)
                } else {
                    (None, None)
                };
                (ep, er, s)
            })
        }
    }
}

macro_rules! impl_i_valid_h_branch {
    ($($R:ident),+; $N:literal; $($value:expr),+) => {
        impl<P: Copy, $($R: Copy,)+ const D: Dep> I<ValidH<(P, BoundedU<$N>), ($($R,)+)>, D> {
            /// A variation of [`branch`] to 3-12 `ValidH` hazard interfaces. See the 2-tuple version for more
            /// information.
            #[allow(clippy::type_complexity)]
            pub fn branch(self) -> ($(I<ValidH<P, $R>, D>,)+) {
                unsafe {
                    Interface::fsm(self, (), |ip, er, s| {
                        let ep = if let Some((ip, sel)) = ip {
                            ($(if sel.value() == $value.into_u() { Some(ip) } else { None },)+)
                        } else {
                            ($(replace!($value, None),)+)
                        };
                        (ep, er, s)
                    })
                }
            }
        }
    };
}

impl_i_valid_h_branch! { R1, R2, R3; 3; 0, 1, 2 }
impl_i_valid_h_branch! { R1, R2, R3, R4; 4; 0, 1, 2, 3 }
impl_i_valid_h_branch! { R1, R2, R3, R4, R5; 5; 0, 1, 2, 3, 4 }
impl_i_valid_h_branch! { R1, R2, R3, R4, R5, R6; 6; 0, 1, 2, 3, 4, 5 }
impl_i_valid_h_branch! { R1, R2, R3, R4, R5, R6, R7; 7; 0, 1, 2, 3, 4, 5, 6 }
impl_i_valid_h_branch! { R1, R2, R3, R4, R5, R6, R7, R8; 8; 0, 1, 2, 3, 4, 5, 6, 7 }
impl_i_valid_h_branch! { R1, R2, R3, R4, R5, R6, R7, R8, R9; 9; 0, 1, 2, 3, 4, 5, 6, 7, 8 }
impl_i_valid_h_branch! { R1, R2, R3, R4, R5, R6, R7, R8, R9, R10; 10; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 }
impl_i_valid_h_branch! { R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11; 11; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 }
impl_i_valid_h_branch! { R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12; 12; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 }

impl<P: Copy, R: Copy, const N: usize, const D: Dep> I<ValidH<(P, BoundedU<N>), Array<R, N>>, D>
where [(); clog2(N)]:
{
    /// Branches into `N` `ValidH` hazard interfaces based on the selector.
    ///
    /// The selector chooses which egress interface to connect to the ingress interface.
    ///
    /// - Payload: Only the selected interface's payload will be valid.
    /// - Resolvers: Preserved, and combined into one interface.
    ///
    /// | Interface | Ingress       | Egress                 |
    /// | :-------: | ------------- | ---------------------- |
    /// |  **Fwd**  | `HOption<P>`  | `Array<HOption<P>, N>` |
    /// |  **Bwd**  | `Array<R, N>` | `Array<R, N>`          |
    pub fn branch(self) -> [I<ValidH<P, R>, D>; N] {
        unsafe {
            Interface::fsm(self, (), |ip, er, s| {
                let ep = if let Some((p, index)) = ip {
                    None.repeat::<N>().set(index.value(), Some(p))
                } else {
                    None.repeat()
                };
                (ep, er, s)
            })
        }
    }
}

impl<P: Copy, const N: usize> Valid<(P, BoundedU<N>)>
where [(); clog2(N)]:
{
    /// A variation of [`branch`] for a valid interface, that has the correct resolver type.
    ///
    /// - Payload: Only the selected interface's payload will be valid.
    /// - Resolvers: The resolvers carry no information.
    ///
    /// | Interface | Ingress      | Egress                 |
    /// | :-------: | ------------ | ---------------------- |
    /// |  **Fwd**  | `HOption<P>` | `Array<HOption<P>, N>` |
    /// |  **Bwd**  | `()`         | `Array<(), N>`         |
    pub fn branch(self) -> [Valid<P>; N] {
        self.map_resolver::<Array<(), N>>(|_| ()).branch()
    }
}

impl<P: Copy, R1: Copy, R2: Copy> I<VrH<(P, BoundedU<2>), (R1, R2)>, { Dep::Helpful }> {
    /// Branches into two `VrH` hazard interfaces based on the selector.
    ///
    /// The selector chooses which egress interface to connect to the ingress interface.
    ///
    /// - Payload: Only the selected interface's payload will be valid.
    /// - Resolvers: The ingress ready signal follows the selected interface's ready signal. If the selector is not
    ///     valid, the ingress ready signal is true. The inner values `R1`, `R2` of the resolvers are preserved, and
    ///     combined into one interface.
    ///
    /// | Interface | Ingress           | Egress                     |
    /// | :-------: | ----------------- | -------------------------- |
    /// |  **Fwd**  | `HOption<P>`      | `(HOption<P>, HOption<P>)` |
    /// |  **Bwd**  | `Ready<(R1, R2)>` | `(Ready<R1>, Ready<R2>)`   |
    #[allow(clippy::type_complexity)]
    pub fn branch(self) -> (I<VrH<P, R1>, { Dep::Helpful }>, I<VrH<P, R2>, { Dep::Helpful }>) {
        unsafe {
            Interface::fsm::<(I<VrH<P, R1>, { Dep::Helpful }>, I<VrH<P, R2>, { Dep::Helpful }>), ()>(
                self,
                (),
                |ip, (er1, er2), s| {
                    let Some((ip, sel)) = ip else {
                        // Ingress ready signal is true when valid signal is false.
                        return ((None, None), Ready::new(true, (er1.inner, er2.inner)), s);
                    };

                    let ep1 = if sel.value() == 0.into_u() { Some(ip) } else { None };
                    let ep2 = if sel.value() == 1.into_u() { Some(ip) } else { None };
                    let ir = Ready::new(U::from([er1.ready, er2.ready])[sel.value()], (er1.inner, er2.inner));

                    ((ep1, ep2), ir, s)
                },
            )
        }
    }
}

macro_rules! impl_i_vr_h_branch {
    ($($R:ident),+; $N:literal; $($index:tt),+) => {
        impl<P: Copy, $($R: Copy,)+> I<VrH<(P, BoundedU<$N>), ($($R,)+)>, { Dep::Helpful }> {
            /// A variation of [`branch`] to 3-12 `VrH` hazard interfaces. See the 2-tuple version for more information.
            #[allow(clippy::type_complexity)]
            pub fn branch(self) -> ($(I<VrH<P, $R>, { Dep::Helpful }>,)+) {
                unsafe {
                    Interface::fsm::<($(I<VrH<P, $R>, { Dep::Helpful }>,)+), ()>(self, (), |ip, er, s| {
                        let Some((ip, sel)) = ip else {
                            // Ingress ready signal is true when valid signal is false.
                            return (($(replace!($index, None),)+), Ready::new(true, ($(er.$index.inner,)+)), s);
                        };

                        let ep = ($(if sel.value() == $index.into_u() { Some(ip) } else { None },)+);
                        let ir = Ready::new(U::from([$(er.$index.ready,)+])[sel.value()], ($(er.$index.inner,)+));

                        (ep, ir, s)
                    })
                }
            }
        }
    };
}

impl_i_vr_h_branch! { R1, R2, R3; 3; 0, 1, 2 }
impl_i_vr_h_branch! { R1, R2, R3, R4; 4; 0, 1, 2, 3 }
impl_i_vr_h_branch! { R1, R2, R3, R4, R5; 5; 0, 1, 2, 3, 4 }
impl_i_vr_h_branch! { R1, R2, R3, R4, R5, R6; 6; 0, 1, 2, 3, 4, 5 }
impl_i_vr_h_branch! { R1, R2, R3, R4, R5, R6, R7; 7; 0, 1, 2, 3, 4, 5, 6 }
impl_i_vr_h_branch! { R1, R2, R3, R4, R5, R6, R7, R8; 8; 0, 1, 2, 3, 4, 5, 6, 7 }
impl_i_vr_h_branch! { R1, R2, R3, R4, R5, R6, R7, R8, R9; 9; 0, 1, 2, 3, 4, 5, 6, 7, 8 }
impl_i_vr_h_branch! { R1, R2, R3, R4, R5, R6, R7, R8, R9, R10; 10; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 }
impl_i_vr_h_branch! { R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11; 11; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 }
impl_i_vr_h_branch! { R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12; 12; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 }

impl<P: Copy, R: Copy, const N: usize> I<VrH<(P, BoundedU<N>), Array<R, N>>, { Dep::Helpful }>
where [(); clog2(N)]:
{
    /// Branches into `N` `VrH` hazard interfaces based on the selector.
    ///
    /// The selector chooses which egress interface to connect to the ingress interface.
    ///
    /// - Payload: Only the selected interface's payload will be valid.
    /// - Resolvers: The ingress ready signal follows the selected interface's ready signal. If the selector is not
    ///     valid, the ingress ready signal is true. The inner values `R` of the resolvers are preserved, and combined
    ///     into one interface.
    ///
    /// | Interface | Ingress              | Egress                 |
    /// | :-------: | -------------------- | ---------------------- |
    /// |  **Fwd**  | `HOption<P>`         | `Array<HOption<P>, N>` |
    /// |  **Bwd**  | `Ready<Array<R, N>>` | `Array<Ready<R>, N>`   |
    pub fn branch(self) -> [I<VrH<P, R>, { Dep::Helpful }>; N] {
        unsafe {
            Interface::fsm::<[I<VrH<P, R>, { Dep::Helpful }>; N], ()>(self, (), |ip, er, s| {
                let Some((ip, sel)) = ip else {
                    // Ingress ready signal is true when valid signal is false.
                    return (None.repeat::<N>(), Ready::new(true, er.map(|r| r.inner)), s);
                };

                let ep = None.repeat::<N>().set(sel.value(), Some(ip));
                let ir = Ready::new(er[sel.value()].ready, er.map(|r| r.inner));

                (ep, ir, s)
            })
        }
    }
}

impl<P: Copy, const N: usize> Vr<(P, BoundedU<N>)>
where [(); clog2(N)]:
{
    /// A variation of [`branch`] for a valid-ready interface, that has the correct resolver type.
    ///
    /// - Payload: Only the selected interface's payload will be valid.
    /// - Resolvers: The ingress ready signal follows the selected interface's ready signal. If the selector is not
    ///     valid, the ingress ready signal is true.
    ///
    /// | Interface | Ingress      | Egress                 |
    /// | :-------: | ------------ | ---------------------- |
    /// |  **Fwd**  | `HOption<P>` | `Array<HOption<P>, N>` |
    /// |  **Bwd**  | `Ready<()>`  | `Array<Ready<()>, N>`  |
    pub fn branch(self) -> [Vr<P>; N] {
        self.map_resolver::<Array<(), N>>(|_| ()).branch()
    }
}
