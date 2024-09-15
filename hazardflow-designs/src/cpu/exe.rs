//! Execute stage.

use super::*;

/// Payload from execute stage to memory stage.
#[derive(Debug, Clone, Copy)]
pub struct ExeEP {
    /// Writeback.
    ///
    /// It contains the writeback address and selector.
    pub wb: HOption<(U<{ clog2(REGS) }>, WbSel)>,

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
    /// Bypassed data from EXE.
    pub bypass_from_exe: HOption<Register>,

    /// Bypassed data from MEM.
    pub bypass_from_mem: HOption<Register>,

    /// Bypassed data from WB.
    pub bypass_from_wb: HOption<Register>,

    /// Stall.
    ///
    /// It contains the rd address of load or CSR instructions.
    pub stall: HOption<U<{ clog2(REGS) }>>,

    /// Indicates that the pipeline should be redirected.
    pub redirect: HOption<u32>,

    /// Register file.
    pub rf: Regfile,
}

impl ExeR {
    /// Creates a new execute resolver.
    pub fn new(
        memr: MemR,
        bypass: HOption<Register>,
        stall: HOption<U<{ clog2(REGS) }>>,
        redirect: HOption<u32>,
    ) -> Self {
        Self {
            bypass_from_exe: bypass,
            bypass_from_mem: memr.bypass_from_mem,
            bypass_from_wb: memr.bypass_from_wb,
            stall,
            redirect: memr.redirect.or(redirect),
            rf: memr.rf,
        }
    }
}

/// Returns redirected PC based on the given payload.
fn get_redirect(p: DecEP, alu_out: u32) -> HOption<u32> {
    let target = p.jmp_target.0 + p.jmp_target.1;

    let alu_true = alu_out != 0;

    match p.br_type {
        BranchType::N => None,
        BranchType::J => {
            // From J-instruction
            Some(target)
        }
        BranchType::Eq | BranchType::Ge | BranchType::Geu => {
            // From Br-instruction
            if !alu_true {
                Some(target)
            } else {
                None
            }
        }
        BranchType::Ne | BranchType::Lt | BranchType::Ltu => {
            // From Br-instruction
            if alu_true {
                Some(target)
            } else {
                None
            }
        }
    }
}

/// Generates resolver from execute stage to decode stage.
fn gen_resolver(er: (HOption<(DecEP, u32)>, MemR)) -> ExeR {
    let (p, memr) = er;

    let stall = p.and_then(|(p, _)| {
        p.wb.and_then(|(addr, wb_sel)| if matches!(wb_sel, WbSel::Mem | WbSel::Csr) { Some(addr) } else { None })
    });

    let Some((p, alu_out)) = p else {
        return ExeR::new(memr, None, stall, None);
    };

    let redirect = get_redirect(p, alu_out);
    let exer_wb = p.wb.map(|(addr, _)| Register::new(addr, alu_out));

    ExeR::new(memr, exer_wb, stall, redirect)
}

/// Generates payload from execute stage to memory stage.
fn gen_payload(ip: DecEP, alu_out: u32, memr: MemR) -> HOption<ExeEP> {
    if memr.redirect.is_some() {
        None
    } else {
        Some(ExeEP {
            alu_out,
            wb: ip.wb,
            mem_op: ip.mem_op,
            st_data: ip.st_data,
            exception: ip.is_illegal,
            pc: ip.pc,
            debug_inst: ip.debug_inst,
        })
    }
}

/// Execute stage.
pub fn exe(i: I<VrH<DecEP, ExeR>, { Dep::Demanding }>) -> I<VrH<ExeEP, MemR>, { Dep::Demanding }> {
    i.map_resolver_inner::<(HOption<(DecEP, u32)>, MemR)>(gen_resolver)
        .reg_fwd(true)
        .map(|p| match p.alu_input.op {
            AluOp::Base(op) => (p, exe_alu(p.alu_input.op1_data, p.alu_input.op2_data, op)),
            AluOp::Mext(_) => todo!("assignment 3"),
        })
        .map_resolver_block_with_p::<VrH<(DecEP, u32), MemR>>(|ip, er| (ip, er.inner))
        .filter_map_drop_with_r(|(ip, alu_out), er| gen_payload(ip, alu_out, er.inner))
}
