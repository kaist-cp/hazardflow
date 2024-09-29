//! Mux.

use super::*;

/// Extension trait for `mux`.
pub trait MuxExt<const N: usize>: Interface
where [(); clog2(N)]:
{
    /// Egress interface.
    type E: Interface;

    /// Mux.
    fn mux(self, cntl: Valid<U<{ clog2(N) }>>) -> Self::E;
}

impl<P: Copy, R: Copy, const N: usize, const D: Dep> MuxExt<N> for [I<ValidH<P, R>, D>; N]
where [(); clog2(N)]:
{
    type E = I<ValidH<P, R>, D>;

    /// Muxes `N` `ValidH` hazard interfaces based on `cntl`.
    ///
    /// `cntl` selects which ingress interface to connect to the egress interface.
    ///
    /// - Payloads: Outputs the payload of the interface selected by `cntl`.
    /// - Resolver: Duplicated to multiple interfaces.
    ///
    /// | Interface | Ingress                | Egress       |
    /// | :-------: | ---------------------- | ------------ |
    /// |  **Fwd**  | `Array<HOption<P>, N>` | `HOption<P>` |
    /// |  **Bwd**  | `Array<R, N>`          | `R`          |
    fn mux(self, cntl: Valid<U<{ clog2(N) }>>) -> I<ValidH<P, R>, D> {
        unsafe {
            (self, cntl).fsm::<I<ValidH<P, R>, D>, ()>((), |(ip, sel), er, s| {
                let ep = sel.and_then(|sel| ip[sel]);
                let ir = er.repeat::<N>();

                (ep, (ir, ()), s)
            })
        }
    }
}

impl<P: Copy, R: Copy, const N: usize, const D: Dep> MuxExt<N> for [I<VrH<P, R>, D>; N]
where [(); clog2(N)]:
{
    type E = I<VrH<P, R>, D>;

    /// Muxes `N` `VrH` hazard interfaces based on `cntl`.
    ///
    /// `cntl` selects which ingress interface to connect to the egress interface.
    ///
    /// - Payloads: Outputs the payload of the interface selected by `cntl`.
    /// - Resolver: The selected interface's resolver follows the egress resolver. All the other resolvers are invalid.
    ///
    /// | Interface | Ingress                | Egress       |
    /// | :-------: | ---------------------- | ------------ |
    /// |  **Fwd**  | `Array<HOption<P>, N>` | `HOption<P>` |
    /// |  **Bwd**  | `Array<Ready<R>, N>`   | `Ready<R>`   |
    fn mux(self, cntl: Valid<U<{ clog2(N) }>>) -> I<VrH<P, R>, D> {
        unsafe {
            (self, cntl).fsm::<I<VrH<P, R>, D>, ()>((), |(ip, sel), er, s| {
                let ep = sel.and_then(|sel| ip[sel]);
                let ir = if let Some(sel) = sel {
                    Ready::invalid().repeat::<N>().set(sel, er)
                } else {
                    Ready::invalid().repeat::<N>()
                };

                (ep, (ir, ()), s)
            })
        }
    }
}

impl<P: Copy, R1: Copy, R2: Copy, const D: Dep> MuxExt<2> for (I<VrH<P, R1>, D>, I<VrH<P, R2>, D>) {
    type E = I<VrH<P, (R1, R2)>, D>;

    /// Muxes two `VrH` hazard interfaces based on `cntl`.
    ///
    /// `cntl` selects which ingress interface to connect to the egress interface.
    ///
    /// - Payloads: Outputs the payload of the interface selected by `cntl`.
    /// - Resolver: The selected interface's resolver follows the egress resolver. All the other resolvers are invalid.
    ///
    /// | Interface | Ingress                    | Egress            |
    /// | :-------: | -------------------------- | ----------------- |
    /// |  **Fwd**  | `(HOption<P>, HOption<P>)` | `HOption<P>`      |
    /// |  **Bwd**  | `(Ready<R1>, Ready<R2>)`   | `Ready<(R1, R2)>` |
    fn mux(self, cntl: Valid<U<{ clog2(2) }>>) -> I<VrH<P, (R1, R2)>, D> {
        unsafe {
            (self, cntl).fsm::<I<VrH<P, (R1, R2)>, D>, ()>((), |((ip1, ip2), sel), er, s| {
                let ep = sel.and_then(|sel| if sel == 0.into_u() { ip1 } else { ip2 });
                let ir1 = if sel.filter(|p| p == 0.into_u()).is_some() {
                    er.map(|r| r.0)
                } else {
                    Ready::new(false, er.inner.0)
                };
                let ir2 = if sel.filter(|p| p == 1.into_u()).is_some() {
                    er.map(|r| r.1)
                } else {
                    Ready::new(false, er.inner.1)
                };

                (ep, ((ir1, ir2), ()), s)
            })
        }
    }
}
