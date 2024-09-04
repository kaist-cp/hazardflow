//! Related to branch prediction.

pub mod bht;
pub mod btb;
pub mod pre_decode;

pub use bht::*;
pub use btb::*;
pub use pre_decode::*;

/// Number of BHT entries.
pub const BHT_ENTRIES: usize = 128;
/// Number of BTB entries.
pub const BTB_ENTRIES: usize = 32;

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
