//! Memory stage.

use super::csr::*;
use super::*;
use crate::std::hazard::*;
use crate::std::*;

/// Operation at memory stage.
#[derive(Debug, Clone, Copy)]
pub enum MemOp {
    /// Access DMEM.
    Dmem {
        /// Function (load or store)
        fcn: MemOpFcn,

        /// Operand type
        typ: MemOpTyp,
    },

    /// Access CSR.
    Csr(CsrInfo),

    /// Do nothing.
    None,
}

impl MemOp {
    /// Returns DMEM access.
    pub fn dmem(self) -> HOption<(MemOpFcn, MemOpTyp)> {
        match self {
            MemOp::Dmem { fcn, typ } => Some((fcn, typ)),
            _ => None,
        }
    }

    /// Returns CSR access.
    pub fn csr(self) -> HOption<CsrInfo> {
        match self {
            MemOp::Csr(csr_info) => Some(csr_info),
            _ => None,
        }
    }
}

/// Payload from memory stage to writeback stage.
#[derive(Debug, Clone, Copy)]
pub struct MemEP {
    /// Writeback.
    ///
    /// It contains the writeback address and data.
    pub wb: HOption<Register>,

    /// PC (To calculate CPI)
    pub debug_pc: u32,

    /// Instruciton (To calculate CPI)
    pub debug_inst: u32,
}

/// Hazard from memory stage to execute stage.
#[derive(Debug, Clone, Copy)]
pub struct MemR {
    /// Exception target.
    pub csr_evec: u32,

    /// CSR eret.
    pub csr_eret: bool,

    /// Indicates that the D$ miss happened or not.
    pub dcache_miss: bool,

    /// Writeback.
    ///
    /// It contains the writeback address and data.
    pub wb: HOption<Register>,

    /// Indicates that the pipeline killed or not.
    pub pipeline_kill: bool,
}

fn get_wb(p: ExeEP, dmem_resp: HOption<MemRespWithAddr>, csr_resp: HOption<CsrResp>) -> HOption<Register> {
    p.wb.map(|(addr, wb_sel)| {
        let data = match wb_sel {
            WbSel::Alu => p.alu_out,
            WbSel::Mem => dmem_resp.unwrap().data,
            WbSel::Pc4 => p.pc + 4,
            WbSel::Csr => csr_resp.unwrap().rw.rdata,
        };

        Register::new(addr, data)
    })
}

fn gen_resolver(
    er: Ready<(HOption<(MemRespWithAddr, ExeEP)>, (HOption<(CsrResp, ExeEP)>, WbR), HOption<ExeEP>)>,
) -> (MemR, WbR) {
    // Extracts resolver from each branch.
    let (er_dmem, er_csr, er_none) = er.inner;

    let dmem_resp = er_dmem.map(|(r, _)| r);
    let csr_resp = er_csr.0.map(|(r, _)| r);
    let exep = er_dmem.map(|(_, r)| r).or(er_csr.0.map(|(_, r)| r)).or(er_none);
    let wbr = er_csr.1;

    let exception = exep.is_some_and(|p| p.exception);
    let pipeline_kill = csr_resp.is_some_and(|r| r.eret) || exception;

    let memr = MemR {
        csr_evec: csr_resp.map(|r| r.evec).unwrap_or(0),
        csr_eret: csr_resp.is_some_and(|r| r.eret),
        dcache_miss: exep.is_some_and(|p| matches!(p.mem_op, MemOp::Dmem { .. }) && !er.ready),
        wb: exep.and_then(|p| get_wb(p, dmem_resp, csr_resp)),
        pipeline_kill,
    };

    (memr, wbr)
}

/// Memory stage.
pub fn mem(
    i: I<VrH<ExeEP, (MemR, WbR)>, { Dep::Demanding }>,
    dmem: impl FnOnce(Vr<MemReq>) -> Vr<MemRespWithAddr>,
) -> I<VrH<MemEP, WbR>, { Dep::Demanding }> {
    let exep = i
        .reg_fwd(true)
        .map_resolver::<(HOption<(MemRespWithAddr, ExeEP)>, (HOption<(CsrResp, ExeEP)>, WbR), HOption<ExeEP>)>(
            gen_resolver,
        );

    let (dmem_req, csr_req, exep) = exep
        .map(|p| {
            let sel = if p.exception {
                // If exception happens, it should go to the CSR.
                1.into_u()
            } else {
                match p.mem_op {
                    MemOp::Dmem { .. } => 0.into_u(),
                    MemOp::Csr(_) => 1.into_u(),
                    MemOp::None => 2.into_u(),
                }
            };

            (p, BoundedU::new(sel))
        })
        .branch();

    let dmem_resp = dmem_req
        .map(|ip| {
            let MemOp::Dmem { fcn, typ } = ip.mem_op else { unsafe { x() } };

            let mem_req = match fcn {
                MemOpFcn::Load => MemReq::load(ip.alu_out, typ),
                MemOpFcn::Store => MemReq::store(ip.alu_out, ip.st_data.unwrap(), typ),
            };

            (mem_req, ip)
        })
        .comb(attach_resolver(attach_payload(dmem)))
        .map_resolver_with_p::<WbR>(|ip, _| ip)
        .map(|ip| (Some(ip), None, None));

    let csr_resp = csr_req
        .map(|ip| {
            let MemOp::Csr(csr) = ip.mem_op else { unsafe { x() } };

            let csr_req = CsrReq {
                rw: CsrRwI { cmd: csr.cmd, wdata: ip.alu_out },
                decode: CsrDecodeI { csr: csr.addr },
                exception: ip.exception,
                pc: ip.pc,
            };

            (csr_req, ip)
        })
        .comb(csr_wrap)
        .map_resolver_with_p::<WbR>(|ip, er| (ip, er.inner))
        .map(|ip| (None, Some(ip), None));

    let exep = exep.map_resolver_with_p::<WbR>(|ip, _| ip).map(|ip| (None, None, Some(ip)));

    [dmem_resp, csr_resp, exep].merge().map(|(mem_resp, csr_resp, exep)| {
        let exep = mem_resp.map(|(_, p)| p).or(csr_resp.map(|(_, p)| p)).or(exep).unwrap();
        let mem_resp = mem_resp.map(|(p, _)| p);
        let csr_resp = csr_resp.map(|(p, _)| p);

        MemEP { wb: get_wb(exep, mem_resp, csr_resp), debug_inst: exep.debug_inst, debug_pc: exep.pc }
    })
}
