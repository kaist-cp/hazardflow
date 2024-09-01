//! Scratchpad Memory
//!
//! TODO: Hardcoded values
//! - data width of `addr` field of ScratchpadReadReq is hardcoded to 12 bits
//! - data width of `data` field of ScratchpadReadResp is hardcoded to 128 bits

use crate::gemmini::isa::rocc::*;
use crate::gemmini::local_addr::*;
use crate::gemmini::*;

/// Data width of entry in the scratchpad.
pub const SP_DATA_WIDTH: usize = 16 * 8; // (meshColumns * tileColumns) * inputType.getWidth
/// Mask width of entry in the scratchpad.
pub const SP_MASK_WIDTH: usize = SP_DATA_WIDTH / 8;

/// Scratchpad memory read request.
#[derive(Debug, Clone, Copy)]
pub struct ScratchpadMemReadReq<const SCALE_BITS: usize> {
    /// Virtual address.
    pub vaddr: U<CORE_MAX_ADDR_BITS>,
    /// Local address.
    pub laddr: LocalAddr, // TODO: Don't use a magic number here

    /// TODO: Documentation
    pub cols: U<16>, // TODO: Don't use a magic number here
    /// TODO: Documentation
    pub repeats: U<16>, // TODO: Don't use a magic number here
    /// TODO: Documentation
    pub scale: U<SCALE_BITS>,
    /// TODO: Documentation
    pub has_acc_bitwidth: bool,
    /// TODO: Documentation
    pub all_zeros: bool,
    /// TODO: Documentation
    pub block_stride: U<16>, // TODO: Don't use a magic number here
    /// TODO: Documentation
    pub pixel_repeats: U<8>, // TODO: Don't use a magic number here
    /// TODO: Documentation
    pub cmd_id: U<8>, // TODO: Don't use a magic number here
    /// TODO: Documentation
    pub status: MStatus,
}

/// Scratchpad memory read response.
#[derive(Debug, Clone, Copy)]
pub struct ScratchpadMemReadResp {
    /// Bytes to read.
    pub bytes_read: U<16>, // TODO: Don't use a magic number here
    /// Command ID.
    pub cmd_id: U<8>, // TODO: Don't use a magic number here
}

/// Scratchpad memory write request.
#[derive(Debug, Clone, Copy)]
pub struct ScratchpadMemWriteReq<const ACC_BITS: usize, const SCALE_BITS: usize> {
    /// TODO: Documentation
    pub vaddr: U<CORE_MAX_ADDR_BITS>,
    /// TODO: Documentation
    pub laddr: LocalAddr,

    /// TODO: Documentation
    pub acc_act: U<3>,
    /// TODO: Documentation
    pub acc_scale: U<SCALE_BITS>,
    /// TODO: Documentation
    pub acc_igelu_qb: U<ACC_BITS>,
    /// TODO: Documentation
    pub acc_igelu_qc: U<ACC_BITS>,
    /// TODO: Documentation
    pub acc_iexp_qln2: U<ACC_BITS>,
    /// TODO: Documentation
    pub acc_iexp_qln2_inv: U<ACC_BITS>,
    /// TODO: Documentation
    pub acc_norm_stats_id: U<8>, // TODO: Don't use a magic number here

    /// TODO: Documentation
    pub len: U<16>,
    /// TODO: Documentation
    pub block: U<8>,

    /// TODO: Documentation
    pub cmd_id: U<8>,
    /// TODO: Documentation
    pub status: MStatus,

    /// TODO: Documentation
    pub pool_en: bool,
    /// TODO: Documentation
    pub store_en: bool,
}

/// Scratchpad memory write response.
#[derive(Debug, Clone, Copy)]
pub struct ScratchpadMemWriteResp {
    /// Command ID.
    pub cmd_id: U<8>,
}

/// ScratchpadReadReq
///
/// <https://github.com/ucb-bar/gemmini/blob/be2e9f26181658895ebc7ca7f7d6be6210f5cdef/src/main/scala/gemmini/Scratchpad.scala#L75>
#[derive(Debug, Clone, Copy)]
pub struct ScratchpadReadReq {
    /// Address.
    pub addr: U<{ clog2(SP_BANK_ENTRIES) }>,
    /// Request was from DMA or not.
    pub from_dma: bool,
}

/// Scratchpad Read Response
///
/// <https://github.com/ucb-bar/gemmini/blob/be2e9f26181658895ebc7ca7f7d6be6210f5cdef/src/main/scala/gemmini/Scratchpad.scala#L80>
#[derive(Debug, Clone, Copy)]
pub struct ScratchpadReadResp {
    /// Data.
    pub data: U<SP_DATA_WIDTH>,
    /// Request was from DMA or not.
    pub from_dma: bool,
}

/// Scratchpad Write Request
///
/// Note: There is no response for Scratchpad Write
///
/// <https://github.com/ucb-bar/gemmini/blob/be2e9f26181658895ebc7ca7f7d6be6210f5cdef/src/main/scala/gemmini/Scratchpad.scala#L90>
#[derive(Debug, Clone, Copy)]
pub struct ScratchpadWriteReq {
    /// Address.
    pub addr: U<{ clog2(SP_BANK_ENTRIES) }>,
    /// Data.
    pub data: U<SP_DATA_WIDTH>,
    /// Mask.
    pub mask: U<SP_MASK_WIDTH>, // sub word write.
}

/// Scratchpad Bank
///
/// <https://github.com/ucb-bar/gemmini/blob/be2e9f26181658895ebc7ca7f7d6be6210f5cdef/src/main/scala/gemmini/Scratchpad.scala#L97>
pub fn spad_bank(
    _read_req: Vr<ScratchpadReadReq>,
    _write_req: Valid<ScratchpadWriteReq>,
) -> (Vr<ScratchpadReadResp>, ()) {
    todo!()
}
