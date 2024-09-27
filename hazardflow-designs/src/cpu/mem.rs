//! Memory stage.

use super::*;

/// Memory access information.
#[derive(Debug, Clone, Copy)]
pub struct MemInfo {
    /// Function (load or store).
    pub fcn: MemOpFcn,

    /// Operand type.
    pub typ: MemOpTyp,

    /// Store data.
    ///
    /// Used for S-type instructions (`sw`, `sh`, `sb`).
    pub data: u32,
}

/// Payload from memory stage to writeback stage.
#[derive(Debug, Clone, Copy)]
pub struct MemEP {
    /// Writeback information.
    ///
    /// It contains the writeback address and data.
    pub wb_info: HOption<Register>,

    /// PC (for debugging purpose).
    pub debug_pc: u32,

    /// Instruction (for debugging purpose).
    pub debug_inst: u32,
}

/// Hazard from memory stage to execute stage.
#[derive(Debug, Clone, Copy)]
pub struct MemR {
    /// Bypassed data from MEM.
    pub bypass_from_mem: HOption<Register>,

    /// Bypassed data from WB.
    pub bypass_from_wb: HOption<Register>,

    /// Indicates that the pipeline should be redirected.
    pub redirect: HOption<u32>,

    /// Register file.
    pub rf: Regfile,
}

impl MemR {
    /// Creates a new memory resolver.
    pub fn new(wbr: WbR, bypass_from_mem: HOption<Register>, redirect: HOption<u32>) -> Self {
        Self { bypass_from_mem, bypass_from_wb: wbr.bypass_from_wb, redirect, rf: wbr.rf }
    }
}

fn get_wb(p: ExeEP, dmem_resp: HOption<MemRespWithAddr>, csr_resp: HOption<CsrResp>) -> HOption<Register> {
    p.wb_info.map(|(addr, wb_sel)| {
        let data = match wb_sel {
            WbSel::Alu => p.alu_out,
            WbSel::Mem => dmem_resp.unwrap().data,
            WbSel::Csr => csr_resp.unwrap().rdata,
        };

        Register::new(addr, data)
    })
}

fn gen_resolver(er: (HOption<(MemRespWithAddr, ExeEP)>, HOption<(CsrResp, ExeEP)>, (HOption<ExeEP>, WbR))) -> MemR {
    // Extracts resolver from each branch.
    let (er_dmem, er_csr, (er_none, wbr)) = er;

    let dmem_resp = er_dmem.map(|(r, _)| r);
    let csr_resp = er_csr.map(|(r, _)| r);
    let exep = er_dmem.map(|(_, r)| r).or(er_csr.map(|(_, r)| r)).or(er_none);

    let exception = exep.is_some_and(|p| p.is_illegal);

    let bypass = exep.and_then(|p| get_wb(p, dmem_resp, csr_resp));
    let redirect = csr_resp.and_then(|r| if r.eret || exception { Some(r.evec) } else { None });

    MemR::new(wbr, bypass, redirect)
}

/// Memory stage.
pub fn mem(
    i: I<VrH<ExeEP, MemR>, { Dep::Demanding }>,
    dmem: impl FnOnce(Vr<MemReq>) -> Vr<MemRespWithAddr>,
) -> I<VrH<MemEP, WbR>, { Dep::Demanding }> {
    let exep = i
        .map_resolver_inner::<(HOption<(MemRespWithAddr, ExeEP)>, HOption<(CsrResp, ExeEP)>, (HOption<ExeEP>, WbR))>(
            gen_resolver,
        )
        .reg_fwd(true);

    let (dmem_req, csr_req, exep) = exep
        .map(|p| {
            let sel = if p.mem_info.is_some() {
                0.into_u()
            } else if p.csr_info.is_some() || p.is_illegal {
                1.into_u()
            } else {
                2.into_u()
            };

            (p, BoundedU::new(sel))
        })
        .branch();

    let dmem_resp = dmem_req
        .map(|ip| {
            let Some(MemInfo { fcn, typ, data }) = ip.mem_info else { unsafe { x() } };

            let mem_req = match fcn {
                MemOpFcn::Load => MemReq::load(ip.alu_out, typ),
                MemOpFcn::Store => MemReq::store(ip.alu_out, data, typ),
            };

            (mem_req, ip)
        })
        .comb(attach_resolver(attach_payload(dmem)))
        .map_resolver_inner_with_p::<WbR>(|ip, _| ip)
        .map(|(dmem_resp, ip)| MemEP {
            wb_info: ip.wb_info.map(|(addr, _)| Register::new(addr, dmem_resp.data)),
            debug_inst: ip.debug_inst,
            debug_pc: ip.pc,
        });

    let csr_resp = csr_req
        .map(|ip| {
            let Some(CsrInfo { cmd, addr }) = ip.csr_info else { unsafe { x() } };

            let csr_req = CsrReq { cmd, wdata: ip.alu_out, decode: addr, exception: ip.is_illegal, pc: ip.pc };

            (csr_req, ip)
        })
        .comb(csr_wrap)
        .map_resolver_inner_with_p::<WbR>(|ip, _| ip)
        .map(|(csr_resp, ip)| MemEP {
            wb_info: ip.wb_info.map(|(addr, _)| Register::new(addr, csr_resp.rdata)),
            debug_inst: ip.debug_inst,
            debug_pc: ip.pc,
        });

    let exep = exep.map_resolver_inner_with_p::<WbR>(|ip, er| (ip, er)).map(|ip| MemEP {
        wb_info: ip.wb_info.map(|(addr, _)| Register::new(addr, ip.alu_out)),
        debug_inst: ip.debug_inst,
        debug_pc: ip.pc,
    });

    [dmem_resp, csr_resp, exep].merge()
}
