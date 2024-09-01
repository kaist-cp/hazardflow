//! FSM egress.

use super::*;

impl<P: Copy, R: Copy, const D: Dep> I<VrH<P, R>, D> {
    /// For each transferred ingress payload, runs a finite state machine described by `f` until `f` returns true for
    /// `is_last`.
    ///
    /// This allows you to process each ingress payload using multiple FSM states.
    ///
    /// - Payload: Controlled by the combinator's behavior.
    /// - Resolver: The ingress ready signal is controlled by the combinator's behavior. The inner value `R` of the
    ///     resolver is preserved.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<R>`    |
    ///
    /// # Detailed explanation
    ///
    /// ## Parameters
    ///
    /// - `init`: The initial state for the FSM.
    /// - `flow`: If true, starts running the FSM immediately from the cycle of an ingress transfer. If false, starts
    ///     running the FSM from the next cycle of an ingress transfer.
    /// - `f`: The function that describes the FSM. If `let (ep, s_next, is_last) = f(p, s)`,
    ///     - `p`: The current saved ingress payload for this FSM.
    ///     - `s`: The current FSM state.
    ///     - `ep`: The calculated egress payload.
    ///     - `s_next`: The next FSM state.
    ///     - `is_last`: Whether this is the last state of the FSM for the current saved payload.
    ///
    /// ## High-level overview of the behavior
    ///
    /// The combinator can be in one of the two cases: **Waiting for an ingress transfer** and **Running the FSM**.
    ///
    /// Initially, the combinator is in the **Waiting for an ingress transfer** case and waits for an ingress transfer
    /// to happen. Once an ingress transfer happens, it saves the ingress payload to transition to the **Running the
    /// FSM** case, and starts a new FSM. Whether this happens on the same cycle as the transfer or on the next cycle
    /// depends on `flow`.
    ///
    /// In the **Running the FSM** case, the combinator runs the FSM with the saved payload, outputting egress payloads.
    /// When `f` returns true for `is_last`, it will save a new ingress payload and start a new FSM if the new ingress
    /// payload is already available. Otherwise, it will transition back to the **Waiting for an ingress transfer**
    /// case next cycle.
    ///
    /// ## Detailed behavior
    ///
    /// > NOTE: The description below assumes the following naming convention. Here, `sp` is the saved ingress payload
    /// > and determines the case the combinator is in. `s` is the FSM state. The description is organized sligtly
    /// > differently from the actual implementation for better clarity.
    /// >
    /// > ```ignore
    /// > // an implementation of `fsm_egress`
    /// > self.fsm((None, init), |ip, er, (sp, s)| {
    /// >     // ... the description below would fit here
    /// >     (ep, ir, (sp_next, s_next))
    /// > })
    /// > ```
    ///
    /// - **Waiting for an ingress transfer** (`sp == None`)
    ///     - Do not produce an egress payload: `ep = None`
    ///     - Can accept a new ingress payload: `ir = (true, er.inner)`
    ///     - If an ingress transfer happens (`if it`),
    ///         - If `flow`,
    ///             - Conceptually save the ingress payload and transition to the **Running the FSM** case *this* cycle:
    ///                 `sp = ip`
    ///             - Start a new FSM with the initial state *this* cycle: `s = init`
    ///             - Run the logic for the **Running the FSM** case, except for `ir` which remains `(true, er.inner)`.
    ///                 - Output the calculated egress payload: `ep = Some(ep_f)`
    ///                 - The case next cycle is determined by the logic: `sp_next = ...`
    ///                 - The FSM state next cycle is determined by the logic: `s_next = ...`
    ///         - If not `flow`,
    ///             - Save the ingress payload to transition to the **Running the FSM** case next cycle: `sp_next = ip`
    ///             - Start a new FSM with the initial state next cycle: `s_next = init`
    ///     - If no ingress transfer happens (`else`),
    ///         - Remain in the current case: `sp_next = sp`
    ///         - Do not change the FSM state: `s_next = s`
    /// - **Running the FSM** (`sp == Some`)
    ///     - Run `f`: `let (ep_f, s_next_f, is_last) = f(sp.unwrap(), s)`
    ///     - Output the calculated egress payload: `ep = Some(ep_f)`
    ///     - Let's say that "the FSM finishes" if an egress transfer happens and this is the last state of the FSM
    ///         (`et && is_last`).
    ///     - Do not accept a new ingress payload, but can accept one if the FSM finishes:
    ///         `ir = (et && is_last, er.inner)`
    ///     - If the FSM finishes and an ingress transfer happens (`if it`),
    ///         - Save the new ingress payload and remain in the current case: `sp_next = ip`
    ///         - Start a new FSM with the initial state next cycle: `s_next = init`
    ///     - If the FSM finishes but no ingress transfer happens (`if et && is_last`),
    ///         - Remove the saved payload to transition to the **Waiting for an ingress transfer** case next cycle:
    ///             `sp_next = None`
    ///         - Reset the FSM state next cycle: `s_next = init`
    ///     - If an egress transfer happens but this is not the last state of the FSM (`else if et`),
    ///         - Remain in the current case: `sp_next = sp`
    ///         - Update the FSM state next cycle: `s_next = s_next_f`
    ///     - If no egress transfer happens (`else`),
    ///         - Remain in the current case: `sp_next = sp`
    ///         - Do not update the FSM state: `s_next = s`
    pub fn fsm_egress<EP: Copy, S: Copy>(
        self,
        init: S,
        flow: bool,
        f: impl Fn(P, S) -> (EP, S, bool),
    ) -> I<VrH<EP, R>, D> {
        unsafe {
            self.fsm::<(HOption<P>, S), D, VrH<EP, R>>((None, init), |ip, er, (sp, s)| {
                let (ep, s_next, is_last) = if let Some(p) = sp {
                    let (ep, s_next, is_last) = f(p, s);
                    (Some(ep), s_next, is_last)
                } else if flow && ip.is_some() && sp.is_none() {
                    let (ep, s_next, is_last) = f(ip.unwrap(), s);
                    (Some(ep), s_next, is_last)
                } else {
                    (None, s, false)
                };

                let et = ep.is_some() && er.ready;
                let ir = Ready::new(sp.is_none() || (et && is_last), er.inner);
                let it = ip.is_some() && ir.ready;

                let (sp_next, s_next) = if flow && it && et && sp.is_none() {
                    if is_last {
                        (None, init)
                    } else {
                        (ip, s_next)
                    }
                } else if it {
                    (ip, init)
                } else if et && is_last {
                    (None, init)
                } else if et {
                    (sp, s_next)
                } else {
                    (sp, s)
                };

                (ep, ir, (sp_next, s_next))
            })
        }
    }
}
