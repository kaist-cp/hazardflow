//! Generator

use super::*;
use crate::prelude::*;

impl<const D: Dep, P: Copy, R: Copy> I<ValidH<P, R>, D> {
    /// Generator.
    ///
    /// TODO: Documentation
    pub fn generator<EP: Copy, S: Copy>(
        self,
        init: S,
        f_write_state: impl Fn(P, R, S) -> S,
        f_read_state: impl Fn(S) -> HOption<EP>,
    ) -> I<ValidH<EP, R>, { Dep::Helpful }> {
        unsafe {
            self.fsm::<_, { Dep::Helpful }, ValidH<EP, R>>(init, |ip, er, s| {
                let ep = f_read_state(s);

                let s_next = match ip {
                    Some(ip) => f_write_state(ip, er, s),
                    _ => s,
                };

                (ep, er, s_next)
            })
        }
    }
}

impl<const D: Dep, P: Copy, R: Copy> I<VrH<P, R>, D> {
    /// Generator.
    ///
    /// # Input
    /// - `init`: Initial state.
    /// - `f_write_state`: Write state function. This will set the state when ingress ingress_transfer happens.
    /// - `f_read_state`: Read state function. This will read the state and trigger egress transfer
    /// when egess ready is `true` and the function returns `Some`. Also, the boolean value
    /// returned by this function will work as the ready signal for the ingress transfer(i.e.,
    /// backpressure).
    pub fn generator<EP: Copy, S: Copy>(
        self,
        init: S,
        f_write_state: impl Fn(P, R, S) -> S,
        f_read_state: impl Fn(S) -> (HOption<EP>, bool),
    ) -> I<VrH<EP, R>, { Dep::Helpful }> {
        unsafe {
            self.fsm::<_, { Dep::Helpful }, VrH<EP, R>>(init, |ip, er, s| {
                let (ep, ready) = f_read_state(s);

                let s_next = match ip {
                    Some(ip) if ready => f_write_state(ip, er.inner, s),
                    _ => s,
                };

                let ir = Ready::new(ready, er.inner);

                (ep, ir, s_next)
            })
        }
    }
}
