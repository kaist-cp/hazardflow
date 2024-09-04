//! Related to branch prediction.

pub mod bht;
pub mod btb;
pub mod pre_decode;

pub use bht::*;
pub use btb::*;
pub use pre_decode::*;

use super::MemRespWithAddr;
use crate::std::*;

/// Number of BHT entries.
pub const BHT_ENTRIES: usize = 128;
/// Number of BTB entries.
pub const BTB_ENTRIES: usize = 32;

/// Branch predictor with BHT and BTB.
#[derive(Debug, Default, Clone, Copy)]
pub struct Bp {
    /// BHT.
    pub bht: Bht,

    /// BTB.
    pub btb: Btb,
}

impl Bp {
    /// Returns the branch prediction result.
    pub fn predict(self, imem_resp: MemRespWithAddr) -> BpResult {
        BpResult {
            pre_decode: pre_decode(imem_resp.data.into_u()),
            bht: self.bht.predict(imem_resp.addr),
            btb: self.btb.predict(imem_resp.addr).unwrap_or(imem_resp.addr + 4),
        }
    }

    /// Updates the branch predictor.
    pub fn update(self, bp_update: BpUpdate) -> Self {
        match bp_update {
            BpUpdate::Bht { pc, taken } => Self { bht: self.bht.update(pc, taken), btb: self.btb },
            BpUpdate::Btb { pc, target } => Self { bht: self.bht, btb: self.btb.update(pc, target) },
        }
    }
}

/// Branch prediction results.
#[derive(Debug, Clone, Copy)]
pub struct BpResult {
    /// Pre-decode result.
    pub pre_decode: PreDecodeResp,

    /// Predicted branch direction (used for branch instructions).
    pub bht: bool,

    /// Predicted target address (used for JALR instruction).
    pub btb: u32,
}

/// Branch prediction update.
#[derive(Debug, Clone, Copy)]
pub enum BpUpdate {
    /// Updates BHT.
    ///
    /// It contains the branch instruction PC and the direction.
    Bht {
        /// Branch instruction PC.
        pc: u32,
        /// Taken or not taken.
        taken: bool,
    },

    /// Updates BTB.
    ///
    /// It contains the mispredicted PC and the correct target address.
    Btb {
        /// Mispredicted PC.
        pc: u32,
        /// Correct target address.
        target: u32,
    },
}
