//! CSR.
//!
//! # References
//!
//! - Constants: <https://github.com/chipsalliance/rocket-chip/blob/master/src/main/scala/rocket/CSR.scala>

use super::exe::ExeEP;
use super::riscv_isa::LEN_CSR_ADDR;
use super::wb::WbR;
use crate::std::hazard::*;
use crate::std::*;

/// Contains information that is needed to interact with CSR.
#[derive(Debug, Clone, Copy)]
pub struct CsrInfo {
    /// CSR address.
    pub addr: U<LEN_CSR_ADDR>,

    /// CSR command.
    pub cmd: CsrCmd,
}

/// CSR Commands.
///
/// NOTE: This type should be represented as 3-bits.
/// - <https://github.com/chipsalliance/rocket-chip/blob/master/src/main/scala/rocket/CSR.scala#L168-L178>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsrCmd {
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

/// TODO: Documentation, Add remaining fields
#[derive(Debug, Clone, Copy)]
pub struct CsrReq {
    /// TODO: Documentation
    pub cmd: CsrCmd,

    /// TODO: Documentation
    pub wdata: u32,

    /// TODO: Documentation
    pub decode: U<LEN_CSR_ADDR>,

    /// TODO: Documentation
    pub exception: bool,

    /// TODO: Documentation
    pub pc: u32,
}

/// TODO: Documentation
#[derive(Debug, Clone, Copy)]
pub struct CsrResp {
    /// TODO: Documentation
    pub rdata: u32,

    /// TODO: Documentation
    pub eret: bool,

    /// TODO: Documentation
    pub evec: u32,
}

/// MStatus.
///
/// Omitted unused fields.
#[derive(Debug, Default, Clone, Copy)]
struct MStatus {
    mpie: bool,
    mie: bool,
}

impl MStatus {
    fn into_u(self) -> U<35> {
        0.into_u::<3>()
            .append(self.mie.repeat::<1>())
            .append(0.into_u::<3>())
            .append(self.mpie.repeat::<1>())
            .append(0x18.into_u::<5>())
            .append(0x180000.into_u::<22>())
    }
}

/// MIP.
///
/// Omitted unusd fields.
#[derive(Debug, Default, Clone, Copy)]
struct Mip {
    mtip: bool,
    msip: bool,
}

impl Mip {
    fn into_u(self) -> U<16> {
        0.into_u::<3>()
            .append(self.msip.repeat::<1>())
            .append(0.into_u::<3>())
            .append(self.mtip.repeat::<1>())
            .append(0.into_u::<8>())
    }
}

/// CSR registers.
#[derive(Debug, Clone, Copy)]
enum CsrReg {
    Mstatus,
    Mtvec,
    Mip,
    Mie,
    Mscratch,
    Mepc,
    Mtval,
    Mcause,
    Medeleg,
    Unsupported,
}

impl From<U<LEN_CSR_ADDR>> for CsrReg {
    fn from(value: U<LEN_CSR_ADDR>) -> Self {
        if value == 0x300.into_u() {
            CsrReg::Mstatus
        } else if value == 0x302.into_u() {
            CsrReg::Medeleg
        } else if value == 0x304.into_u() {
            CsrReg::Mie
        } else if value == 0x305.into_u() {
            CsrReg::Mtvec
        } else if value == 0x340.into_u() {
            CsrReg::Mscratch
        } else if value == 0x341.into_u() {
            CsrReg::Mepc
        } else if value == 0x342.into_u() {
            CsrReg::Mcause
        } else if value == 0x343.into_u() {
            CsrReg::Mtval
        } else if value == 0x344.into_u() {
            CsrReg::Mip
        } else {
            CsrReg::Unsupported
        }
    }
}

/// CSR state.
#[derive(Debug, Clone, Copy)]
struct CsrS {
    mstatus: MStatus,
    mepc: u32,
    mcause: u32,
    mtval: u32,
    mscratch: u32,
    medeleg: u32,
    mip: Mip,
    mie: Mip,
}

impl Default for CsrS {
    fn default() -> Self {
        CsrS {
            mstatus: MStatus::default(),
            mepc: 0,
            mcause: 0,
            mtval: 0,
            mscratch: 0,
            medeleg: 0,
            mip: Mip { mtip: true, msip: false },
            mie: Mip::default(),
        }
    }
}

/// CSR file.
pub fn csr(i: Valid<CsrReq>) -> Valid<CsrResp> {
    i.fsm_map::<CsrResp, CsrS>(CsrS::default(), |ip, s| {
        let system_insn = matches!(ip.cmd, CsrCmd::I);
        let cpu_ren = !system_insn;

        let decoded_addr = CsrReg::from(ip.decode);

        let rdata = match decoded_addr {
            CsrReg::Mstatus => u32::from(s.mstatus.into_u()),
            CsrReg::Mtvec => 0x100,
            CsrReg::Mip => u32::from(s.mip.into_u()),
            CsrReg::Mie => u32::from(s.mie.into_u()),
            CsrReg::Mscratch => s.mscratch,
            CsrReg::Mepc => s.mepc,
            CsrReg::Mtval => s.mtval,
            CsrReg::Mcause => s.mcause,
            CsrReg::Medeleg => s.medeleg,
            CsrReg::Unsupported => 0,
        };

        let read_only = ip.decode.clip_const::<2>(10) == 0b11.into_u();
        let cpu_wen = cpu_ren && !matches!(ip.cmd, CsrCmd::R);
        let wen = cpu_wen && !read_only;
        let wdata = (if matches!(ip.cmd, CsrCmd::S | CsrCmd::C) { rdata } else { 0 } | ip.wdata)
            & !if matches!(ip.cmd, CsrCmd::C) { ip.wdata } else { 0 };

        let opcode = 0.into_u::<7>().set(ip.decode.clip_const::<3>(0), true);
        let insn_call = system_insn && opcode[0];
        let insn_break = system_insn && opcode[1];
        let insn_ret = system_insn && opcode[2];

        let eret = insn_call || insn_break || insn_ret;

        let ep = CsrResp { rdata, eret, evec: if insn_ret && !ip.decode[10] { s.mepc } else { 0x80000004 } };

        let s_next = CsrS {
            mstatus: if wen && matches!(decoded_addr, CsrReg::Mstatus) {
                MStatus { mie: U::<32>::from(wdata)[3], mpie: U::<32>::from(wdata)[7] }
            } else if insn_ret && !ip.decode[10] {
                MStatus { mie: s.mstatus.mpie, mpie: true }
            } else {
                s.mstatus
            },
            mepc: if wen && matches!(decoded_addr, CsrReg::Mepc) {
                (wdata >> 2) << 2
            } else if ip.exception || insn_call || insn_break {
                ip.pc
            } else {
                s.mepc
            },
            mcause: if wen && matches!(decoded_addr, CsrReg::Mcause) {
                wdata & 0x8000001F
            } else if ip.exception {
                0x2
            } else if insn_call {
                0xb
            } else if insn_break {
                0x3
            } else {
                s.mcause
            },
            mtval: if wen && matches!(decoded_addr, CsrReg::Mtval) { wdata } else { s.mtval },
            mscratch: if wen && matches!(decoded_addr, CsrReg::Mscratch) { wdata } else { s.mscratch },
            medeleg: if wen && matches!(decoded_addr, CsrReg::Medeleg) { wdata } else { s.medeleg },
            mip: if wen && matches!(decoded_addr, CsrReg::Mip) {
                Mip { mtip: s.mip.mtip, msip: U::<32>::from(wdata)[3] }
            } else {
                s.mip
            },
            mie: if wen && matches!(decoded_addr, CsrReg::Mie) {
                Mip { msip: U::<32>::from(wdata)[3], mtip: U::<32>::from(wdata)[7] }
            } else {
                s.mie
            },
        };

        (ep, s_next)
    })
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
                (ep, ((), er), s)
            },
        )
    }
}
