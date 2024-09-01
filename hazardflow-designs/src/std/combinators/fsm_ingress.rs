//! FSM ingress.

use super::*;

impl<P: Copy, R: Copy, const D: Dep> I<VrH<P, R>, D> {
    /// Runs a finite state machine described by `f`, accepting successive ingress payloads until `f` returns true for
    /// `done_next`. Then, outputs the resulting FSM state and reset.
    ///
    /// This allows you to accumulate successive ingress payloads into the internal FSM state until it is ready to be
    /// transmitted.
    ///
    /// - Payload: Controlled by the combinator's behavior.
    /// - Resolver: The ingress ready signal is controlled by the combinator's behavior. The inner value `R` of the
    ///     resolver is preserved.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<S>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<R>`   |
    ///
    /// # Detailed explanation
    ///
    /// ## Parameters
    ///
    /// - `init`: The initial state for the FSM.
    /// - `f`: The function that describes the FSM. If `let (s_next, done_next) = f(p, r, s)`,
    ///     - `p`: The ingress payload.
    ///     - `r`: The inner value of the egress resolver.
    ///     - `s`: The current FSM state.
    ///     - `s_next`: The next FSM state.
    ///     - `done_next`: Whether the FSM is done accumulating.
    ///
    /// ## High-level overview of the behavior
    ///
    /// The combinator can be in one of the two cases: **Accumulating** and **Outputting**.
    ///
    /// The combinator is initially in the **Acumulating** case. In this case, it keeps accepting ingress payloads and
    /// runs the FSM, allowing it to accumulate the payloads. When `f` returns true for `done_next`, the combinator will
    /// transition to the **Outputting** case next cycle.
    ///
    /// In the **Outputting** case, the combinator outputs the resulting FSM state and blocks the ingress. When the
    /// egress transfer of the state happens, it will transition back to the **Accumulating** case next cycle.
    ///
    /// ## Detailed behavior
    ///
    /// > NOTE: The description below assumes the following naming convention. Here, `s` is the FSM state, and `done`
    /// > represents the current case the combinator is in. The description is organized sligtly differently from the
    /// > actual implementation for better clarity.
    /// >
    /// > ```ignore
    /// > // an implementation of `fsm_ingress`
    /// > self.fsm((init, false), |ip, er, (s, done)| {
    /// >     // ... (the description below would fit here)
    /// >     (ep, ir, (s_next, done_next))
    /// > })
    /// > ```
    ///
    /// - **Accumulating** (`done == false`)
    ///     - Do not produce an egress payload: `ep = None`
    ///     - Accept ingress payloads: `ir = (true, er.inner)`
    ///     - If an ingress transfer happens (`if it`),
    ///         - Run `f`: `let (s_next_f, done_next_f) = f(ip.unwrap(), er.inner, s)`
    ///         - Update the FSM state next cycle: `s_next = s_next_f`
    ///         - Remain in the current case or transition to the **Outputting** case next cycle, depending on the
    ///             returned value: `done_next = done_next_f`
    ///     - If no ingress transfer happens (`else`),
    ///         - Do not update the FSM state: `s_next = s`
    ///         - Remain in the current case: `done_next = done`
    /// - **Outputting** (`done == true`)
    ///     - Output the FSM state: `ep = Some(s)`
    ///     - Block ingress payloads: `ir = (false, er.inner)`
    ///     - If an egress transfer happens (`if et`)
    ///         - Reset the FSM state next cycle: `s_next = init`
    ///         - Transition to the **Accumulating** case next cycle: `done_next = false`
    ///     - If no egress transfer happens (`else`)
    ///         - Do not change the FSM state: `s_next = s`
    ///         - Remain in the current case: `done_next = done`
    pub fn fsm_ingress<S: Copy>(self, init: S, f: impl Fn(P, R, S) -> (S, bool)) -> I<VrH<S, R>, { Dep::Helpful }> {
        unsafe {
            self.fsm::<(S, bool), { Dep::Helpful }, VrH<S, R>>((init, false), |ip, er, (s, done)| {
                let ir = Ready::new(!done, er.inner);

                let it = ip.is_some() && !done;
                let et = er.ready && done;

                let ep = if done { Some(s) } else { None };

                let s_next = if it {
                    f(ip.unwrap(), er.inner, s)
                } else if et {
                    (init, false)
                } else {
                    (s, done)
                };

                (ep, ir, s_next)
            })
        }
    }
}
