//! FSM mapping combinators.

use super::*;

impl<P: Copy, R: Copy, const D: Dep> I<ValidH<P, R>, D> {
    /// A [`map`] with an internal state.
    ///
    /// `f` additionally takes the current state and returns the next state. The state is updated if the ingress payload
    /// is valid and so an ingress transfer happens.
    ///
    /// - Payload: Mapped by `f`.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `R`          | `R`           |
    pub fn fsm_map<EP: Copy, S: Copy>(self, init_state: S, f: impl Fn(P, S) -> (EP, S)) -> I<ValidH<EP, R>, D> {
        self.map_resolver::<(R, S)>(|(r, _)| r).naked_fsm_map(init_state, f)
    }

    /// A [`filter_map`] with an internal state.
    ///
    /// `f` additionally takes the current state and returns the next state. The state is updated if the ingress payload
    /// is valid and so an ingress transfer happens.
    ///
    /// - Payload: Filter-mapped by `f`.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `R`          | `R`           |
    pub fn fsm_filter_map<EP: Copy, S: Copy>(
        self,
        init: S,
        f: impl Fn(P, S) -> (HOption<EP>, S),
    ) -> I<ValidH<EP, R>, D> {
        self.map_resolver::<(R, S)>(|(r, _)| r).naked_fsm_filter_map(init, f)
    }
}

impl<P: Copy, R: Copy, S: Copy, const D: Dep> I<ValidH<P, (R, S)>, D> {
    /// A variation of [`fsm_map`] that attaches an additional internal state signal to the ingress resolver.
    ///
    /// - Payload: Mapped by `f`.
    /// - Resolver: Preserved. The internal state `S` is additionally outputted.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `(R, S)`     | `R`           |
    pub fn naked_fsm_map<EP: Copy>(self, init_state: S, f: impl Fn(P, S) -> (EP, S)) -> I<ValidH<EP, R>, D> {
        self.naked_fsm_filter_map(init_state, |p, s| {
            let (ep, s_next) = f(p, s);
            (Some(ep), s_next)
        })
    }

    /// A variation of [`naked_fsm_map`](fsm_map) that allows `f` to additionally consider the egress resolver.
    ///
    /// - Payload: Mapped by `f`.
    /// - Resolver: Preserved. The internal state `S` is additionally outputted.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `(R, S)`     | `R`           |
    pub fn naked_fsm_map_with_r<EP: Copy>(
        self,
        init_state: S,
        f: impl Fn(P, R, S) -> (EP, S),
    ) -> I<ValidH<EP, R>, { Dep::Demanding }> {
        self.naked_fsm_filter_map_with_r(init_state, |p, r, s| {
            let (ep, s_next) = f(p, r, s);
            (Some(ep), s_next)
        })
    }

    /// A variation of [`fsm_filter_map`](fsm_map) that attaches an additional internal state signal to the ingress
    /// resolver.
    ///
    /// - Payload: Filter-mapped by `f`.
    /// - Resolver: Preserved. The internal state `S` is additionally outputted.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `(R, S)`     | `R`           |
    pub fn naked_fsm_filter_map<EP: Copy>(self, init: S, f: impl Fn(P, S) -> (HOption<EP>, S)) -> I<ValidH<EP, R>, D> {
        unsafe {
            self.fsm(init, |ip, er, s| {
                let (ep, s_next) = match ip {
                    Some(p) => f(p, s),
                    None => (None, s),
                };
                (ep, (er, s), s_next)
            })
        }
    }

    /// A variation of [`naked_fsm_filter_map`](fsm_map) that allows `f` to additionally consider the egress resolver.
    ///
    /// - Payload: Filter-mapped by `f`.
    /// - Resolver: Preserved. The internal state `S` is additionally outputted.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `(R, S)`     | `R`           |
    pub fn naked_fsm_filter_map_with_r<EP: Copy>(
        self,
        init: S,
        f: impl Fn(P, R, S) -> (HOption<EP>, S),
    ) -> I<ValidH<EP, R>, { Dep::Demanding }> {
        unsafe {
            self.fsm(init, |ip, er, s| {
                let (ep, s_next) = match ip {
                    Some(p) => f(p, er, s),
                    None => (None, s),
                };
                (ep, (er, s), s_next)
            })
        }
    }
}

impl<P: Copy, R: Copy, const D: Dep> I<VrH<P, R>, D> {
    /// A [`map`] with an internal state.
    ///
    /// `f` additionally takes the current state and returns the next state. The state is updated if an ingress transfer
    /// happens.
    ///
    /// - Payload: Mapped by `f`.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<R>`    |
    pub fn fsm_map<EP: Copy, S: Copy>(self, init_state: S, f: impl Fn(P, S) -> (EP, S)) -> I<VrH<EP, R>, D> {
        self.map_resolver_inner::<(R, S)>(|(r, _)| r).naked_fsm_map(init_state, f)
    }

    /// A [`filter_map`] with an internal state.
    ///
    /// `f` additionally takes the current state and returns the next state. The state is updated if an ingress transfer
    /// happens.
    ///
    /// - Payload: Filter-mapped by `f`.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress        |
    /// | :-------: | ------------ | ------------- |
    /// |  **Fwd**  | `HOption<P>` | `HOption<EP>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<R>`    |
    pub fn fsm_filter_map<EP: Copy, S: Copy>(self, init: S, f: impl Fn(P, S) -> (HOption<EP>, S)) -> I<VrH<EP, R>, D> {
        self.map_resolver_inner::<(R, S)>(|(r, _)| r).naked_fsm_filter_map(init, f)
    }
}

impl<P: Copy, R: Copy, S: Copy, const D: Dep> I<VrH<P, (R, S)>, D> {
    /// A variation of [`fsm_map`] that attaches an additional internal state signal to the ingress resolver.
    ///
    /// - Payload: Mapped by `f`.
    /// - Resolver: Preserved. The internal state `S` is additionally outputted.
    ///
    /// | Interface | Ingress         | Egress        |
    /// | :-------: | --------------- | ------------- |
    /// |  **Fwd**  | `HOption<P>`    | `HOption<EP>` |
    /// |  **Bwd**  | `Ready<(R, S)>` | `Ready<R>`    |
    pub fn naked_fsm_map<EP: Copy>(self, init_state: S, f: impl Fn(P, S) -> (EP, S)) -> I<VrH<EP, R>, D> {
        self.naked_fsm_filter_map(init_state, |p, s| {
            let (ep, s_next) = f(p, s);
            (Some(ep), s_next)
        })
    }

    /// A variation of [`naked_fsm_map`](fsm_map) that allows `f` to additionally consider the egress resolver.
    ///
    /// - Payload: Mapped by `f`. The payload is dropped if `er.ready` is false, even if `f` returns `Some`.
    ///     ([why?](super#notes-on-dropping-combinators))
    /// - Resolver: Preserved. The internal state `S` is additionally outputted.
    ///
    /// | Interface | Ingress         | Egress        |
    /// | :-------: | --------------- | ------------- |
    /// |  **Fwd**  | `HOption<P>`    | `HOption<EP>` |
    /// |  **Bwd**  | `Ready<(R, S)>` | `Ready<R>`    |
    pub fn naked_fsm_map_drop_with_r<EP: Copy>(
        self,
        init: S,
        f: impl Fn(P, Ready<R>, S) -> (EP, S),
    ) -> I<VrH<EP, R>, { Dep::Demanding }> {
        self.naked_fsm_filter_map_drop_with_r(init, |p, r, s| {
            let (ep, s_next) = f(p, r, s);
            (Some(ep), s_next)
        })
    }

    /// A variation of [`fsm_filter_map`](fsm_map) that attaches an additional internal state signal to the ingress
    /// resolver.
    ///
    /// - Payload: Filter-mapped by `f`.
    /// - Resolver: Preserved. The internal state `S` is additionally outputted.
    ///
    /// | Interface | Ingress         | Egress        |
    /// | :-------: | --------------- | ------------- |
    /// |  **Fwd**  | `HOption<P>`    | `HOption<EP>` |
    /// |  **Bwd**  | `Ready<(R, S)>` | `Ready<R>`    |
    pub fn naked_fsm_filter_map<EP: Copy>(self, init: S, f: impl Fn(P, S) -> (HOption<EP>, S)) -> I<VrH<EP, R>, D> {
        unsafe {
            self.fsm::<S, D, VrH<EP, R>>(init, |ip, er, s| {
                let (ep, s_next) = match ip {
                    Some(ip) => f(ip, s),
                    None => (None, s),
                };
                let ir = er.map(|r| (r, s));
                (ep, ir, if er.ready { s_next } else { s })
            })
        }
    }

    /// A variation of [`naked_fsm_filter_map`](fsm_map) that allows `f` to additionally consider the egress resolver.
    ///
    /// - Payload: Filter-mapped by `f`. The payload is dropped if `er.ready` is false, even if `f` returns `Some`.
    ///     ([why?](super#notes-on-dropping-combinators))
    /// - Resolver: Preserved. The internal state `S` is additionally outputted.
    ///
    /// | Interface | Ingress         | Egress        |
    /// | :-------: | --------------- | ------------- |
    /// |  **Fwd**  | `HOption<P>`    | `HOption<EP>` |
    /// |  **Bwd**  | `Ready<(R, S)>` | `Ready<R>`    |
    pub fn naked_fsm_filter_map_drop_with_r<EP: Copy>(
        self,
        init: S,
        f: impl Fn(P, Ready<R>, S) -> (HOption<EP>, S),
    ) -> I<VrH<EP, R>, { Dep::Demanding }> {
        unsafe {
            self.fsm::<S, { Dep::Demanding }, VrH<EP, R>>(init, |ip, er, s| {
                let (ep, s_next) = match ip {
                    Some(ip) if er.ready => f(ip, er, s),
                    _ => (None, s),
                };
                let ir = er.map(|r| (r, s));
                (ep, ir, s_next)
            })
        }
    }
}
