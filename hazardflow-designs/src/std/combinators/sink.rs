//! Sink.

use super::*;

impl<H: Hazard> I<H, { Dep::Helpful }> {
    /// A sink that maps and returns the data from the payload to the resolver.
    ///
    /// - Payload: Mapped to the resolver by `f`.
    /// - Resolver: Outputs the mapped value.
    ///
    /// | Interface | Ingress         |
    /// | :-------: | --------------- |
    /// |  **Fwd**  | `HOption<H::P>` |
    /// |  **Bwd**  | `H::R`          |
    pub fn sink_map(self, f: impl Fn(HOption<H::P>) -> H::R) {
        self.sink_fsm_map((), |ip, ()| (f(ip), ()))
    }

    /// A [`I::sink_map`] with an internal state.
    ///
    /// `f` additionally takes the current state and returns the next state. The state is updated when an ingress
    /// transfer happens.
    ///
    /// - Payload: Mapped to the resolver by `f`.
    /// - Resolver: Outputs the mapped value.
    ///
    /// | Interface | Ingress         |
    /// | :-------: | --------------- |
    /// |  **Fwd**  | `HOption<H::P>` |
    /// |  **Bwd**  | `H::R`          |
    pub fn sink_fsm_map<S: Copy>(self, init_state: S, f: impl Fn(HOption<H::P>, S) -> (H::R, S)) {
        // TODO: Write safety condition
        // TODO: ir is dependent on ip, so this might cause a loop if Dep >= SelfOnly
        unsafe {
            <Self as Interface>::fsm(self, init_state, |ip, (), s| {
                let (ir, s_next) = f(ip, s);

                let s_next = if ip.is_some_and(|p| H::ready(p, ir)) { s_next } else { s };

                ((), ir, s_next)
            })
        }
    }
}
