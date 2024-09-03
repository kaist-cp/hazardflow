//! Pre-decode.

use crate::std::*;

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

    let uj_imm = |i: U<32>| {
        false
            .repeat::<1>()
            .append(i.clip_const::<10>(21))
            .append(i[20].repeat::<1>())
            .append(i.clip_const::<8>(12))
            .append(i[31].repeat::<12>())
    };

    let sb_imm = |i: U<32>| {
        false
            .repeat::<1>()
            .append(i.clip_const::<4>(8))
            .append(i.clip_const::<6>(25))
            .append(i[7].repeat::<1>())
            .append(i[31].repeat::<1>())
            .append(i[31].repeat::<19>())
    };

    let is_branch = opcode == 0b1100011.into_u();
    let is_jalr = funct3 == 0b000.into_u() && opcode == 0b1100111.into_u();
    let is_jal = opcode == 0b1101111.into_u();
    let imm = if i[3] { uj_imm(i) } else { sb_imm(i) };

    PreDecodeResp { is_branch, is_jalr, is_jal, imm }
}
