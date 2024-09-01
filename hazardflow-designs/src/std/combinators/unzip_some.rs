//! Unzip some.

use super::*;

impl<P1: Copy, P2: Copy, R1: Copy, R2: Copy, const D: Dep> I<VrH<(P1, P2), (R1, R2)>, D> {
    /// Unzips the `VrH` hazard interface into some of the two `VrH` hazard interfaces.
    ///
    /// An ingress transfer and egress transfers happen as soon as when the ingress payload is valid and at least one of
    /// the egress ready signals is true. Note that the egress transfers happen only for the ready egress interfaces.
    ///
    /// - Payload: Unzipped to multiple interfaces, and each egress payload becomes available when its own egress ready
    ///     signal is true.
    /// - Resolvers: The ingress ready signal is true if any of the egress ready signals are true. The inner values
    ///     `R1`, `R2` of the resolvers are preserved, and combined into one interface.
    ///
    /// | Interface | Ingress             | Egress                       |
    /// | :-------: | ------------------- | ---------------------------- |
    /// |  **Fwd**  | `HOption<(P1, P2)>` | `(HOption<P1>, HOption<P2>)` |
    /// |  **Bwd**  | `Ready<(R1, R2)>`   | `(Ready<R1>, Ready<R2>)`     |
    #[allow(clippy::type_complexity)]
    pub fn unzip_some(self) -> (I<VrH<P1, R1>, { Dep::Demanding }>, I<VrH<P2, R2>, { Dep::Demanding }>) {
        unsafe {
            Interface::fsm(self, (), |ip, er: (Ready<R1>, Ready<R2>), ()| {
                let ready = er.0.ready || er.1.ready;
                let ep = if ip.is_some() {
                    let (p1, p2) = ip.unwrap();
                    let ep0 = if er.0.ready { Some(p1) } else { None };
                    let ep1 = if er.1.ready { Some(p2) } else { None };
                    (ep0, ep1)
                } else {
                    (None, None)
                };
                let ir = Ready::new(ready, (er.0.inner, er.1.inner));
                (ep, ir, ())
            })
        }
    }
}

macro_rules! impl_i_vr_h_unzip_some {
    ($($P:ident),+; $($R:ident),+; $($index:tt),+) => {
        impl<$($P: Copy,)+ $($R: Copy,)+ const D: Dep> I<VrH<($($P,)+), ($($R,)+)>, D> {
            /// A variation of [`unzip_some`] to 3-12 `VrH` hazard interfaces. See the 2-tuple version for more
            /// information.
            #[allow(clippy::type_complexity)]
            pub fn unzip_some(self) -> ($(I<VrH<$P, $R>, { Dep::Demanding }>,)+) {
                unsafe {
                    Interface::fsm(self, (), |ip, er: ($(Ready<$R>,)+), ()| {
                        let ready = $(er.$index.ready)||+;
                        #[allow(non_snake_case)]
                        let ep = if ip.is_some() {
                            // This is a hack that uses `P1`, `P2`, ... as variable names.
                            let ($($P,)+) = ip.unwrap();
                            $(let $P = if er.$index.ready { Some($P) } else { None };)+
                            ($($P,)+)
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

impl_i_vr_h_unzip_some! { P1, P2, P3; R1, R2, R3; 0, 1, 2 }
impl_i_vr_h_unzip_some! { P1, P2, P3, P4; R1, R2, R3, R4; 0, 1, 2, 3 }
impl_i_vr_h_unzip_some! { P1, P2, P3, P4, P5; R1, R2, R3, R4, R5; 0, 1, 2, 3, 4 }
impl_i_vr_h_unzip_some! { P1, P2, P3, P4, P5, P6; R1, R2, R3, R4, R5, R6; 0, 1, 2, 3, 4, 5 }
impl_i_vr_h_unzip_some! { P1, P2, P3, P4, P5, P6, P7; R1, R2, R3, R4, R5, R6, R7; 0, 1, 2, 3, 4, 5, 6 }
impl_i_vr_h_unzip_some! { P1, P2, P3, P4, P5, P6, P7, P8; R1, R2, R3, R4, R5, R6, R7, R8; 0, 1, 2, 3, 4, 5, 6, 7 }
impl_i_vr_h_unzip_some! { P1, P2, P3, P4, P5, P6, P7, P8, P9; R1, R2, R3, R4, R5, R6, R7, R8, R9; 0, 1, 2, 3, 4, 5, 6, 7, 8 }
impl_i_vr_h_unzip_some! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 }
impl_i_vr_h_unzip_some! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 }
impl_i_vr_h_unzip_some! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12; R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 }

impl<P: Copy, R: Copy, const N: usize, const D: Dep> I<VrH<Array<P, N>, Array<R, N>>, D> {
    /// Unzips the `VrH` hazard interface into some of the `N` `VrH` hazard interfaces.
    ///
    /// An ingress transfer and egress transfers happen as soon as when the ingress payload is valid and at least one of
    /// the egress ready signals is true. Note that the egress transfers happen only for the ready egress interfaces.
    ///
    /// - Payload: Unzipped to multiple interfaces, and each egress payload becomes available when its own egress ready
    ///     signal is true.
    /// - Resolvers: The ingress ready signal is true if any of the egress ready signals are true. The inner values `R`
    ///     of the resolvers are preserved, and combined into one interface.
    ///
    /// | Interface | Ingress              | Egress                 |
    /// | :-------: | -------------------- | ---------------------- |
    /// |  **Fwd**  | `HOption<Array<P>>`  | `Array<HOption<P>, N>` |
    /// |  **Bwd**  | `Ready<Array<R, N>>` | `Array<Ready<R>, N>`   |
    pub fn unzip_some(self) -> [I<VrH<P, R>, { Dep::Demanding }>; N] {
        unsafe {
            Interface::fsm(self, (), |ip, er: Array<Ready<R>, N>, ()| {
                let ready = er.any(|r| r.ready);
                let ep = if ip.is_some() {
                    ip.unwrap().zip(er).map(|(p, r)| if r.ready { Some(p) } else { None })
                } else {
                    None.repeat()
                };
                let ir = Ready::new(ready, er.map(|r| r.inner));
                (ep, ir, ())
            })
        }
    }
}

impl<P1: Copy, P2: Copy, const D: Dep> Vr<(P1, P2), D> {
    /// A variation of [`unzip_some`] for a valid-ready interface, that has the correct resolver type.
    ///
    /// - Payload: Unzipped to multiple interfaces, and each egress payload becomes available when its own egress ready
    ///     signal is true.
    /// - Resolvers: The ingress ready signal is true if any of the egress ready signals are true.
    ///
    /// | Interface | Ingress             | Egress                       |
    /// | :-------: | ------------------- | ---------------------------- |
    /// |  **Fwd**  | `HOption<(P1, P2)>` | `(HOption<P1>, HOption<P2>)` |
    /// |  **Bwd**  | `Ready<()>`         | `(Ready<()>, Ready<()>)`     |
    pub fn unzip_some(self) -> (Vr<P1, { Dep::Demanding }>, Vr<P2, { Dep::Demanding }>) {
        self.map_resolver::<((), ())>(|_| ()).unzip_some()
    }
}

macro_rules! impl_vr_unzip_some {
    ($($P:ident),+) => {
        impl<$($P: Copy,)+ const D: Dep> Vr<($($P,)+), D> {
            /// A variation of [`unzip_some`] to 3-12 valid-ready interfaces. See the 2-tuple version for more
            /// information.
            pub fn unzip_some(self) -> ($(Vr<$P, { Dep::Demanding }>,)+) {
                self.map_resolver::<($(replace!($P, ()),)+)>(|_| ()).unzip_some()
            }
        }
    };
}

impl_vr_unzip_some! { P1, P2, P3 }
impl_vr_unzip_some! { P1, P2, P3, P4 }
impl_vr_unzip_some! { P1, P2, P3, P4, P5 }
impl_vr_unzip_some! { P1, P2, P3, P4, P5, P6 }
impl_vr_unzip_some! { P1, P2, P3, P4, P5, P6, P7 }
impl_vr_unzip_some! { P1, P2, P3, P4, P5, P6, P7, P8 }
impl_vr_unzip_some! { P1, P2, P3, P4, P5, P6, P7, P8, P9 }
impl_vr_unzip_some! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10 }
impl_vr_unzip_some! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11 }
impl_vr_unzip_some! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12 }

impl<P: Copy, const N: usize, const D: Dep> Vr<Array<P, N>, D> {
    /// A variation of [`unzip_some`] for a valid-ready interface, that has the correct resolver type.
    ///
    /// - Payload: Unzipped to multiple interfaces, and each egress payload becomes available when its own egress ready
    ///     signal is true.
    /// - Resolvers: The ingress ready signal is true if any of the egress ready signals are true.
    ///
    /// | Interface | Ingress                | Egress                 |
    /// | :-------: | ---------------------- | ---------------------- |
    /// |  **Fwd**  | `HOption<Array<P, N>>` | `Array<HOption<P>, N>` |
    /// |  **Bwd**  | `Ready<()>`            | `Array<Ready<()>, N>`  |
    pub fn unzip_some(self) -> [Vr<P, { Dep::Demanding }>; N] {
        self.map_resolver::<Array<(), N>>(|_| ()).unzip_some()
    }
}
