//! SRAM: This module contains the implementation of Scratchpad and Accumulator.

pub mod accumulator;
pub mod dma;
pub mod scratchpad;

use accumulator::*;
use scratchpad::*;

use crate::gemmini::*;

/// # SramAddr
///
/// Sram has two types of memory: Scratchpad and Accumulator.
/// Each inner field indicates bank id and address.
///
/// Used in execute module
#[derive(Debug, Clone, Copy)]
pub enum SramAddr {
    /// Address for Scratchpad
    Spad {
        /// Bank id
        bank: U<2>,
        /// Address
        address: U<14>,
    },
    /// Address for Accumulator
    Acc {
        /// Bank id
        bank: U<1>,
        /// Address
        address: U<14>,
    },
}

/// `TLBReq`: <https://github.com/chipsalliance/rocket-chip/blob/master/src/main/scala/rocket/TLB.scala#L47-L60>
#[derive(Debug, Clone, Copy)]
pub struct TlbReq;

/// `TlbResp`: <https://github.com/chipsalliance/rocket-chip/blob/master/src/main/scala/rocket/TLB.scala#L68C7-L89>
#[derive(Debug, Clone, Copy)]
pub struct TlbResp;

/// SRAM in the Gemmini
///
/// Gemmini stores inputs and outputs for the systolic array in a set of private SRAMs, which we call the "scratchpad" and the "accumulator".
/// Typically, inputs are stored in the scratchpad, while partial sums and final results are stored in the the accumulator.
///
/// <https://github.com/ucb-bar/gemmini/blob/be2e9f26181658895ebc7ca7f7d6be6210f5cdef/src/main/scala/gemmini/Scratchpad.scala#L172>
#[allow(clippy::type_complexity)]
pub fn sram(
    _dma: (Vr<ScratchpadMemReadReq<MVIN_SCALE_BITS>>, Vr<ScratchpadMemWriteReq<32, ACC_SCALE_BITS>>),
    _exe: (
        ([Vr<ScratchpadReadReq, { Dep::Demanding }>; SP_BANKS], [Valid<ScratchpadWriteReq>; SP_BANKS]),
        ([Vr<AccumulatorReadReq, { Dep::Demanding }>; ACC_BANKS], [Valid<AccumulatorWriteReq>; ACC_BANKS]),
    ),
    // tlb_accessor: impl FnOnce([Vr<TlbResp>; 2]) -> [Valid<TlbReq>; 2],   // TODO: Should figure out how SRAM interacts with TLB (and other modules)
) -> (
    (Valid<ScratchpadMemReadResp>, Valid<ScratchpadMemWriteResp>),
    (([Vr<ScratchpadReadResp>; SP_BANKS], ()), ([Vr<AccumulatorReadResp>; ACC_BANKS], ())),
) {
    todo!()
}
