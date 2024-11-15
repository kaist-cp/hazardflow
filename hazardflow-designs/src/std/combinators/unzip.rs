//! Unzip.

use super::*;

impl<P1: Copy, P2: Copy, R1: Copy, R2: Copy, const D: Dep> I<ValidH<(P1, P2), (R1, R2)>, D> {
    /// Unzips a `ValidH` hazard interface into two `ValidH` hazard interfaces.
    ///
    /// - Payload: Unzipped to multiple interfaces.
    /// - Resolvers: Preserved, and combined into one interface.
    ///
    /// | Interface | Ingress             | Egress                       |
    /// | :-------: | ------------------- | ---------------------------- |
    /// |  **Fwd**  | `HOption<(P1, P2)>` | `(HOption<P1>, HOption<P2>)` |
    /// |  **Bwd**  | `(R1, R2)`          | `(R1, R2)`                   |
    #[allow(clippy::type_complexity)]
    pub fn unzip(self) -> (I<ValidH<P1, R1>, D>, I<ValidH<P2, R2>, D>) {
        unsafe {
            Interface::fsm(self, (), |ip, er, ()| {
                let ep = ip.unzip();
                (ep, er, ())
            })
        }
    }
}

macro_rules! impl_i_valid_h_unzip {
    ($($P:ident),+; $($R:ident),+) => {
        impl<$($P: Copy,)+ $($R: Copy,)+ const D: Dep> I<ValidH<($($P,)+), ($($R,)+)>, D> {
            /// A variation of [`unzip`] to 3-12 `ValidH` hazard interfaces. See the 2-tuple version for more
            /// information.
            #[allow(clippy::type_complexity)]
            pub fn unzip(self) -> ($(I<ValidH<$P, $R>, D>,)+) {
                unsafe {
                    Interface::fsm(self, (), |ip, er, ()| {
                        // Equivalent to `unzip` for `HOption<(P1, P2, ...)>`.
                        let ep = match ip {
                            // This is a hack that uses `P1`, `P2`, ... as variable names.
                            #[allow(non_snake_case)]
                            Some(($($P,)+)) => ($(Some($P),)+),
                            None => ($(replace!($P, None),)+),
                        };
                        (ep, er, ())
                    })
                }
            }
        }
    };
}

impl_i_valid_h_unzip! { P1, P2, P3; R1, R2, R3 }
impl_i_valid_h_unzip! { P1, P2, P3, P4; R1, R2, R3, R4 }
impl_i_valid_h_unzip! { P1, P2, P3, P4, P5; R1, R2, R3, R4, R5 }
impl_i_valid_h_unzip! { P1, P2, P3, P4, P5, P6; R1, R2, R3, R4, R5, R6 }
impl_i_valid_h_unzip! { P1, P2, P3, P4, P5, P6, P7; R1, R2, R3, R4, R5, R6, R7 }
impl_i_valid_h_unzip! { P1, P2, P3, P4, P5, P6, P7, P8; R1, R2, R3, R4, R5, R6, R7, R8 }
impl_i_valid_h_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9; R1, R2, R3, R4, R5, R6, R7, R8, R9 }
impl_i_valid_h_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10 }
impl_i_valid_h_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11 }
impl_i_valid_h_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12 }

impl<P: Copy, R: Copy, const N: usize, const D: Dep> I<ValidH<Array<P, N>, Array<R, N>>, D> {
    /// Unzips a `ValidH` hazard interface into `N` `ValidH` hazard interfaces.
    ///
    /// - Payload: Unzipped to multiple interfaces.
    /// - Resolvers: Preserved, and combined into one interface.
    ///
    /// | Interface | Ingress                | Egress                 |
    /// | :-------: | ---------------------- | ---------------------- |
    /// |  **Fwd**  | `HOption<Array<P, N>>` | `Array<HOption<P>, N>` |
    /// |  **Bwd**  | `Array<R, N>`          | `Array<R, N>`          |
    pub fn unzip(self) -> [I<ValidH<P, R>, D>; N] {
        unsafe {
            Interface::fsm(self, (), |ip, er, ()| {
                let ep = ip.map_or(None.repeat(), |ip| ip.map(|x| Some(x)));
                (ep, er, ())
            })
        }
    }
}

impl<P1: Copy, P2: Copy> Valid<(P1, P2)> {
    /// A variation of [`unzip`] for a valid interface, that has the correct resolver type.
    ///
    /// - Payload: Unzipped to multiple interfaces.
    /// - Resolvers: The resolvers carry no information.
    ///
    /// | Interface | Ingress             | Egress                       |
    /// | :-------: | ------------------- | ---------------------------- |
    /// |  **Fwd**  | `HOption<(P1, P2)>` | `(HOption<P1>, HOption<P2>)` |
    /// |  **Bwd**  | `()`                | `((), ())         `          |
    pub fn unzip(self) -> (Valid<P1>, Valid<P2>) {
        self.map_resolver::<((), ())>(|_| ()).unzip()
    }
}

macro_rules! impl_valid_unzip {
    ($($P:ident),+) => {
        impl<$($P: Copy,)+> Valid<($($P,)+)> {
            /// A variation of [`unzip`] to 3-12 valid interfaces. See the 2-tuple version for more information.
            pub fn unzip(self) -> ($(Valid<$P>,)+) {
                self.map_resolver::<($(replace!($P, ()),)+)>(|_| ()).unzip()
            }
        }
    };
}

impl_valid_unzip! { P1, P2, P3 }
impl_valid_unzip! { P1, P2, P3, P4 }
impl_valid_unzip! { P1, P2, P3, P4, P5 }
impl_valid_unzip! { P1, P2, P3, P4, P5, P6 }
impl_valid_unzip! { P1, P2, P3, P4, P5, P6, P7 }
impl_valid_unzip! { P1, P2, P3, P4, P5, P6, P7, P8 }
impl_valid_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9 }
impl_valid_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10 }
impl_valid_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11 }
impl_valid_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12 }

impl<P: Copy, const N: usize> Valid<Array<P, N>> {
    /// A variation of [`unzip`] for a valid interface, that has the correct resolver type.
    ///
    /// - Payload: Unzipped to multiple interfaces.
    /// - Resolvers: The resolvers carry no information.
    ///
    /// | Interface | Ingress                | Egress                 |
    /// | :-------: | ---------------------- | ---------------------- |
    /// |  **Fwd**  | `HOption<Array<P, N>>` | `Array<HOption<P>, N>` |
    /// |  **Bwd**  | `()`                   | `Array<(), N>`         |
    pub fn unzip(self) -> [Valid<P>; N] {
        self.map_resolver::<Array<(), N>>(|_| ()).unzip()
    }
}

impl<P1: Copy, P2: Copy, R1: Copy, R2: Copy, const D: Dep> I<VrH<(P1, P2), (R1, R2)>, D> {
    /// Unzips a `VrH` hazard interface into two `VrH` hazard interfaces.
    ///
    /// An ingress transfer and all egress transfers happen at once when the ingress payload is valid and all the egress
    /// ready signals are true.
    ///
    /// - Payload: Unzipped to multiple interfaces, and all the egress payloads become available at once when all the
    ///     egress ready signals are true.
    /// - Resolvers: The ingress ready signal is true if all the egress ready signals are true. The inner values `R1`,
    ///     `R2` of the resolvers are preserved, and combined into one interface.
    ///
    /// | Interface | Ingress             | Egress                       |
    /// | :-------: | ------------------- | ---------------------------- |
    /// |  **Fwd**  | `HOption<(P1, P2)>` | `(HOption<P1>, HOption<P2>)` |
    /// |  **Bwd**  | `Ready<(R1, R2)>`   | `(Ready<R1>, Ready<R2>)`     |
    #[allow(clippy::type_complexity)]
    pub fn unzip(self) -> (I<VrH<P1, R1>, D>, I<VrH<P2, R2>, D>) {
        unsafe {
            Interface::fsm::<(I<VrH<P1, R1>, D>, I<VrH<P2, R2>, D>), ()>(self, (), |ip, (er1, er2), ()| {
                let ep1 = if er2.ready { ip.map(|(p, _)| p) } else { None };
                let ep2 = if er1.ready { ip.map(|(_, p)| p) } else { None };
                let ir = Ready::new(er1.ready && er2.ready, (er1.inner, er2.inner));

                ((ep1, ep2), ir, ())
            })
        }
    }
}

macro_rules! impl_i_vr_h_unzip {
    ($($P:ident),+; $($R:ident),+; $($index:tt),+) => {
        impl<$($P: Copy,)+ $($R: Copy,)+ const D: Dep> I<VrH<($($P,)+), ($($R,)+)>, D> {
            /// A variation of [`unzip`] to 3-12 `VrH` hazard interfaces. See the 2-tuple version for more information.
            #[allow(clippy::type_complexity)]
            pub fn unzip(self) -> ($(I<VrH<$P, $R>, { Dep::Demanding }>,)+) {
                unsafe {
                    Interface::fsm(self, (), |ip, er: ($(Ready<$R>,)+), ()| {
                        let ready = $(er.$index.ready)&&+;
                        let ep = if ready && ip.is_some() {
                            // This is a hack that uses `P1`, `P2`, ... as variable names.
                            #[allow(non_snake_case)]
                            let ($($P,)+) = ip.unwrap();
                            ($(Some($P),)+)
                        } else {
                            ($(replace!($P, None),)+)
                        };
                        let ir = Ready::new(ready, ($(er.$index.inner,)+));
                        (ep, ir, ())
                    })
                }
            }
        }
    };
}

impl_i_vr_h_unzip! { P1, P2, P3; R1, R2, R3; 0, 1, 2 }
impl_i_vr_h_unzip! { P1, P2, P3, P4; R1, R2, R3, R4; 0, 1, 2, 3 }
impl_i_vr_h_unzip! { P1, P2, P3, P4, P5; R1, R2, R3, R4, R5; 0, 1, 2, 3, 4 }
impl_i_vr_h_unzip! { P1, P2, P3, P4, P5, P6; R1, R2, R3, R4, R5, R6; 0, 1, 2, 3, 4, 5 }
impl_i_vr_h_unzip! { P1, P2, P3, P4, P5, P6, P7; R1, R2, R3, R4, R5, R6, R7; 0, 1, 2, 3, 4, 5, 6 }
impl_i_vr_h_unzip! { P1, P2, P3, P4, P5, P6, P7, P8; R1, R2, R3, R4, R5, R6, R7, R8; 0, 1, 2, 3, 4, 5, 6, 7 }
impl_i_vr_h_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9; R1, R2, R3, R4, R5, R6, R7, R8, R9; 0, 1, 2, 3, 4, 5, 6, 7, 8 }
impl_i_vr_h_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 }
impl_i_vr_h_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 }
impl_i_vr_h_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 }

impl<P: Copy, R: Copy, const N: usize, const D: Dep> I<VrH<Array<P, N>, Array<R, N>>, D> {
    /// Unzips a `VrH` hazard interface into `N` `VrH` hazard interfaces.
    ///
    /// An ingress transfer and all egress transfers happen at once when the ingress payload is valid and all the egress
    /// ready signals are true.
    ///
    /// - Payload: Unzipped to multiple interfaces, and all the egress payloads become available at once when all the
    ///     egress ready signals are true.
    /// - Resolvers: The ingress ready signal is true if all the egress ready signals are true. The inner values `R` of
    ///     the resolvers are preserved, and combined into one interface.
    ///
    /// | Interface | Ingress                | Egress                 |
    /// | :-------: | ---------------------- | ---------------------- |
    /// |  **Fwd**  | `HOption<Array<P, N>>` | `Array<HOption<P>, N>` |
    /// |  **Bwd**  | `Ready<Array<R, N>>`   | `Array<Ready<R>, N>`   |
    pub fn unzip(self) -> [I<VrH<P, R>, { Dep::Demanding }>; N] {
        unsafe {
            Interface::fsm(self, (), |ip, er: Array<Ready<R>, N>, ()| {
                let ready = er.all(|r| r.ready);
                let ep = if ready && ip.is_some() { ip.unwrap().map(Some) } else { None.repeat() };
                let ir = Ready::new(ready, er.map(|r| r.inner));
                (ep, ir, ())
            })
        }
    }
}

impl<P1: Copy, P2: Copy, const D: Dep> Vr<(P1, P2), D> {
    /// A variation of [`unzip`] for a valid-ready interface, that has the correct resolver type.
    ///
    /// - Payload: Unzipped to multiple interfaces, and all the egress payloads become available at once when all the
    ///     egress ready signals are true.
    /// - Resolvers: The ingress ready signal is true if all the egress ready signals are true.
    ///
    /// | Interface | Ingress             | Egress                       |
    /// | :-------: | ------------------- | ---------------------------- |
    /// |  **Fwd**  | `HOption<(P1, P2)>` | `(HOption<P1>, HOption<P2>)` |
    /// |  **Bwd**  | `Ready<()>`         | `(Ready<()>, Ready<()>)`     |
    pub fn unzip(self) -> (Vr<P1, D>, Vr<P2, D>) {
        self.map_resolver::<((), ())>(|_| ()).unzip()
    }
}

macro_rules! impl_vr_unzip {
    ($($P:ident),+) => {
        impl<$($P: Copy,)+ const D: Dep> Vr<($($P,)+), D> {
            /// A variation of [`unzip`] to 3-12 valid-ready interfaces. See the 2-tuple version for more information.
            pub fn unzip(self) -> ($(Vr<$P, { Dep::Demanding }>,)+) {
                self.map_resolver_inner::<($(replace!($P, ()),)+)>(|_| ()).unzip()
            }
        }
    };
}

impl_vr_unzip! { P1, P2, P3 }
impl_vr_unzip! { P1, P2, P3, P4 }
impl_vr_unzip! { P1, P2, P3, P4, P5 }
impl_vr_unzip! { P1, P2, P3, P4, P5, P6 }
impl_vr_unzip! { P1, P2, P3, P4, P5, P6, P7 }
impl_vr_unzip! { P1, P2, P3, P4, P5, P6, P7, P8 }
impl_vr_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9 }
impl_vr_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10 }
impl_vr_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11 }
impl_vr_unzip! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12 }

impl<P: Copy, const N: usize, const D: Dep> Vr<Array<P, N>, D> {
    /// A variation of [`unzip`] for a valid-ready interface, that has the correct resolver type.
    ///
    /// - Payload: Unzipped to multiple interfaces, and all the egress payloads become available at once when all the
    ///     egress ready signals are true.
    /// - Resolvers: The ingress ready signal is true if all the egress ready signals are true.
    ///
    /// | Interface | Ingress                | Egress                 |
    /// | :-------: | ---------------------- | ---------------------- |
    /// |  **Fwd**  | `HOption<Array<P, N>>` | `Array<HOption<P>, N>` |
    /// |  **Bwd**  | `Ready<()>`            | `Array<Ready<()>, N>`  |
    pub fn unzip(self) -> [Vr<P, { Dep::Demanding }>; N] {
        self.map_resolver::<Array<(), N>>(|_| ()).unzip()
    }
}
