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

    /// Store data.
    ///
    /// The `SW`, `SH`, and `SB` instructions store 32-bit, 16-bit, and 8-bit values from the low bits of `rs2` to memory.
    pub st_data: HOption<u32>,

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

    /// Indicates that the instruction is illegal/unsupported or not.
    pub is_illegal: bool,

    /// PC.
    pub pc: u32,

    /// Instruciton (To calculate CPI)
    pub debug_inst: u32,
}

/// Hazard from decode stage to fetch stage.
#[derive(Debug, Clone, Copy)]
pub struct DecR {
    /// Indicates that the pipeline should be redirected.
    pub redirect: HOption<u32>,
}

impl DecR {
    /// Creates a new decode resolver.
    pub fn new(exer: ExeR) -> Self {
        Self { redirect: exer.redirect }
    }
}

/// Decode stage ingress interface hazard.
#[derive(Debug, Clone, Copy)]
pub struct DecH;

impl Hazard for DecH {
    type P = (FetEP, Instruction);
    type R = ExeR;

    fn ready((_, inst): (FetEP, Instruction), exer: ExeR) -> bool {
        let rs1_addr = inst.rs1_addr;
        let rs2_addr = inst.rs2_addr;

        // Stalled from load-use or CSR.
        let stall = exer.stall.is_some_and(|addr| rs1_addr == Some(addr) || rs2_addr == Some(addr));

        exer.redirect.is_some() || !stall
    }
}

/// Generates payload from decode stage to execute stage.
fn gen_payload(ip: FetEP, inst: Instruction, er: ExeR) -> HOption<DecEP> {
    if er.redirect.is_some() {
        return None;
    }

    let rs1_addr = inst.rs1_addr;
    let rs2_addr = inst.rs2_addr;

    let bypass = |addr: U<LEN_REG_ADDR>| -> u32 {
        // Check that the data can be bypassed.
        let from_exe = er.bypass_from_exe.filter(|r| addr == r.addr).map(|r| r.data);
        let from_mem = er.bypass_from_mem.filter(|r| addr == r.addr).map(|r| r.data);
        let from_wb = er.bypass_from_wb.filter(|r| addr == r.addr).map(|r| r.data);

        // Bypassing priority: EXE > MEM > WB
        from_exe.or(from_mem).or(from_wb).unwrap_or(er.rf[addr])
    };

    let rs1 = rs1_addr.map(|addr| Register::new(addr, bypass(addr)));
    let rs2 = rs2_addr.map(|addr| Register::new(addr, bypass(addr)));

    // ALU input.
    let alu_input = {
        // Comment about JALR and JAL instruction:
        // Both instructions store pc + 4 value to rd.
        // op1 will be pc value, and op2 will be 4.

        // First operand of ALU.
        let op1_data = inst.op1_data(rs1, ip.imem_resp.addr);

        // Second operand of ALU.
        let op2_data = inst.op2_data(rs2);

        AluInput { op: inst.alu_op, op1_data, op2_data }
    };

    let jmp_target = inst.jmp_target(rs1, ip.imem_resp.addr);

    Some(DecEP {
        wb: inst.rd_addr.zip(inst.wb_sel),
        st_data: rs2.map(|rs2| rs2.data),
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
        is_illegal: inst.is_illegal,
        pc: ip.imem_resp.addr,
        debug_inst: ip.imem_resp.data,
    })
}

/// Decode stage.
pub fn decode(i: I<VrH<FetEP, DecR>, { Dep::Demanding }>) -> I<VrH<DecEP, ExeR>, { Dep::Demanding }> {
    i.map_resolver_inner::<ExeR>(DecR::new)
        .reg_fwd(true)
        .map(|p| (p, Instruction::from(p.imem_resp.data)))
        .map_resolver_block::<AndH<DecH>>(|er| er.inner)
        .filter_map_drop_with_r(|(p, inst), er| gen_payload(p, inst, er.inner))
}
