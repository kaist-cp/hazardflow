//! CSR.
//!
//! # References
//!
//! - Constants: <https://github.com/chipsalliance/rocket-chip/blob/master/src/main/scala/rocket/CSR.scala>

use hazardflow_macro::magic;

use super::exe::ExeEP;
use super::riscv_isa::LEN_CSR_ADDR;
use super::wb::WbR;
use crate::std::hazard::*;
use crate::std::*;

/// Contains information that is needed to interact with CSR.
#[derive(Debug, Clone, Copy)]
pub struct CsrInfo {
    /// CSR address
    /// `csr.io.rw.addr`
    pub addr: U<LEN_CSR_ADDR>,

    /// CSR command.
    /// `csr.io.rw.cmd`
    pub cmd: CsrCommand,
}

/// CSR Commands.
///
/// NOTE: This type should be represented as 3-bits.
/// - <https://github.com/chipsalliance/rocket-chip/blob/master/src/main/scala/rocket/CSR.scala#L168-L178>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsrCommand {
    /// TODO: Documentation
    R = 5,

    /// TODO: Documentation
    S = 2,

    /// TODO: Documentation
    C = 3,

    /// TODO: Documentation
    W = 1,

    /// TODO: Documentation
    I = 4,
}

/// TODO: Documentation
#[derive(Debug, Clone, Copy)]
pub struct CsrDecodeI {
    /// TODO: Documentation
    pub csr: U<LEN_CSR_ADDR>,
}

/// TODO: Documentation
#[derive(Debug, Clone, Copy)]
pub struct CsrRwI {
    /// TODO: Documentation
    pub cmd: CsrCommand,

    /// TODO: Documentation
    pub wdata: u32,
}

/// TODO: Documentation
#[derive(Debug, Clone, Copy)]
pub struct CsrRwE {
    /// TODO: Documentation
    pub rdata: u32,
}

/// TODO: Documentation, Add remaining fields
#[derive(Debug, Clone, Copy)]
pub struct CsrReq {
    /// TODO: Documentation
    pub rw: CsrRwI,

    /// TODO: Documentation
    pub decode: CsrDecodeI,

    /// TODO: Documentation
    pub exception: bool,

    /// TODO: Documentation
    pub pc: u32,
}

/// TODO: Documentation
#[derive(Debug, Clone, Copy)]
pub struct CsrResp {
    /// TODO: Documentation
    pub rw: CsrRwE,

    /// TODO: Documentation
    pub eret: bool,

    /// TODO: Documentation
    pub evec: u32,

    /// TODO: Documentation
    pub time: u32,
}

/// CSR file.
#[magic(ffi::CSRFileWrapper())]
pub fn csr(_csr_req: Valid<CsrReq>) -> I<ValidH<CsrResp, bool>, { Dep::Helpful }> {
    unreachable!("csr_wrapper.v")
}

/// TODO: Documentation
pub fn csr_wrap<P: Copy>(
    i: I<VrH<(CsrReq, P), (HOption<(CsrResp, ExeEP)>, WbR)>, { Dep::Helpful }>,
) -> I<VrH<(CsrResp, P), (HOption<(CsrResp, ExeEP)>, WbR)>, { Dep::Helpful }> {
    let (i1, i2) = unsafe {
        Interface::fsm::<(Valid<CsrReq>, I<VrH<P, (HOption<(CsrResp, ExeEP)>, WbR)>, { Dep::Helpful }>), ()>(
            i,
            (),
            |ip, er, s| {
                let ep1 = ip.map(|p| p.0);
                let ep2 = ip.map(|p| p.1);
                let ir = er.1;
                ((ep1, ep2), ir, s)
            },
        )
    };

    let e1 = i1.comb(csr);

    unsafe {
        (e1, i2).fsm::<I<VrH<(CsrResp, P), (HOption<(CsrResp, ExeEP)>, WbR)>, { Dep::Helpful }>, ()>(
            (),
            |(ip1, ip2), er, s| {
                let ep = ip1.zip(ip2);
                let ir1 = er.inner.1.retire;
                let ir2 = er;
                (ep, (ir1, ir2), s)
            },
        )
    }
}
