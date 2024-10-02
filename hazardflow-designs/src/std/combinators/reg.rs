//! Reg.

use super::*;

impl<P: Copy, R: Copy, const D: Dep> I<ValidH<P, R>, D> {
    /// A 1-cycle buffer for payloads.
    ///
    /// - Payload: Buffered by 1 cycle.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `R`          | `R`          |
    pub fn reg_fwd_always(self) -> I<ValidH<P, R>, { Dep::Helpful }> {
        self.shift_reg_fwd::<1>()
    }

    /// A register that is enabled only if the payload is valid.
    ///
    /// - Payload: Only valid payloads are stored, and the stored payload will keep being outputted until a new valid
    ///     payload comes in.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `R`          | `R`          |
    pub fn reg_fwd_valid(self) -> I<ValidH<P, R>, { Dep::Helpful }> {
        unsafe {
            self.fsm::<HOption<P>, { Dep::Helpful }, ValidH<P, R>>(None, |ip, er, s| {
                let ep = s;
                let ir = er;
                let s_next = if ip.is_some() { ip } else { s };
                (ep, ir, s_next)
            })
        }
    }

    /// A [shift register](https://en.wikipedia.org/wiki/Shift_register) for payloads, with `LATENCY`-cycle latency.
    ///
    /// - Payload: Stored, and outputted after `LATENCY` cycles.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `R`          | `R`          |
    ///
    /// Currently only supports SISO.
    // TODO: support other types (SIPO, PISO)
    pub fn shift_reg_fwd<const LATENCY: usize>(self) -> I<ValidH<P, R>, { Dep::Helpful }>
    where [(); 1 + LATENCY]: {
        unsafe {
            self.fsm::<Array<HOption<P>, LATENCY>, { Dep::Helpful }, ValidH<P, R>>(None.repeat(), |ip, er, s| {
                let new_s = ip.repeat::<1>().append(s).clip_const::<LATENCY>(0);
                (s[LATENCY - 1], er, new_s)
            })
        }
    }
}

impl<P: Copy, R: Copy, const D: Dep> I<VrH<P, R>, D> {
    /// A register for a `VrH` hazard interface.
    ///
    /// If `pipe` is true, payloads can be pushed into the state with full throughput. Specifically, the register can
    /// accept a new payload in the same cycle as an egress transfer.
    ///
    /// - Payload: Stored after an ingress transfer happens. The stored payload is outputted, and cleared after an
    ///     egress transfer happens.
    /// - Resolver: The ingress ready signal is true when nothing is stored in the state (or additionally when an egress
    ///     transfer is happening if `pipe` is true). The inner value `R` of the resolver is preserved.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<R>`   |
    pub fn reg_fwd(self, pipe: bool) -> I<VrH<P, R>, { Dep::Helpful }> {
        self.map_resolver_inner::<(R, HOption<P>)>(|(r, _)| r).transparent_reg_fwd(pipe)
    }

    /// A variant of [`I::reg_fwd`] that takes the initial value of the state.
    ///
    /// - Payload: The same behavior as [`I::reg_fwd`].
    /// - Resolver: The same behavior as [`I::reg_fwd`].
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<R>`   |
    pub fn reg_fwd_with_init(self, pipe: bool, init: P) -> I<VrH<P, R>, { Dep::Helpful }> {
        self.map_resolver_inner::<(R, HOption<P>)>(|(r, _)| r).transparent_reg_fwd_with_opt_init(pipe, Some(init))
    }
}

impl<P: Copy, const D: Dep> Vr<P, D> {
    /// A register that is enabled only if the egress ready signal is true.
    ///
    /// - Payload: Outputs the stored payload.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `Ready<()>`  | `Ready<()>`  |
    pub fn reg_fwd_ready(self) -> Vr<P> {
        unsafe {
            self.fsm::<HOption<P>, { Dep::Helpful }, VrH<P>>(None, |ip, er, s| {
                let s_next = if ip.is_some() && er.ready {
                    ip
                } else if s.is_some() && er.ready {
                    None
                } else {
                    s
                };
                (s, er, s_next)
            })
        }
    }

    /// A backward register for the resolver ready signal.
    ///
    /// - Payload: Preserved.
    /// - Resolver: The egress ready signal is stored, and outputted after 1 cycle.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `Ready<()>`  | `Ready<()>`  |
    pub fn reg_bwd(self) -> Vr<P> {
        unsafe {
            self.fsm::<bool, { Dep::Helpful }, VrH<P>>(false, |ip, er, s| {
                let ep = ip;
                let ir = Ready::new(s, ());
                let s_next = er.ready;
                (ep, ir, s_next)
            })
        }
    }

    /// A backward register for the resolver ready signal that drops the payload if the stored ready signal is false.
    ///
    /// - Payload: Dropped if the stored ready signal is false.
    /// - Resolver: The egress ready signal is stored, and outputted after 1 cycle.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `Ready<()>`  | `Ready<()>`  |
    pub fn reg_bwd_drop(self) -> Vr<P> {
        unsafe {
            self.fsm::<bool, { Dep::Helpful }, VrH<P>>(false, |ip, er, s| {
                let ep = ip.filter(|_| s);
                let ir = Ready::new(s, ());
                let s_next = er.ready;
                (ep, ir, s_next)
            })
        }
    }
}

impl<const D: Dep, P: Copy, R: Copy, H: Hazard<P = P, R = (R, HOption<P>)>> I<AndH<H>, D> {
    /// A register for an `AndH` hazard interface.
    ///
    /// If `pipe` is true, payloads can be pushed into the state with full throughput. Specifically, the register can
    /// accept a new payload in the same cycle as an egress transfer.
    ///
    /// - Payload: Stored after an ingress transfer happens. The stored payload is outputted, and cleared after an
    ///     egress transfer happens.
    /// - Resolver: The ingress ready signal is true when nothing is stored in the state (or additionally when an egress
    ///     transfer is happening if `pipe` is true). The inner value `R` of the resolver is preserved, and additionally
    ///     the internal state `HOption<P>` is outputted to the ingress resolver.
    ///
    /// | Interface | Ingress                  | Egress       |
    /// | :-------: | ------------------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>`             | `HOption<P>` |
    /// |  **Bwd**  | `Ready<(R, HOption<P>)>` | `Ready<R>`   |
    pub fn transparent_reg_fwd<EH: Hazard<P = P, R = R>>(self, pipe: bool) -> I<AndH<EH>, { Dep::Helpful }> {
        self.transparent_reg_fwd_with_opt_init(pipe, None)
    }

    /// A variant of [`I::transparent_reg_fwd`] that takes the initial value of the state.
    ///
    /// - Payload: The same behavior as [`I::transparent_reg_fwd`].
    /// - Resolver: The same behavior as [`I::transparent_reg_fwd`].
    ///
    /// | Interface | Ingress                  | Egress       |
    /// | :-------: | ------------------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>`             | `HOption<P>` |
    /// |  **Bwd**  | `Ready<(R, HOption<P>)>` | `Ready<R>`   |
    pub fn transparent_reg_fwd_with_init<EH: Hazard<P = P, R = R>>(
        self,
        pipe: bool,
        init: P,
    ) -> I<AndH<EH>, { Dep::Helpful }> {
        self.transparent_reg_fwd_with_opt_init(pipe, Some(init))
    }

    fn transparent_reg_fwd_with_opt_init<EH: Hazard<P = P, R = R>>(
        self,
        pipe: bool,
        init: HOption<P>,
    ) -> I<AndH<EH>, { Dep::Helpful }> {
        unsafe {
            self.fsm::<HOption<P>, { Dep::Helpful }, AndH<EH>>(init, |ip, er, s| {
                // Egress transfer happens?
                let ep = s;
                let et = ep.is_some_and(|p| er.ready && EH::ready(p, er.inner));

                let ir = if pipe {
                    Ready::new(s.is_none() || et, (er.inner, s))
                } else {
                    Ready::new(s.is_none(), (er.inner, s))
                };
                let it = ip.is_some_and(|p| ir.ready && H::ready(p, ir.inner));

                let s_next = if it {
                    ip
                } else if et {
                    None
                } else {
                    s
                };

                (ep, ir, s_next)
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SkidS<P: Copy> {
    /// Directly connected to module output.
    m_axis_data: HOption<P>,
    /// Temp register of skid buffer.
    temp_m_axis_data: HOption<P>,
    /// Datapath control.
    m_axis_ready_int: bool,
}

impl<P: Copy> Default for SkidS<P> {
    fn default() -> Self {
        SkidS { m_axis_data: None, temp_m_axis_data: None, m_axis_ready_int: false }
    }
}

impl<P: Copy, const D: Dep> Vr<P, D> {
    /// A skid-buffer for a valid-ready interface.
    ///
    /// | Interface | Ingress                  | Egress       |
    /// | :-------: | ------------------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>`             | `HOption<P>` |
    /// |  **Bwd**  | `Ready<()>`              | `Ready<()>`  |
    pub fn reg_skid(self) -> Vr<P> {
        unsafe {
            self.fsm::<SkidS<P>, { Dep::Helpful }, VrH<P>>(SkidS::default(), |ip, er, s| {
                let skid_buffer_data_int = ip.unwrap();
                let skid_buffer_valid_int = ip.is_some();

                let m_axis_data_reg = s.m_axis_data.unwrap();
                let m_axis_valid_reg = s.m_axis_data.is_some();
                let temp_m_axis_data_reg = s.temp_m_axis_data.unwrap();
                let temp_m_axis_valid_reg = s.temp_m_axis_data.is_some();
                let m_axis_ready_int_reg = s.m_axis_ready_int;

                let m_axis_ready = er.ready;

                let m_axis_valid_int = skid_buffer_valid_int && m_axis_ready_int_reg;
                let m_axis_ready_int_early =
                    m_axis_ready || (!temp_m_axis_valid_reg && (!m_axis_valid_reg || !m_axis_valid_int));

                let store_axis_int_to_output = m_axis_ready_int_reg & m_axis_ready | !m_axis_valid_reg;
                let store_axis_int_to_temp = m_axis_ready_int_reg & !m_axis_ready & m_axis_valid_reg;
                let store_axis_temp_to_output = !m_axis_ready_int_reg & m_axis_ready;

                let m_axis_data_next = if store_axis_int_to_output {
                    skid_buffer_data_int
                } else if store_axis_temp_to_output {
                    temp_m_axis_data_reg
                } else {
                    m_axis_data_reg
                };
                let temp_m_axis_data_next =
                    if store_axis_int_to_temp { skid_buffer_data_int } else { temp_m_axis_data_reg };

                let m_axis_valid_next = if m_axis_ready_int_reg {
                    if m_axis_ready || !m_axis_valid_reg {
                        m_axis_valid_int
                    } else {
                        m_axis_valid_reg
                    }
                } else if m_axis_ready {
                    temp_m_axis_valid_reg
                } else {
                    m_axis_valid_reg
                };
                let temp_m_axis_valid_next = if m_axis_ready_int_reg {
                    if m_axis_ready || !m_axis_valid_reg {
                        temp_m_axis_valid_reg
                    } else {
                        m_axis_valid_int
                    }
                } else {
                    !m_axis_ready && temp_m_axis_valid_reg
                };

                let s_next = SkidS {
                    m_axis_data: if m_axis_valid_next { Some(m_axis_data_next) } else { None },
                    temp_m_axis_data: if temp_m_axis_valid_next { Some(temp_m_axis_data_next) } else { None },
                    m_axis_ready_int: m_axis_ready_int_early,
                };

                (s.m_axis_data, Ready::new(s.m_axis_ready_int, ()), s_next)
            })
        }
    }
}
