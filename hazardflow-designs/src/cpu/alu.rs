//! ALU.

use super::*;

/// ALU input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AluInput {
    /// Operator.
    pub op: AluOp,

    /// First operand data.
    pub op1_data: u32,

    /// Second operand data.
    pub op2_data: u32,
}

/// ALU operation signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AluOp {
    /// Base op.
    Base(BaseAluOp),
    /// M extension op.
    Mext(MulOp),
}

/// Base ALU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseAluOp {
    /// Addition
    Add,

    /// Subtraction
    Sub,

    /// Logical left shift
    Sll,

    /// Logical right shift
    Srl,

    /// Arithmetic right shift
    Sra,

    /// And
    And,

    /// Or
    Or,

    /// Xor
    Xor,

    /// Set less than
    Slt,

    /// Set less than unsigned
    Sltu,

    /// Copy op1
    CopyOp1,

    /// Copy op2
    CopyOp2,

    /// Zero
    Zero,
}

/// Execute alu
pub fn exe_alu(alu_op1: u32, alu_op2: u32, op: BaseAluOp) -> u32 {
    let alu_shamt = alu_op2 & 0x1f;

    match op {
        BaseAluOp::Add => alu_op1 + alu_op2,
        BaseAluOp::Sub => alu_op1 - alu_op2,
        BaseAluOp::And => alu_op1 & alu_op2,
        BaseAluOp::Or => alu_op1 | alu_op2,
        BaseAluOp::Xor => alu_op1 ^ alu_op2,
        BaseAluOp::Slt => ((alu_op1 as i32) < (alu_op2 as i32)) as u32,
        BaseAluOp::Sltu => (alu_op1 < alu_op2) as u32,
        BaseAluOp::Sll => alu_op1 << alu_shamt,
        BaseAluOp::Sra => ((alu_op1 as i32) >> alu_shamt) as u32,
        BaseAluOp::Srl => alu_op1 >> alu_shamt,
        BaseAluOp::CopyOp1 => alu_op1,
        BaseAluOp::CopyOp2 => alu_op2,
        BaseAluOp::Zero => 0,
    }
}
