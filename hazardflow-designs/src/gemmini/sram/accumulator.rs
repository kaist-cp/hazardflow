//! Accumulator
//! <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/AccumulatorMem.scala>
//! <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/AccumulatorScale.scala>

use super::*;

/// Data width of entry in the scratchpad.
pub const ACC_DATA_WIDTH: usize = 128;

/// Accumulator Read Request
///
/// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/AccumulatorMem.scala#L8>
#[derive(Debug, Clone, Copy)]
pub struct AccumulatorReadReq {
    // TODO: modify data types
    /// acc_scale
    pub scale: U<32>,
    /// full
    pub full: bool,
    /// activation
    pub act: U<3>,
    /// fromDMA
    pub from_dma: bool,
    /// accumulator address
    pub addr: U<9>,
}

/// Accumulator Read Response
///
/// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/AccumulatorMem.scala#L22>
#[derive(Debug, Clone, Copy)]
pub struct AccumulatorReadResp {
    /// data
    pub data: U<ACC_DATA_WIDTH>,
    /// from_dma
    pub from_dma: bool,
}

/// Accumulator Write Request
///
/// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/AccumulatorMem.scala#L39>
#[derive(Debug, Clone, Copy)]
pub struct AccumulatorWriteReq {
    /// Address.
    pub addr: U<{ clog2(ACC_BANK_ENTRIES) }>,
    /// Data.
    pub data: Array<U<32>, 16>,
    /// TODO: Documentation
    pub acc: bool,
    /// TODO: Documentation
    pub mask: U<64>, // Vec(t.getWidth / 8, Bool() * 16)  == 32 / 8 * 16
}

/// Accumulator Bank
///
/// <https://github.com/ucb-bar/gemmini/blob/be2e9f26181658895ebc7ca7f7d6be6210f5cdef/src/main/scala/gemmini/Scratchpad.scala#L640>
/// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/AccumulatorMem.scala#L92C7-L92C21>
pub fn accumulator_bank(
    _read_req: Vr<AccumulatorReadReq>,
    _write_req: Vr<AccumulatorWriteReq>,
) -> (Vr<AccumulatorReadResp>, ()) {
    todo!()
}
