//! Execute stage.

use super::*;
use crate::prelude::*;
use crate::std::*;

/// Payload from execute stage to memory stage.
#[derive(Debug, Clone, Copy)]
pub struct ExeEP {
    /// Writeback.
    ///
    /// It contains the writeback address and selector.
    pub wb: HOption<(U<LEN_REG_ADDR>, WbSel)>,

    /// ALU output.
    pub alu_out: u32,

    /// Memory operation.
    pub mem_op: MemOp,

    /// Store data.
    ///
    /// The `SW`, `SH`, and `SB` instructions store 32-bit, 16-bit, and 8-bit values from the low bits of `rs2` to memory.
    pub st_data: HOption<u32>,

    /// Indicates that exception happened or not.
    pub exception: bool,

    /// PC.
    pub pc: u32,

    /// Instruciton (To calculate CPI)
    pub debug_inst: u32,
}

/// Hazard from execute stage to decode stage.
#[derive(Debug, Clone, Copy)]
pub struct ExeR {
    /// Indicates that the fetch stage is killed or not.
    pub if_kill: bool,

    /// Indicates that the decode stage is killed or not.
    pub dec_kill: bool,

    /// Next PC selector.
    pub pc_sel: PcSel,

    /// Writeback.
    ///
    /// It contains the writeback address and data.
    pub wb: HOption<Register>,

    /// Indicates that the instruction access CSR or not.
    pub is_csr: bool,

    /// Indicates that the instruction is load or not.
    pub is_load: bool,

    /// Indicates that the instruction is FENCE.I or not.
    pub is_fencei: bool,
}

/// Execute stage ingress interface hazard.
#[derive(Debug, Clone, Copy)]
pub struct ExeH;

impl Hazard for ExeH {
    type P = (DecEP, u32);
    type R = (MemR, WbR);

    fn ready(_: (DecEP, u32), (memr, _): (MemR, WbR)) -> bool {
        memr.pipeline_kill || !memr.dcache_miss
    }
}

/// Returns PC selector based on the given payload.
fn get_pc_sel(p: DecEP, alu_out: u32) -> PcSel {
    let target = p.jmp_target.0 + p.jmp_target.1;

    let alu_true = alu_out != 0;

    match p.br_type {
        BranchType::N => PcSel::Plus4,
        BranchType::J => {
            // From J-instruction
            PcSel::Jmp(target)
        }
        BranchType::Eq | BranchType::Ge | BranchType::Geu => {
            // From Br-instruction
            if !alu_true {
                PcSel::Jmp(target)
            } else {
                PcSel::Plus4
            }
        }
        BranchType::Ne | BranchType::Lt | BranchType::Ltu => {
            // From Br-instruction
            if alu_true {
                PcSel::Jmp(target)
            } else {
                PcSel::Plus4
            }
        }
    }
}

/// Generates resolver from execute stage to decode stage.
fn gen_resolver(er: (HOption<(DecEP, u32)>, MemR, WbR)) -> (ExeR, MemR, WbR) {
    let (p, memr, wbr) = er;

    let (p, alu_out) = p.unzip();
    let is_csr = p.is_some_and(|p| match p.mem_op {
        MemOp::Csr(csr_info) => !matches!(csr_info.cmd, csr::CsrCommand::I),
        _ => false,
    });

    let is_fencei = p.is_some_and(|p| p.is_fencei);

    if memr.pipeline_kill {
        let exer = ExeR {
            if_kill: true,
            dec_kill: true,
            pc_sel: PcSel::Exception(memr.csr_evec),
            wb: None,
            is_csr,
            is_load: false,
            is_fencei,
        };

        return (exer, memr, wbr);
    }

    let Some(p) = p else {
        let exer =
            ExeR { if_kill: false, dec_kill: false, pc_sel: PcSel::Plus4, wb: None, is_csr, is_load: false, is_fencei };

        return (exer, memr, wbr);
    };

    let Some(alu_out) = alu_out else {
        let exer =
            ExeR { if_kill: false, dec_kill: false, pc_sel: PcSel::Plus4, wb: None, is_csr, is_load: false, is_fencei };

        return (exer, memr, wbr);
    };

    let pc_sel = get_pc_sel(p, alu_out);

    let stalled = memr.dcache_miss;
    let exer_wb = if stalled { None } else { p.wb.map(|(addr, _)| Register::new(addr, alu_out)) };

    let exer = ExeR {
        if_kill: !matches!(pc_sel, PcSel::Plus4) || p.is_fencei,
        dec_kill: !matches!(pc_sel, PcSel::Plus4),
        pc_sel,
        wb: exer_wb,
        is_csr,
        is_load: matches!(p.mem_op, MemOp::Dmem { fcn: MemOpFcn::Load, .. }),
        is_fencei,
    };

    (exer, memr, wbr)
}

/// Generates payload from execute stage to memory stage.
fn gen_payload(ip: DecEP, alu_out: u32, (memr, _): (MemR, WbR)) -> HOption<ExeEP> {
    if memr.pipeline_kill {
        None
    } else {
        Some(ExeEP {
            alu_out,
            wb: ip.wb,
            mem_op: ip.mem_op,
            st_data: ip.rs2.map(|rs2| rs2.data),
            exception: ip.is_illegal,
            pc: ip.pc,
            debug_inst: ip.debug_inst,
        })
    }
}

/// Execute stage.
pub fn exe(i: I<VrH<DecEP, (ExeR, MemR, WbR)>, { Dep::Demanding }>) -> I<VrH<ExeEP, (MemR, WbR)>, { Dep::Demanding }> {
    i.map_resolver_inner::<(HOption<(DecEP, u32)>, MemR, WbR)>(gen_resolver)
        .reg_fwd(true)
        .map(|p| (p, exe_alu(p.alu_input.op1_data, p.alu_input.op2_data, p.alu_input.op)))
        .map_resolver_block_with_p::<AndH<ExeH>>(|ip, er| {
            let (memr, wbr) = er.inner;
            (ip, memr, wbr)
        })
        .filter_map_drop_with_r(|(ip, alu_out), er| gen_payload(ip, alu_out, er.inner))
}
