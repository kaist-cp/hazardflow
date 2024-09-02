//! RISC-V Instruction.
//! Currently supports
//! - RV32I Base Instruction Set
//! - RV32 Zifencei Standard Extension
//! - RV32 Zicsr Standard Extension
//! - Partial RISC-V Privileged Instruction Set including:
//!   + Trap-Return Instructions
//!   + Interrupt-Management Instructions
// TODO: Extend to 64-bit architecture

#![allow(missing_docs)]

use super::alu::*;
use super::csr::{CsrCommand, CsrInfo};
use super::mem_interface::*;
use super::wb::Register;
use crate::prelude::*;

// =========== Constants =========== //
/// CSR Address is 12-bit.
pub const LEN_CSR_ADDR: usize = 12;

/// There are 32 integer registers, thus encoded as 5-bit.
pub const LEN_REG_ADDR: usize = 5;

/// Op1 data selector.
#[derive(Debug, Clone, Copy)]
pub enum Op1Sel {
    Rs1,
    Pc,
    Imm,
}

/// Op2 data selector.
#[derive(Debug, Clone, Copy)]
pub enum Op2Sel {
    Four,
    Imm,
    Rs2,
}

/// Jmp target selector.
#[derive(Debug, Clone, Copy)]
pub enum JmpTargetSel {
    BType,
    JType,
    Jalr,
}

/// Instruction2.
#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub is_illegal: bool,
    pub br_type: BranchType,
    pub rs1_addr: HOption<U<LEN_REG_ADDR>>,
    pub rs2_addr: HOption<U<LEN_REG_ADDR>>,
    pub rd_addr: HOption<U<LEN_REG_ADDR>>,
    pub imm: u32,
    pub alu_op: BaseAluOp,
    pub wb_sel: HOption<WbSel>,
    pub is_fencei: bool,
    pub csr_info: HOption<CsrInfo>,
    pub mem_info: HOption<(MemOpFcn, MemOpTyp)>,
    jmp_target_sel: HOption<JmpTargetSel>,
    op1_sel: HOption<Op1Sel>,
    op2_sel: HOption<Op2Sel>,
}

impl Instruction {
    pub fn op1_data(self, rs1: HOption<Register>, pc: u32) -> u32 {
        self.op1_sel
            .map(|sel| match sel {
                Op1Sel::Rs1 => rs1.unwrap().data,
                Op1Sel::Pc => pc,
                Op1Sel::Imm => self.imm,
            })
            .unwrap_or(0)
    }

    pub fn op2_data(self, rs2: HOption<Register>) -> u32 {
        self.op2_sel
            .map(|sel| match sel {
                Op2Sel::Rs2 => rs2.unwrap().data,
                Op2Sel::Four => 4,
                Op2Sel::Imm => self.imm,
            })
            .unwrap_or(0)
    }

    pub fn jmp_target(self, rs1: HOption<Register>, pc: u32) -> (u32, u32) {
        self.jmp_target_sel
            .map(|sel| match sel {
                JmpTargetSel::BType => (pc, self.imm),
                JmpTargetSel::JType => (self.imm, pc),
                JmpTargetSel::Jalr => (rs1.unwrap().data, self.imm),
            })
            .unwrap_or((0, 0))
    }
}

impl From<u32> for Instruction {
    fn from(value: u32) -> Self {
        let funct7 = (value & 0xfe000000) >> 25;
        let funct3 = (value & 0x00007000) >> 12;
        let opcode = value & 0x0000007f;

        /* RV32I Base Instruction Set */
        let is_lui = opcode == 0b0110111;
        let is_auipc = opcode == 0b0010111;

        let is_jal = opcode == 0b1101111;
        let is_jalr = funct3 == 0b000 && opcode == 0b1100111;
        let is_beq = funct3 == 0b000 && opcode == 0b1100011;
        let is_bne = funct3 == 0b001 && opcode == 0b1100011;
        let is_blt = funct3 == 0b100 && opcode == 0b1100011;
        let is_bge = funct3 == 0b101 && opcode == 0b1100011;
        let is_bltu = funct3 == 0b110 && opcode == 0b1100011;
        let is_bgeu = funct3 == 0b111 && opcode == 0b1100011;

        let is_lb = funct3 == 0b000 && opcode == 0b0000011;
        let is_lh = funct3 == 0b001 && opcode == 0b0000011;
        let is_lw = funct3 == 0b010 && opcode == 0b0000011;
        let is_lbu = funct3 == 0b100 && opcode == 0b0000011;
        let is_lhu = funct3 == 0b101 && opcode == 0b0000011;
        let is_sb = funct3 == 0b000 && opcode == 0b0100011;
        let is_sh = funct3 == 0b001 && opcode == 0b0100011;
        let is_sw = funct3 == 0b010 && opcode == 0b0100011;

        let is_addi = funct3 == 0b000 && opcode == 0b0010011;
        let is_slti = funct3 == 0b010 && opcode == 0b0010011;
        let is_sltiu = funct3 == 0b011 && opcode == 0b0010011;
        let is_xori = funct3 == 0b100 && opcode == 0b0010011;
        let is_ori = funct3 == 0b110 && opcode == 0b0010011;
        let is_andi = funct3 == 0b111 && opcode == 0b0010011;
        let is_slli = funct7 == 0b0000000 && funct3 == 0b001 && opcode == 0b0010011;
        let is_srli = funct7 == 0b0000000 && funct3 == 0b101 && opcode == 0b0010011;
        let is_srai = funct7 == 0b0100000 && funct3 == 0b101 && opcode == 0b0010011;

        let is_add = funct7 == 0b0000000 && funct3 == 0b000 && opcode == 0b0110011;
        let is_sub = funct7 == 0b0100000 && funct3 == 0b000 && opcode == 0b0110011;
        let is_sll = funct7 == 0b0000000 && funct3 == 0b001 && opcode == 0b0110011;
        let is_slt = funct7 == 0b0000000 && funct3 == 0b010 && opcode == 0b0110011;
        let is_sltu = funct7 == 0b0000000 && funct3 == 0b011 && opcode == 0b0110011;
        let is_xor = funct7 == 0b0000000 && funct3 == 0b100 && opcode == 0b0110011;
        let is_srl = funct7 == 0b0000000 && funct3 == 0b101 && opcode == 0b0110011;
        let is_sra = funct7 == 0b0100000 && funct3 == 0b101 && opcode == 0b0110011;
        let is_or = funct7 == 0b0000000 && funct3 == 0b110 && opcode == 0b0110011;
        let is_and = funct7 == 0b0000000 && funct3 == 0b111 && opcode == 0b0110011;

        let is_fence = funct3 == 0b000 && opcode == 0b0001111;
        let is_fencei = funct3 == 0b001 && opcode == 0b0001111;

        let is_ecall = value == 0b00000000000000000000000001110011;
        let is_ebreak = value == 0b00000000000100000000000001110011;

        let is_csrrw = funct3 == 0b001 && opcode == 0b1110011;
        let is_csrrs = funct3 == 0b010 && opcode == 0b1110011;
        let is_csrrc = funct3 == 0b011 && opcode == 0b1110011;
        let is_csrrwi = funct3 == 0b101 && opcode == 0b1110011;
        let is_csrrsi = funct3 == 0b110 && opcode == 0b1110011;
        let is_csrrci = funct3 == 0b111 && opcode == 0b1110011;

        /* RV Priviledged Set */
        let is_mret = value == 0x30200073;
        let is_wfi = value == 0x10500073;

        let l1 = is_lw || is_lb || is_lbu || is_lh || is_lhu || is_sw || is_sb || is_sh;
        let l2 = is_auipc || is_lui;
        let l3 = is_addi || is_andi || is_ori || is_xori || is_slti || is_sltiu || is_slli || is_srai || is_srli;
        let l4 = is_sll || is_add || is_sub || is_slt || is_sltu || is_and || is_or || is_xor || is_sra || is_srl;
        let l5 = is_jal || is_jalr || is_beq || is_bne || is_bge || is_bgeu || is_blt || is_bltu;
        let l6 = is_csrrwi || is_csrrsi || is_csrrw || is_csrrs || is_csrrc || is_csrrci;
        let l7 = is_ecall || is_mret || is_ebreak || is_wfi;
        let l8 = is_fencei || is_fence;

        let is_illegal = !(l1 || l2 || l3 || l4 || l5 || l6 || l7 || l8);
        let is_rtype = l4;
        let is_itype = is_lw || is_lb || is_lbu || is_lh || is_lhu || l3 || is_jalr;
        let is_stype = is_sw || is_sh || is_sb;
        let is_btype = is_beq || is_bne || is_bge || is_bgeu || is_blt || is_bltu;
        let is_utype = l2;
        let is_jtype = is_jal;
        let is_csr = is_csrrw || is_csrrs || is_csrrc;
        let is_csri = is_csrrwi || is_csrrsi || is_csrrci;

        let br_type = if is_beq {
            BranchType::Eq
        } else if is_bne {
            BranchType::Ne
        } else if is_bge {
            BranchType::Ge
        } else if is_bgeu {
            BranchType::Geu
        } else if is_blt {
            BranchType::Lt
        } else if is_bltu {
            BranchType::Ltu
        } else if is_jtype || is_jalr {
            BranchType::J
        } else {
            BranchType::N
        };

        let value = U::<32>::from(value);
        let rs1_addr = value.clip_const::<5>(15);
        let rs2_addr = value.clip_const::<5>(20);
        let rd_addr = value.clip_const::<5>(7);
        let csr_addr = value.clip_const::<12>(20);
        let itype_sext: U<32> = {
            let sign_bit = value.clip_const::<1>(31);
            (value.clip_const::<11>(20)).append(sign_bit.repeat::<21>().concat())
        };

        let rs1_addr = if is_rtype || is_itype || is_stype || is_btype || is_csr { Some(rs1_addr) } else { None };
        let rs2_addr = if is_rtype || is_stype || is_btype { Some(rs2_addr) } else { None };

        let rd_addr = if (is_rtype || is_itype || is_utype || is_jtype || is_csr || is_csri) && (rd_addr != U::from(0))
        {
            Some(rd_addr)
        } else {
            None
        };

        let imm = if is_lui || is_auipc {
            (value >> 12) << 12
        } else if is_jal {
            let imm_20 = value.clip_const::<1>(31); // this should  be sign-extneded
            let imm_10_1 = value.clip_const::<10>(21);
            let imm_11 = value.clip_const::<1>(20);
            let imm_19_12 = value.clip_const::<8>(12);
            let imm_0 = U::from(false);
            imm_0.append(imm_10_1).append(imm_11).append(imm_19_12).append(imm_20.repeat::<12>().concat())
        } else if is_jalr {
            itype_sext
        } else if is_btype {
            let imm_12 = value.clip_const::<1>(31); // this should be sign-extended
            let imm_10_5 = value.clip_const::<6>(25);
            let imm_4_1 = value.clip_const::<4>(8);
            let imm_11 = value.clip_const::<1>(7);
            let imm_0 = U::from(false);
            imm_0.append(imm_4_1).append(imm_10_5).append(imm_11).append(imm_12.repeat::<20>().concat())
        } else if is_lb || is_lh || is_lw || is_lbu || is_lhu {
            itype_sext
        } else if is_sb || is_sh || is_sw {
            let imm_11 = value.clip_const::<1>(31); // this should  be sign-extneded
            let imm_10_5 = value.clip_const::<6>(25);
            let imm_4_0 = value.clip_const::<5>(7);
            imm_4_0.append(imm_10_5).append(imm_11.repeat::<21>().concat())
        } else if is_addi || is_slti || is_sltiu || is_xori || is_ori || is_andi {
            itype_sext
        } else if is_slli || is_srli || is_srai {
            value.clip_const::<5>(20).append(U::<27>::from(0u32))
        } else if is_csri {
            value.clip_const::<5>(15).append(0.into_u())
        } else {
            U::from(0)
        };

        let alu_op = if is_sll || is_slli {
            BaseAluOp::Sll
        } else if is_add || is_addi || is_jalr {
            BaseAluOp::Add
        } else if is_sub {
            BaseAluOp::Sub
        } else if is_slt || is_slti {
            BaseAluOp::Slt
        } else if is_sltu || is_sltiu {
            BaseAluOp::Sltu
        } else if is_and || is_andi {
            BaseAluOp::And
        } else if is_or || is_ori {
            BaseAluOp::Or
        } else if is_xor || is_xori {
            BaseAluOp::Xor
        } else if is_sra || is_srai {
            BaseAluOp::Sra
        } else if is_srl || is_srli {
            BaseAluOp::Srl
        } else if is_lw || is_lh || is_lhu || is_lb || is_lbu || is_jtype || is_stype || is_auipc {
            BaseAluOp::Add
        } else if is_lui {
            BaseAluOp::CopyOp2
        } else if is_beq || is_bne {
            BaseAluOp::Xor
        } else if is_bge {
            BaseAluOp::Slt
        } else if is_bgeu {
            BaseAluOp::Sltu
        } else if is_blt {
            BaseAluOp::Slt
        } else if is_bltu {
            BaseAluOp::Sltu
        } else if is_csr || is_csri {
            BaseAluOp::CopyOp1
        } else {
            BaseAluOp::Zero
        };

        let wb_sel = if is_rtype {
            Some(WbSel::Alu)
        } else if is_itype {
            if is_lw || is_lh || is_lhu || is_lb || is_lbu {
                Some(WbSel::Mem)
            } else if is_jalr {
                Some(WbSel::Pc4)
            } else {
                Some(WbSel::Alu)
            }
        } else if is_utype {
            Some(WbSel::Alu)
        } else if is_jtype {
            Some(WbSel::Pc4)
        } else if is_stype || is_btype {
            None
        } else if is_csr || is_csri {
            Some(WbSel::Csr)
        } else {
            None
        };
        let imm = u32::from(imm);

        let csr_info = if is_csrrc || is_csrrci {
            Some(CsrInfo {
                addr: csr_addr,
                cmd: if rs1_addr == Some(U::from(0)) { CsrCommand::R } else { CsrCommand::C },
            })
        } else if is_csrrw || is_csrrwi {
            Some(CsrInfo { addr: csr_addr, cmd: CsrCommand::W })
        } else if is_csrrs || is_csrrsi {
            Some(CsrInfo {
                addr: csr_addr,
                cmd: if rs1_addr == Some(U::from(0)) { CsrCommand::R } else { CsrCommand::S },
            })
        } else if is_ecall || is_ebreak || is_mret {
            Some(CsrInfo { addr: csr_addr, cmd: CsrCommand::I })
        } else {
            None
        };

        let mem_info = if is_lw {
            Some((MemOpFcn::Load, MemOpTyp::W))
        } else if is_lh {
            Some((MemOpFcn::Load, MemOpTyp::H))
        } else if is_lhu {
            Some((MemOpFcn::Load, MemOpTyp::HU))
        } else if is_lb {
            Some((MemOpFcn::Load, MemOpTyp::B))
        } else if is_lbu {
            Some((MemOpFcn::Load, MemOpTyp::BU))
        } else if is_sw {
            Some((MemOpFcn::Store, MemOpTyp::W))
        } else if is_sh {
            Some((MemOpFcn::Store, MemOpTyp::H))
        } else if is_sb {
            Some((MemOpFcn::Store, MemOpTyp::B))
        } else {
            None
        };

        let op1_sel = if is_auipc || is_jtype {
            Some(Op1Sel::Pc)
        } else if is_csri {
            Some(Op1Sel::Imm)
        } else if is_lui || is_ecall || is_ebreak || is_fencei || is_fence || is_mret || is_wfi || is_illegal {
            None
        } else {
            Some(Op1Sel::Rs1)
        };

        let op2_sel = if is_rtype || is_btype {
            Some(Op2Sel::Rs2)
        } else if is_jtype {
            Some(Op2Sel::Four)
        } else if is_itype || is_stype || is_utype {
            Some(Op2Sel::Imm)
        } else {
            None
        };

        let jmp_target_sel = if is_btype {
            Some(JmpTargetSel::BType)
        } else if is_jtype {
            // JAL: Jump to `imm`.
            Some(JmpTargetSel::JType)
        } else if is_jalr {
            // JALR: Jump to `rs1` + `imm`.
            Some(JmpTargetSel::Jalr)
        } else {
            None
        };

        Self {
            is_illegal,
            br_type,
            rs1_addr,
            rs2_addr,
            rd_addr,
            imm,
            alu_op,
            wb_sel,
            is_fencei,
            csr_info,
            mem_info,
            jmp_target_sel,
            op1_sel,
            op2_sel,
        }
    }
}

/// Branch Type
// NOTE: We ordered variants for comb logic optimization
#[derive(Debug, Clone, Copy)]
pub enum BranchType {
    /// Next
    N,

    /// Jump
    J,

    /// Branch on Equal
    Eq,

    /// Branch on NotEqual
    Ne,

    /// Branch on Greater/Equal
    Ge,

    /// Branch on Less Than
    Lt,

    /// Branch on Greater/Equal Unsigned
    Geu,

    /// Branch on Less Than Unsigned
    Ltu,
}

/// Writeback Select Signal
#[derive(Debug, Clone, Copy)]
pub enum WbSel {
    /// ALU
    Alu,

    /// Memory
    Mem,

    /// PC + 4
    Pc4,

    /// CSR
    Csr,
}
