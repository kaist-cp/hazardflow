//! Decode stage.

use super::*;
use crate::prelude::*;
use crate::std::*;

/// Payload from decode stage to execute stage.
#[derive(Debug, Clone, Copy)]
pub struct DecEP {
    /// Writeback.
    ///
    /// It contains the writeback address and selector.
    pub wb: HOption<(U<LEN_REG_ADDR>, WbSel)>,

    /// RS1.
    pub rs1: HOption<Register>,

    /// RS2.
    pub rs2: HOption<Register>,

    /// Branch type.
    pub br_type: BranchType,

    /// Jump target.
    ///
    /// It contains the base address and offset.
    pub jmp_target: (u32, u32),

    /// ALU input.
    pub alu_input: AluInput,

    /// Memory operation.
    pub mem_op: MemOp,

    /// Indicates that the instruction is `FenceI` or not.
    pub is_fencei: bool,

    /// Indicates that the instruction is illegal/unsupported or not.
    pub is_illegal: bool,

    /// PC.
    pub pc: u32,

    /// Instruciton (To calculate CPI)
    pub debug_inst: u32,
}

/// Decode stage ingress interface hazard.
#[derive(Debug, Clone, Copy)]
pub struct DecH;

impl Hazard for DecH {
    type P = (MemRespWithAddr, Instruction);
    type R = (ExeR, MemR, WbR);

    fn ready((_, inst): (MemRespWithAddr, Instruction), (exer, memr, _): (ExeR, MemR, WbR)) -> bool {
        let rs1_addr = inst.rs1_addr;
        let rs2_addr = inst.rs2_addr;

        // Load-use stall.
        let load_use_stall =
            exer.is_load && exer.wb.is_some_and(|wb| rs1_addr == Some(wb.addr) || rs2_addr == Some(wb.addr));

        // D$ miss stall.
        let dcache_stall = memr.dcache_miss;

        memr.pipeline_kill || (!load_use_stall && !exer.is_csr && !dcache_stall)
    }
}

/// Generates resolver from decode stage to fetch stage.
fn gen_resolver(er: (HOption<(MemRespWithAddr, Instruction)>, ExeR, MemR, WbR)) -> (bool, PcSel) {
    let (p, exer, memr, _) = er;

    let inst = p.map(|(_, inst)| inst);
    let is_fencei = inst.is_some_and(|inst| inst.is_fencei);
    let if_kill = exer.if_kill || is_fencei || memr.pipeline_kill;

    let pc_sel = if matches!(exer.pc_sel, PcSel::Jmp { .. } | PcSel::Exception(_)) {
        exer.pc_sel
    } else if is_fencei || exer.is_fencei {
        PcSel::Curr
    } else {
        exer.pc_sel
    };

    (if_kill, pc_sel)
}

/// Generates payload from decode stage to execute stage.
fn gen_payload(ip: MemRespWithAddr, inst: Instruction, er: (ExeR, MemR, WbR)) -> HOption<DecEP> {
    let (exer, memr, wbr) = er;

    if exer.dec_kill || memr.pipeline_kill {
        return None;
    }

    let rs1_addr = inst.rs1_addr;
    let rs2_addr = inst.rs2_addr;

    let bypass = |addr: U<LEN_REG_ADDR>| -> u32 {
        // Check that the data can be bypassed.
        let from_exe = exer.wb.filter(|r| addr == r.addr).map(|r| r.data);
        let from_mem = memr.wb.filter(|r| addr == r.addr).map(|r| r.data);
        let from_wb = wbr.wb.filter(|r| addr == r.addr).map(|r| r.data);

        // Bypassing priority: EXE > MEM > WB
        from_exe.or(from_mem).or(from_wb).unwrap_or(wbr.rf[addr])
    };

    let rs1 = rs1_addr.map(|addr| Register::new(addr, bypass(addr)));
    let rs2 = rs2_addr.map(|addr| Register::new(addr, bypass(addr)));

    // ALU input.
    let alu_input = {
        // Comment about JALR and JAL instruction:
        // Both instructions store pc + 4 value to rd.
        // op1 will be pc value, and op2 will be 4.

        // First operand of ALU.
        let op1_data = inst.op1_data(rs1, ip.addr);

        // Second operand of ALU.
        let op2_data = inst.op2_data(rs2);

        AluInput { op: inst.alu_op, op1_data, op2_data }
    };

    let jmp_target = inst.jmp_target(rs1, ip.addr);

    Some(DecEP {
        wb: inst.rd_addr.zip(inst.wb_sel),
        rs1,
        rs2,
        br_type: inst.br_type,
        jmp_target,
        alu_input,
        mem_op: if let Some((fcn, typ)) = inst.mem_info {
            MemOp::Dmem { fcn, typ }
        } else if let Some(csr_info) = inst.csr_info {
            MemOp::Csr(csr_info)
        } else {
            MemOp::None
        },
        is_fencei: inst.is_fencei,
        // If it is returning from trap (`csr_eret`), clear instruction exception.
        is_illegal: inst.is_illegal,
        pc: ip.addr,
        debug_inst: ip.data,
    })
}

/// Decode stage.
pub fn decode(
    i: I<VrH<MemRespWithAddr, (bool, PcSel)>, { Dep::Demanding }>,
) -> I<VrH<DecEP, (ExeR, MemR, WbR)>, { Dep::Demanding }> {
    i.map_resolver_inner::<(HOption<(MemRespWithAddr, Instruction)>, ExeR, MemR, WbR)>(gen_resolver)
        .reg_fwd(true)
        .map(|p| (p, Instruction::from(p.data)))
        .map_resolver_block_with_p::<AndH<DecH>>(|ip, er| {
            let (exer, memr, wbr) = er.inner;
            (ip, exer, memr, wbr)
        })
        .filter_map_drop_with_r(|(p, inst), er| gen_payload(p, inst, er.inner))
}
