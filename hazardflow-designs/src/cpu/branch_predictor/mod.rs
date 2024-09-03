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

/// Branch prediction info.
#[derive(Debug, Clone, Copy)]
pub struct BpInfo {
    /// Pre-decode result.
    pub pre_decoded: PreDecodeResp,

    /// Branch was taken or not?
    pub is_taken: bool,

    /// Target address.
    pub target: u32,
}

/// Branch prediction update.
#[derive(Debug, Clone, Copy)]
pub enum BpUpdate {
    /// Updates BHT. Contains PC.
    Bht(u32),

    /// Updates BTB. Contains (PC, target).
    Btb(u32, u32),
}
