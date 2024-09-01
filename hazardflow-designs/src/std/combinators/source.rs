//! Source.

use super::*;

impl<P: Copy> I<ValidH<P, P>, { Dep::Demanding }> {
    /// A source that returns the data coming from the resolver to the payload.
    ///
    /// - Payload: Outputs the resolver data. Always valid.
    /// - Resolver: Returned to the payload.
    ///
    /// | Interface | Egress       |
    /// | :-------: | ------------ |
    /// |  **Fwd**  | `HOption<P>` |
    /// |  **Bwd**  | `P`          |
    pub fn source() -> I<ValidH<P, P>, { Dep::Demanding }> {
        I::source_map_drop(Some)
    }
}

impl<P: Copy> I<VrH<P, P>, { Dep::Demanding }> {
    /// A source that returns the data coming from the resolver to the payload.
    ///
    /// - Payload: Outputs the resolver data. Dropped if the egress ready signal is false.
    /// - Resolver: Returned to the payload.
    ///
    /// | Interface | Egress       |
    /// | :-------: | ------------ |
    /// |  **Fwd**  | `HOption<P>` |
    /// |  **Bwd**  | `Ready<P>`   |
    pub fn source_drop() -> I<VrH<P, P>, { Dep::Demanding }> {
        I::source_map_drop(HOption::from)
    }
}

impl<H: Hazard> I<H, { Dep::Demanding }> {
    /// A source that maps and returns the data coming from the resolver to the payload.
    ///
    /// - Payload: Outputs the resolver data. Dropped if `H::ready` is false.
    /// - Resolver: Returned to the payload.
    ///
    /// | Interface | Egress          |
    /// | :-------: | --------------- |
    /// |  **Fwd**  | `HOption<H::P>` |
    /// |  **Bwd**  | `H::R`          |
    pub fn source_map_drop(f: impl Fn(H::R) -> HOption<H::P>) -> I<H, { Dep::Demanding }> {
        unsafe {
            ().fsm((), |(), er, ()| {
                let ep = f(er).filter(|ep| H::ready(ep, er));
                (ep, (), ())
            })
        }
    }
}

impl<P: Copy> Valid<P> {
    /// A constant signal.
    ///
    /// - Payload: Always valid.
    /// - Resolver: The resolver carries no information.
    ///
    /// | Interface | Egress       |
    /// | :-------: | ------------ |
    /// |  **Fwd**  | `HOption<P>` |
    /// |  **Bwd**  | `()`         |
    pub fn constant(value: P) -> Self {
        unsafe { ().fsm((), |_, _, _| (Some(value), (), ())) }
    }
}

impl<P: Copy> Vr<P> {
    /// A constant signal.
    ///
    /// - Payload: Always valid.
    /// - Resolver: Ignored.
    ///
    /// | Interface | Egress       |
    /// | :-------: | ------------ |
    /// |  **Fwd**  | `HOption<P>` |
    /// |  **Bwd**  | `Ready<()>`  |
    pub fn constant(value: P) -> Vr<P> {
        unsafe { ().fsm((), |_, _, _| (Some(value), (), ())) }
    }
}
