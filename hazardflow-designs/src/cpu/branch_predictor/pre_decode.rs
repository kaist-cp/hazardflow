//! Pre-decode.

use super::*;

/// Pre-decode response.
#[derive(Debug, Clone, Copy)]
pub struct PreDecodeResp {
    /// Is branch instruction?
    pub is_branch: bool,

    /// Is JALR instruction?
    pub is_jalr: bool,

    /// Is JAL instruction?
    pub is_jal: bool,

    /// Immediate.
    pub imm: U<32>,
}

/// Performs pre-decode the bytecode.
///
/// It is used in the fetch stage to extract minimum required information for branch prediction.
pub fn pre_decode(i: U<32>) -> PreDecodeResp {
    let funct3 = i.clip_const::<3>(12);
    let opcode = i.clip_const::<7>(0);

    let is_branch = opcode == 0b1100011.into_u();
    let is_jalr = funct3 == 0b000.into_u() && opcode == 0b1100111.into_u();
    let is_jal = opcode == 0b1101111.into_u();
    let imm = if i[3] { imm_jtype(i) } else { imm_btype(i) };

    PreDecodeResp { is_branch, is_jalr, is_jal, imm }
}
