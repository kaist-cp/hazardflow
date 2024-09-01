//! Zip any.

use std::marker::PhantomData;

use super::*;

/// Extension trait for `zip_any`.
pub trait ZipAnyExt: Interface {
    /// Egress interface.
    type E: Interface;

    /// Zip-any.
    fn zip_any(self) -> Self::E;
}

/// Hazard specification for zip-any with two interfaces.
#[derive(Debug, Clone, Copy)]
pub struct ZipAnyH<H1: Hazard, H2: Hazard> {
    _marker: PhantomData<(H1, H2)>,
}

impl<H1: Hazard, H2: Hazard> Hazard for ZipAnyH<H1, H2> {
    type P = (HOption<H1::P>, HOption<H2::P>);
    type R = (H1::R, H2::R);

    fn ready((p1, p2): Self::P, (r1, r2): Self::R) -> bool {
        p1.is_some_and(|p| H1::ready(p, r1)) || p2.is_some_and(|p| H2::ready(p, r2))
    }
}

impl<H1: Hazard, H2: Hazard, const D: Dep> ZipAnyExt for (I<H1, D>, I<H2, D>) {
    type E = I<ZipAnyH<H1, H2>, D>;

    /// Zips any of the two hazard interfaces.
    ///
    /// Ingress transfers and an egress transfer happen as soon as any of the ingress transfer conditions are satisfied.
    /// Note that the ingress transfers happen only for the interfaces whose transfer condition is satisfied.
    ///
    /// To achieve this, the egress interface's hazard is `ZipAnyH` with ready condition "any of the ingress interfaces'
    /// transfer conditions are true".
    ///
    /// - Payloads: Wrapped in another `HOption`. The outer `HOption` is `Some` if any of the payloads are `Some`.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress                            | Egress                                      |
    /// | :-------: | ---------------------------------- | ------------------------------------------- |
    /// |  **Fwd**  | `(HOption<H1::P>, HOption<H2::P>)` | `HOption<(HOption<H1::P>, HOption<H2::P>)>` |
    /// |  **Bwd**  | `(H1::R, H2::R)`                   | `(H1::R, H2::R)`                            |
    fn zip_any(self) -> I<ZipAnyH<H1, H2>, D> {
        unsafe {
            self.fsm::<I<ZipAnyH<H1, H2>, D>, ()>((), |(ip1, ip2), er, s| {
                let ep = if ip1.is_some() || ip2.is_some() { Some((ip1, ip2)) } else { None };
                (ep, er, s)
            })
        }
    }
}

// TODO: Add 4 to 12-tuple variants.

/// Hazard specification for zip-any with 3 interfaces.
#[derive(Debug, Clone, Copy)]
pub struct ZipAny3H<H1: Hazard, H2: Hazard, H3: Hazard> {
    _marker: PhantomData<(H1, H2, H3)>,
}

impl<H1: Hazard, H2: Hazard, H3: Hazard> Hazard for ZipAny3H<H1, H2, H3> {
    type P = (HOption<H1::P>, HOption<H2::P>, HOption<H3::P>);
    type R = (H1::R, H2::R, H3::R);

    fn ready((p1, p2, p3): Self::P, (r1, r2, r3): Self::R) -> bool {
        p1.is_some_and(|p| H1::ready(p, r1))
            || p2.is_some_and(|p| H2::ready(p, r2))
            || p3.is_some_and(|p| H3::ready(p, r3))
    }
}

impl<H1: Hazard, H2: Hazard, H3: Hazard, const D: Dep> ZipAnyExt for (I<H1, D>, I<H2, D>, I<H3, D>) {
    type E = I<ZipAny3H<H1, H2, H3>, D>;

    /// A variation of [`zip_any`] for 3 hazard interfaces. See the 2-tuple version for more information.
    fn zip_any(self) -> I<ZipAny3H<H1, H2, H3>, D> {
        unsafe {
            self.fsm::<I<ZipAny3H<H1, H2, H3>, D>, ()>((), |(ip1, ip2, ip3), er, s| {
                let ep = if ip1.is_some() || ip2.is_some() || ip3.is_some() { Some((ip1, ip2, ip3)) } else { None };
                (ep, er, s)
            })
        }
    }
}

/// Extension trait for `zip_any_valid`.
pub trait ZipAnyValidExt: Interface {
    /// Egress interface.
    type E: Interface;

    /// Zip-any valid.
    fn zip_any_valid(self) -> Self::E;
}

impl<P1: Copy, P2: Copy> ZipAnyValidExt for (Valid<P1>, Valid<P2>) {
    type E = Valid<(HOption<P1>, HOption<P2>)>;

    /// Zips any of the two valid interfaces.
    ///
    /// - Payloads: Wrapped in another `HOption`. The outer `HOption` is `Some` if any of the payloads are `Some`.
    /// - Resolver: The resolver carries no information.
    ///
    /// | Interface | Ingress                      | Egress                                |
    /// | :-------: | ---------------------------- | ------------------------------------- |
    /// |  **Fwd**  | `(HOption<P1>, HOption<P2>)` | `HOption<(HOption<P1>, HOption<P2>)>` |
    /// |  **Bwd**  | `((), ())`                   | `()`                                  |
    fn zip_any_valid(self) -> Valid<(HOption<P1>, HOption<P2>)> {
        unsafe {
            self.fsm((), |(l, r), (), ()| (if l.is_some() || r.is_some() { Some((l, r)) } else { None }, ((), ()), ()))
        }
    }
}

macro_rules! impl_valid_zip_any_valid {
    ($($P:ident),+) => {
        impl<$($P: Copy,)+> ZipAnyValidExt for ($(Valid<$P>,)+) {
            type E = Valid<($(HOption<$P>,)+)>;

            /// A variation of [`zip_any_valid`](zip_any) for 3-12 valid interfaces. See the 2-tuple version for more
            /// information.
            fn zip_any_valid(self) -> Valid<($(HOption<$P>,)+)> {
                unsafe {
                    // This is a hack that uses `P1`, `P2`, ... as variable names.
                    #[allow(non_snake_case)]
                    self.fsm((), |($($P,)+), (), ()| {
                        (if $($P.is_some())||+ { Some(($($P,)+)) } else { None }, ($(replace!($P, ()),)+), ())
                    })
                }
            }
        }
    };
}

impl_valid_zip_any_valid! { P1, P2, P3 }
impl_valid_zip_any_valid! { P1, P2, P3, P4 }
impl_valid_zip_any_valid! { P1, P2, P3, P4, P5 }
impl_valid_zip_any_valid! { P1, P2, P3, P4, P5, P6 }
impl_valid_zip_any_valid! { P1, P2, P3, P4, P5, P6, P7 }
impl_valid_zip_any_valid! { P1, P2, P3, P4, P5, P6, P7, P8 }
impl_valid_zip_any_valid! { P1, P2, P3, P4, P5, P6, P7, P8, P9 }
impl_valid_zip_any_valid! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10 }
impl_valid_zip_any_valid! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11 }
impl_valid_zip_any_valid! { P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12 }
