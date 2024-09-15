//! Multiplier.

use super::*;

/// Multiplier function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MulOp {
    /// TODO: Documentation
    Mul,
    /// TODO: Documentation
    Mulh,
    /// TODO: Documentation
    Mulhu,
    /// TODO: Documentation
    Mulhsu,
    /// TODO: Documentation
    Div,
    /// TODO: Documentation
    Rem,
    /// TODO: Documentation
    Divu,
    /// TODO: Documentation
    Remu,
}

impl MulOp {
    /// Decodes multiplier operation.
    ///
    /// It returns `cmd_mul`, `cmd_hi`, `lhs_signed`, and `rhs_signed` respectively.
    pub fn decode(self) -> (bool, bool, bool, bool) {
        let cmd_mul = matches!(self, MulOp::Mul | MulOp::Mulh | MulOp::Mulhu | MulOp::Mulhsu);
        let cmd_hi = matches!(self, MulOp::Mulh | MulOp::Mulhu | MulOp::Mulhsu | MulOp::Rem | MulOp::Remu);
        let lhs_signed = matches!(self, MulOp::Mulh | MulOp::Mulhsu | MulOp::Div | MulOp::Rem);
        let rhs_signed = matches!(self, MulOp::Mulh | MulOp::Div | MulOp::Rem);

        (cmd_mul, cmd_hi, lhs_signed, rhs_signed)
    }
}

/// Multiplier request.
#[derive(Debug, Clone, Copy)]
pub struct MulReq {
    /// Operation type.
    pub op: MulOp,
    /// First operand.
    pub in1: U<32>,
    /// Second operand.
    pub in2: U<32>,
}

/// TODO: Documentation
#[derive(Debug, Default, Clone, Copy)]
enum Status {
    #[default]
    Ready,
    NegInputs,
    Mul,
    Div,
    NegOutput,
    DoneMul,
    DoneDiv,
}

/// Multiplier state.
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct MulS<P: Copy> {
    status: Status,
    req: (P, MulReq),
    count: U<33>,
    neg_out: bool,
    is_hi: bool,
    res_hi: bool,
    divisor: U<33>,
    remainder: U<{ 2 * 32 + 2 }>,
}

impl<P: Copy> Default for MulS<P> {
    fn default() -> Self {
        Self {
            status: Status::default(),
            req: unsafe { x() },
            count: 0.into_u(),
            neg_out: false,
            is_hi: false,
            res_hi: false,
            divisor: 0.into_u(),
            remainder: 0.into_u(),
        }
    }
}

/// Multiplier.
pub fn muldiv<P: Copy, R: Copy>(
    i: I<VrH<(P, MulReq), R>, { Dep::Helpful }>,
) -> I<VrH<(P, U<32>), (R, bool)>, { Dep::Helpful }> {
    unsafe {
        i.fsm::<MulS<P>, { Dep::Helpful }, VrH<(P, U<32>), (R, bool)>>(MulS::default(), |ip, er, s| {
            let kill = er.inner.1;

            if kill {
                // If kill happens, return some garbage value.
                let ep = Some((s.req.0, 0.into_u()));
                let ir = Ready::new(matches!(s.status, Status::Ready), er.inner.0);
                let s_next = MulS::default();

                return (ep, ir, s_next);
            }

            let subtractor = s.remainder.clip_const::<33>(32) - s.divisor;
            let result = if s.res_hi { s.remainder.clip_const::<32>(32 + 1) } else { s.remainder.clip_const::<32>(0) };
            let negated_remainder = 0.into_u() - result;

            let ep = if matches!(s.status, Status::DoneMul | Status::DoneDiv) { Some((s.req.0, result)) } else { None };
            let ir = Ready::new(matches!(s.status, Status::Ready), er.inner.0);

            let s_next = match s.status {
                Status::Ready => {
                    if let Some((p, req)) = ip {
                        let (cmd_mul, cmd_hi, lhs_signed, rhs_signed) = req.op.decode();
                        let lhs_sign = lhs_signed && req.in1[32 - 1];
                        let rhs_sign = rhs_signed && req.in2[32 - 1];

                        MulS {
                            status: if cmd_mul {
                                Status::Mul
                            } else if lhs_sign || rhs_sign {
                                Status::NegInputs
                            } else {
                                Status::Div
                            },
                            is_hi: cmd_hi,
                            res_hi: false,
                            count: 0.into_u(),
                            neg_out: if cmd_hi { lhs_sign } else { lhs_sign ^ rhs_sign },
                            divisor: req.in2.append(rhs_sign.repeat::<1>()),
                            remainder: req.in1.resize(),
                            req: (p, req),
                        }
                    } else {
                        s
                    }
                }
                Status::NegInputs => MulS {
                    remainder: if s.remainder[32 - 1] { negated_remainder.resize() } else { s.remainder },
                    divisor: if s.divisor[32 - 1] { subtractor } else { s.divisor },
                    status: Status::Div,
                    ..s
                },
                Status::NegOutput => {
                    MulS { remainder: negated_remainder.resize(), status: Status::DoneDiv, res_hi: false, ..s }
                }
                Status::Mul => {
                    let mplier_sign = s.remainder[32];
                    let mplier = s.remainder.clip_const::<32>(0);
                    let accum = s.remainder.clip_const::<33>(32 + 1);
                    let mpcand = s.divisor;

                    let prod = {
                        let mpcand = mpcand.sext::<34>();
                        let accum = accum.sext::<34>();

                        if !mplier[0] {
                            accum
                        } else if mplier_sign {
                            accum - mpcand
                        } else {
                            (accum + mpcand).resize()
                        }
                    };

                    let next_mul_reg = mplier.clip_const::<31>(1).append(prod);
                    let next_mplier_sign = s.count == 30.into_u() && s.neg_out;

                    MulS {
                        count: (s.count + 1.into_u()).resize(),
                        status: if s.count == (32 - 1).into_u() { Status::DoneMul } else { s.status },
                        res_hi: if s.count == (32 - 1).into_u() { s.is_hi } else { s.res_hi },
                        remainder: next_mul_reg
                            .clip_const::<32>(0)
                            .append(next_mplier_sign.repeat::<1>())
                            .append(next_mul_reg.clip_const::<33>(32)),
                        ..s
                    }
                }
                Status::Div => {
                    let remainder = {
                        let difference = subtractor;
                        let less = difference[32];
                        (!less).repeat::<1>().append(s.remainder.clip_const::<32>(0)).append(if less {
                            s.remainder.clip_const::<32>(32)
                        } else {
                            difference.clip_const::<32>(0)
                        })
                    };

                    let divby0 = s.count == 0.into_u() && !subtractor[32];

                    MulS {
                        remainder: remainder.resize(),
                        status: if s.count == 32.into_u() {
                            if s.neg_out {
                                Status::NegOutput
                            } else {
                                Status::DoneDiv
                            }
                        } else {
                            s.status
                        },
                        res_hi: if s.count == 32.into_u() { s.is_hi } else { s.res_hi },
                        count: (s.count + 1.into_u()).resize(),
                        neg_out: if divby0 && !s.is_hi { false } else { s.neg_out },
                        ..s
                    }
                }
                Status::DoneMul | Status::DoneDiv => {
                    if er.ready {
                        MulS::default()
                    } else {
                        s
                    }
                }
            };

            (ep, ir, s_next)
        })
    }
}
