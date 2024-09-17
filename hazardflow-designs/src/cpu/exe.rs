//! Execute stage.

use super::*;

/// Payload from execute stage to memory stage.
#[derive(Debug, Clone, Copy)]
pub struct ExeEP {
    /// Writeback information.
    ///
    /// It contains the writeback address and selector.
    pub wb_info: HOption<(U<{ clog2(REGS) }>, WbSel)>,

    /// ALU output.
    pub alu_out: u32,

    /// Memory information.
    pub mem_info: HOption<MemInfo>,

    /// CSR information.
    pub csr_info: HOption<CsrInfo>,

    /// Indicates that the instruction is illegal or not.
    pub is_illegal: bool,

    /// PC.
    pub pc: u32,

    /// Instruction (for debugging purpose).
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
    let Some(br_info) = p.br_info else {
        return None;
    };

    let target = br_info.base + br_info.offset;
    let alu_true = alu_out != 0;

    match br_info.typ {
        BrType::Jal | BrType::Jalr => Some(target),
        BrType::Beq | BrType::Bge | BrType::Bgeu => {
            if !alu_true {
                Some(target)
            } else {
                None
            }
        }
        BrType::Bne | BrType::Blt | BrType::Bltu => {
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
        p.wb_info.and_then(|(addr, wb_sel)| if matches!(wb_sel, WbSel::Mem | WbSel::Csr) { Some(addr) } else { None })
    });

    let Some((p, alu_out)) = p else {
        return ExeR::new(memr, None, stall, None);
    };

    let bypass =
        p.wb_info.and_then(
            |(addr, wb_sel)| if matches!(wb_sel, WbSel::Alu) { Some(Register::new(addr, alu_out)) } else { None },
        );

    let redirect = get_redirect(p, alu_out);

    ExeR::new(memr, bypass, stall, redirect)
}

/// Generates payload from execute stage to memory stage.
fn gen_payload(ip: DecEP, alu_out: u32, memr: MemR) -> HOption<ExeEP> {
    if memr.redirect.is_some() {
        None
    } else {
        Some(ExeEP {
            alu_out,
            wb_info: ip.wb_info,
            mem_info: ip.mem_info,
            csr_info: ip.csr_info,
            is_illegal: ip.is_illegal,
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
        .filter_map_drop_with_r_inner(|(ip, alu_out), er| gen_payload(ip, alu_out, er))
}
