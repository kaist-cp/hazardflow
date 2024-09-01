//! Join.

use super::*;

/// Extension trait for `join`.
pub trait JoinExt: Interface {
    /// Egress interface.
    type E: Interface;

    /// Join.
    fn join(self) -> Self::E;
}

impl<P1: Copy, P2: Copy, R1: Copy, R2: Copy, const D: Dep> JoinExt for (I<ValidH<P1, R1>, D>, I<ValidH<P2, R2>, D>) {
    type E = I<ValidH<(P1, P2), (R1, R2)>, D>;

    /// Joins two `ValidH` hazard interfaces.
    ///
    /// - Payloads: Zipped to one interface. Note that a payload may get dropped if the other interface's payload is
    ///     `None`.
    /// - Resolver: Preserved, and split to multiple interfaces.
    ///
    /// | Interface | Ingress                      | Egress              |
    /// | :-------: | ---------------------------- | ------------------- |
    /// |  **Fwd**  | `(HOption<P1>, HOption<P2>)` | `HOption<(P1, P2)>` |
    /// |  **Bwd**  | `(R1, R2)`                   | `(R1, R2)`          |
    fn join(self) -> I<ValidH<(P1, P2), (R1, R2)>, D> {
        unsafe {
            self.fsm((), |(ip1, ip2), er, ()| {
                let ep = ip1.zip(ip2);
                (ep, er, ())
            })
        }
    }
}

macro_rules! impl_i_valid_h_join {
    ($($P:ident),+; $($R:ident),+) => {
        impl<$($P: Copy,)+ $($R: Copy,)+ const D: Dep> JoinExt for ($(I<ValidH<$P, $R>, D>,)+) {
            type E = I<ValidH<($($P,)+), ($($R,)+)>, D>;

            /// A variation of [`join`] for 3-12 `ValidH` hazard interfaces. See the 2-tuple version for more
            /// information.
            fn join(self) -> I<ValidH<($($P,)+), ($($R,)+)>, D> {
                unsafe {
                    self.fsm((), |ip, er, ()| {
                        // Equivalent to `zip` for `(HOption<P1>, HOption<P2>, ...)`.
                        let ep = match ip {
                            // This is a hack that uses `P1`, `P2`, ... as variable names.
                            #[allow(non_snake_case)]
                            ($(Some($P),)+) => Some(($($P,)+)),
                            _ => None,
                        };
                        (ep, er, ())
                    })
                }
            }
        }
    };
}

impl_i_valid_h_join! { P1, P2, P3; R1, R2, R3 }
impl_i_valid_h_join! { P1, P2, P3, P4; R1, R2, R3, R4 }
impl_i_valid_h_join! { P1, P2, P3, P4, P5; R1, R2, R3, R4, R5 }
impl_i_valid_h_join! { P1, P2, P3, P4, P5, P6; R1, R2, R3, R4, R5, R6 }
impl_i_valid_h_join! { P1, P2, P3, P4, P5, P6, P7; R1, R2, R3, R4, R5, R6, R7 }
impl_i_valid_h_join! { P1, P2, P3, P4, P5, P6, P7, P8; R1, R2, R3, R4, R5, R6, R7, R8 }
impl_i_valid_h_join! { P1, P2, P3, P4, P5, P6, P7, P8, P9; R1, R2, R3, R4, R5, R6, R7, R8, R9 }
impl_i_valid_h_join! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10 }
impl_i_valid_h_join! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11 }
impl_i_valid_h_join! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12 }

impl<P: Copy, R: Copy, const D: Dep, const N: usize> JoinExt for [I<ValidH<P, R>, D>; N] {
    type E = I<ValidH<Array<P, N>, Array<R, N>>, D>;

    /// Joins `N` `ValidH` hazard interfaces.
    ///
    /// - Payloads: Zipped to one interface. Note that a payload may get dropped if the other interface's payload is
    ///     `None`.
    /// - Resolver: Preserved, and split to multiple interfaces.
    ///
    /// | Interface | Ingress                | Egress                 |
    /// | :-------: | ---------------------- | ---------------------- |
    /// |  **Fwd**  | `Array<HOption<P>, N>` | `HOption<Array<P, N>>` |
    /// |  **Bwd**  | `Array<R, N>`          | `Array<R, N>`          |
    fn join(self) -> I<ValidH<Array<P, N>, Array<R, N>>, D> {
        unsafe {
            self.fsm((), |ip, er, ()| {
                let ep = if ip.all(|p| p.is_some()) { Some(ip.map(|p| p.unwrap())) } else { None };
                (ep, er, ())
            })
        }
    }
}

/// Extension trait for `join_valid`
pub trait JoinValidExt: Interface {
    /// Egress interface.
    type E: Interface;

    /// Join valid.
    fn join_valid(self) -> Self::E;
}

impl<P1: Copy, P2: Copy> JoinValidExt for (Valid<P1>, Valid<P2>) {
    type E = Valid<(P1, P2)>;

    /// A variation of [`join`] for valid interfaces, that has the correct resolver type.
    ///
    /// - Payloads: Zipped to one interface. Note that a payload may get dropped if the other interface's payload is
    ///     `None`.
    /// - Resolver: The resolver carries no information.
    ///
    /// | Interface | Ingress                      | Egress              |
    /// | :-------: | ---------------------------- | ------------------- |
    /// |  **Fwd**  | `(HOption<P1>, HOption<P2>)` | `HOption<(P1, P2)>` |
    /// |  **Bwd**  | `((), ())`                   | `()`                |
    fn join_valid(self) -> Valid<(P1, P2)> {
        self.join().map_resolver::<()>(|_| ((), ()))
    }
}

macro_rules! impl_valid_join_valid {
    ($($P:ident),+) => {
        impl<$($P: Copy,)+> JoinValidExt for ($(Valid<$P>,)+) {
            type E = Valid<($($P,)+)>;

            /// A variation of [`join_valid`](join) for 3-12 valid interfaces. See the 2-tuple version for more
            /// information.
            fn join_valid(self) -> Valid<($($P,)+)> {
                self.join().map_resolver::<()>(|_| ($(replace!($P, ()),)+))
            }
        }
    };
}

impl_valid_join_valid! { P1, P2, P3 }
impl_valid_join_valid! { P1, P2, P3, P4 }
impl_valid_join_valid! { P1, P2, P3, P4, P5 }
impl_valid_join_valid! { P1, P2, P3, P4, P5, P6 }
impl_valid_join_valid! { P1, P2, P3, P4, P5, P6, P7 }
impl_valid_join_valid! { P1, P2, P3, P4, P5, P6, P7, P8 }
impl_valid_join_valid! { P1, P2, P3, P4, P5, P6, P7, P8, P9 }
impl_valid_join_valid! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10 }
impl_valid_join_valid! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11 }
impl_valid_join_valid! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12 }

// Joins N valid interfaces.
impl<P: Copy, const N: usize> JoinValidExt for [Valid<P>; N] {
    type E = Valid<Array<P, N>>;

    /// A variation of [`join`] for valid interfaces, that has the correct resolver type.
    ///
    /// - Payloads: Zipped to one interface. Note that a payload may get dropped if the other interface's payload is
    ///     `None`.
    /// - Resolver: The resolver carries no information.
    ///
    /// | Interface | Ingress                | Egress                 |
    /// | :-------: | ---------------------- | ---------------------- |
    /// |  **Fwd**  | `Array<HOption<P>, N>` | `HOption<Array<P, N>>` |
    /// |  **Bwd**  | `Array<(), N>`         | `()`                   |
    fn join_valid(self) -> Valid<Array<P, N>> {
        self.join().map_resolver::<()>(|_| ().repeat())
    }
}

impl<P1: Copy, P2: Copy, R1: Copy, R2: Copy> JoinExt
    for (I<VrH<P1, R1>, { Dep::Helpful }>, I<VrH<P2, R2>, { Dep::Helpful }>)
{
    type E = I<VrH<(P1, P2), (R1, R2)>, { Dep::Helpful }>;

    /// Joins two `VrH` hazard interfaces.
    ///
    /// All ingress transfers and an egress transfer happen at once when all the ingress payloads are valid and the
    /// egress ready signal is true.
    ///
    /// - Payloads: Zipped to one interface.
    /// - Resolver: If all the ingress payloads are valid and the egress ready signal is true, then all the ingress
    ///     ready signals are true and the inner value `(R1, R2)` of the resolver is preserved and split. Otherwise, all
    ///     the ingress ready signals are false and the resolver value is dropped.
    ///
    /// | Interface | Ingress                      | Egress              |
    /// | :-------: | ---------------------------- | ------------------- |
    /// |  **Fwd**  | `(HOption<P1>, HOption<P2>)` | `HOption<(P1, P2)>` |
    /// |  **Bwd**  | `(Ready<R1>, Ready<R2>)`     | `Ready<(R1, R2)>`   |
    fn join(self) -> I<VrH<(P1, P2), (R1, R2)>, { Dep::Helpful }> {
        unsafe {
            self.fsm::<I<VrH<(P1, P2), (R1, R2)>, { Dep::Helpful }>, ()>((), |ip, er, s| {
                let ep = match ip {
                    (Some(l), Some(r)) => Some((l, r)),
                    _ => None,
                };
                let ir = if ep.is_some() && er.ready {
                    (Ready::valid(er.inner.0), Ready::valid(er.inner.1))
                } else {
                    (Ready::invalid(), Ready::invalid())
                };

                (ep, ir, s)
            })
        }
    }
}

macro_rules! impl_i_vr_h_join {
    ($($P:ident),+; $($R:ident),+; $($index:tt),+) => {
        impl<$($P: Copy,)+ $($R: Copy,)+> JoinExt for ($(I<VrH<$P, $R>, { Dep::Helpful }>,)+) {
            type E = I<VrH<($($P,)+), ($($R,)+)>, { Dep::Helpful }>;

            /// A variation of [`join`] for 3-12 `VrH` hazard interfaces. See the 2-tuple version for more information.
            fn join(self) -> I<VrH<($($P,)+), ($($R,)+)>, { Dep::Helpful }> {
                unsafe {
                    self.fsm::<I<VrH<($($P,)+), ($($R,)+)>, { Dep::Helpful }>, ()>((), |ip, er, s| {
                        let ep = match ip {
                            // This is a hack that uses `P1`, `P2`, ... as variable names.
                            #[allow(non_snake_case)]
                            ($(Some($P),)+) => Some(($($P,)+)),
                            _ => None,
                        };
                        let ir = if ep.is_some() && er.ready {
                            ($(Ready::valid(er.inner.$index),)+)
                        } else {
                            ($(replace!($index, Ready::invalid()),)+)
                        };

                        (ep, ir, s)
                    })
                }
            }
        }
    };
}

impl_i_vr_h_join! { P1, P2, P3; R1, R2, R3; 0, 1, 2 }
impl_i_vr_h_join! { P1, P2, P3, P4; R1, R2, R3, R4; 0, 1, 2, 3 }
impl_i_vr_h_join! { P1, P2, P3, P4, P5; R1, R2, R3, R4, R5; 0, 1, 2, 3, 4 }
impl_i_vr_h_join! { P1, P2, P3, P4, P5, P6; R1, R2, R3, R4, R5, R6; 0, 1, 2, 3, 4, 5 }
impl_i_vr_h_join! { P1, P2, P3, P4, P5, P6, P7; R1, R2, R3, R4, R5, R6, R7; 0, 1, 2, 3, 4, 5, 6 }
impl_i_vr_h_join! { P1, P2, P3, P4, P5, P6, P7, P8; R1, R2, R3, R4, R5, R6, R7, R8; 0, 1, 2, 3, 4, 5, 6, 7 }
impl_i_vr_h_join! { P1, P2, P3, P4, P5, P6, P7, P8, P9; R1, R2, R3, R4, R5, R6, R7, R8, R9; 0, 1, 2, 3, 4, 5, 6, 7, 8 }
impl_i_vr_h_join! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 }
impl_i_vr_h_join! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 }
impl_i_vr_h_join! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 }

impl<P1: Copy, P2: Copy, R: Copy, const D: Dep> JoinExt for (I<VrH<P1, R>, D>, Valid<P2>) {
    type E = I<VrH<(P1, P2), R>, D>;

    /// Joins a `VrH` hazard interface and a valid interface.
    ///
    /// - Payloads: Zipped to one interface. Note that the valid interface's payload `P2` may get discarded if a
    ///     transfer does not happen for the `VrH` hazard interface.
    /// - Resolver: The resolver is preserved through the `VrH` hazard interface.
    ///
    /// | Interface | Ingress                      | Egress              |
    /// | :-------: | ---------------------------- | ------------------- |
    /// |  **Fwd**  | `(HOption<P1>, HOption<P2>)` | `HOption<(P1, P2)>` |
    /// |  **Bwd**  | `(Ready<R>, ())`             | `Ready<R>`          |
    fn join(self) -> I<VrH<(P1, P2), R>, D> {
        unsafe {
            self.fsm::<I<VrH<(P1, P2), R>, D>, ()>((), |(ip1, ip2), er, s| {
                let ep = ip1.zip(ip2);
                let ir = (er, ());
                (ep, ir, s)
            })
        }
    }
}

// TODO: Add 4 to 12-tuple variants.
impl<P1: Copy, P2: Copy, P3: Copy, R: Copy, const D: Dep> JoinExt for (I<VrH<P1, R>, D>, Valid<P2>, Valid<P3>) {
    type E = I<VrH<(P1, P2, P3), R>, D>;

    /// A variation of [`join`] for a `VrH` hazard interface and 2 valid interfaces. See the 2-tuple version for more
    /// information.
    fn join(self) -> I<VrH<(P1, P2, P3), R>, D> {
        ((self.0, self.1).join(), self.2).join().map(|((p1, p2), p3)| (p1, p2, p3))
    }
}

impl<P: Copy, R: Copy, const N: usize> JoinExt for [I<VrH<P, R>, { Dep::Helpful }>; N] {
    type E = I<VrH<Array<P, N>, Array<R, N>>, { Dep::Helpful }>;

    /// Joins `N` `VrH` hazard interfaces.
    ///
    /// All ingress transfers and an egress transfer happen at once when all the ingress payloads are valid and the
    /// egress ready signal is true.
    ///
    /// - Payloads: Zipped to one interface.
    /// - Resolver: If all the ingress payloads are valid and the egress ready signal is true, then all the ingress
    ///     ready signals are true and the inner value `Array<R, N>` of the resolver is preserved and split. Otherwise,
    ///     all the ingress ready signals are false and the resolver value is dropped.
    ///
    /// | Interface | Ingress                | Egress                 |
    /// | :-------: | ---------------------- | ---------------------- |
    /// |  **Fwd**  | `Array<HOption<P>, N>` | `HOption<Array<P, N>>` |
    /// |  **Bwd**  | `Array<Ready<R>, N>`   | `Ready<Array<R, N>>`   |
    fn join(self) -> I<VrH<Array<P, N>, Array<R, N>>, { Dep::Helpful }> {
        unsafe {
            self.fsm::<I<VrH<Array<P, N>, Array<R, N>>, { Dep::Helpful }>, ()>((), |ip, er, s| {
                let ep = if ip.all(|p| p.is_some()) { Some(ip.map(|p| p.unwrap_or(x()))) } else { None };
                let ir = if ep.is_some() && er.ready {
                    // TODO: Use the following expression instead of the below: `er.inner.map(Ready::valid)`
                    Ready::valid(()).repeat::<N>().zip(er.inner).map(|(_, r)| Ready::valid(r))
                } else {
                    Ready::invalid().repeat::<N>()
                };

                (ep, ir, s)
            })
        }
    }
}

/// Extension trait for `join_vr`.
pub trait JoinVrExt: Interface {
    /// Egress interface.
    type E: Interface;

    /// Join valid-ready.
    fn join_vr(self) -> Self::E;
}

impl<P1: Copy, P2: Copy> JoinVrExt for (Vr<P1>, Vr<P2>) {
    type E = Vr<(P1, P2)>;

    /// A variation of [`join`] for valid-ready interfaces, that has the correct resolver type.
    ///
    /// - Payloads: Zipped to one interface.
    /// - Resolver: If all the ingress payloads are valid and the egress ready signal is true, then all the ingress
    ///     ready signals are true. Otherwise, all the ingress ready signals are false.
    ///
    /// | Interface | Ingress                      | Egress              |
    /// | :-------: | ---------------------------- | ------------------- |
    /// |  **Fwd**  | `(HOption<P1>, HOption<P2>)` | `HOption<(P1, P2)>` |
    /// |  **Bwd**  | `(Ready<()>, Ready<()>)`     | `Ready<()>`         |
    fn join_vr(self) -> Vr<(P1, P2)> {
        self.join().map_resolver::<()>(|_| ((), ()))
    }
}

macro_rules! impl_vr_join_vr {
    ($($P:ident),+) => {
        impl<$($P: Copy,)+> JoinVrExt for ($(Vr<$P>,)+) {
            type E = Vr<($($P,)+)>;

            /// A variation of [`join_vr`](join) for 3-12 valid-ready interfaces. See the 2-tuple version for more
            /// information.
            fn join_vr(self) -> Vr<($($P,)+)> {
                self.join().map_resolver::<()>(|_| ($(replace!($P, ()),)+))
            }
        }
    };
}

impl_vr_join_vr! { P1, P2, P3 }
impl_vr_join_vr! { P1, P2, P3, P4 }
impl_vr_join_vr! { P1, P2, P3, P4, P5 }
impl_vr_join_vr! { P1, P2, P3, P4, P5, P6 }
impl_vr_join_vr! { P1, P2, P3, P4, P5, P6, P7 }
impl_vr_join_vr! { P1, P2, P3, P4, P5, P6, P7, P8 }
impl_vr_join_vr! { P1, P2, P3, P4, P5, P6, P7, P8, P9 }
impl_vr_join_vr! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10 }
impl_vr_join_vr! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11 }
impl_vr_join_vr! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12 }

impl<P: Copy, const N: usize> JoinVrExt for [Vr<P>; N] {
    type E = Vr<Array<P, N>>;

    /// A variation of [`join`] for valid-ready interfaces, that has the correct resolver type.
    ///
    /// - Payloads: Zipped to one interface.
    /// - Resolver: If all the ingress payloads are valid and the egress ready signal is true, then all the ingress
    ///     ready signals are true. Otherwise, all the ingress ready signals are false.
    ///
    /// | Interface | Ingress                | Egress                 |
    /// | :-------: | ---------------------- | ---------------------- |
    /// |  **Fwd**  | `Array<HOption<P>, N>` | `HOption<Array<P, N>>` |
    /// |  **Bwd**  | `Array<Ready<()>, N>`  | `Ready<()>`            |
    fn join_vr(self) -> Vr<Array<P, N>> {
        self.join().map_resolver::<()>(|_| ().repeat())
    }
}
