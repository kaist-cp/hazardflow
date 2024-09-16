//! Decode stage.

use super::*;

/// Payload from decode stage to execute stage.
#[derive(Debug, Clone, Copy)]
pub struct DecEP {
    /// Writeback information.
    ///
    /// It contains the writeback address and selector.
    pub wb_info: HOption<(U<{ clog2(REGS) }>, WbSel)>,

    /// Branch information.
    pub br_info: HOption<BrInfo>,

    /// ALU input.
    pub alu_input: AluInput,

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

    let bypass = |addr: U<{ clog2(REGS) }>| -> u32 {
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

    let br_info = inst.br_info(rs1, ip.imem_resp.addr);

    Some(DecEP {
        wb_info: inst.rd_addr.zip(inst.wb_sel),
        br_info,
        alu_input,
        mem_info: inst.mem_info.map(|(fcn, typ)| MemInfo {
            fcn,
            typ,
            data: rs2.map(|r| r.data).unwrap_or(unsafe { x() }),
        }),
        csr_info: inst.csr_info,
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
