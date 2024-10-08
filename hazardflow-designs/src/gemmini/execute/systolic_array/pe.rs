//! Processing element.
//!
//! FIXME:
//! Currently, this implementation is assuming the base configuration(i.e., inputType = SInt(8.W), accType = SInt(32.W), spatialArrayOutputType = SInt(20.W))

#![allow(unused)] // Added for assignment.

use super::*;

/// PE Row Data
#[derive(Debug, Clone, Copy)]
pub struct PeRowData {
    /// a
    pub a: U<INPUT_BITS>,
}

/// PE Column Data
#[derive(Debug, Clone, Copy)]
pub struct PeColData {
    /// b
    pub b: U<OUTPUT_BITS>,
    /// d
    pub d: U<OUTPUT_BITS>,
}

/// Which register to use to preload the value
#[derive(Debug, Default, Clone, Copy)]
pub enum Propagate {
    /// use Reg1 for preload and Reg2 for computation
    #[default]
    Reg1,
    /// use Reg2 for preload and Reg1 for computation
    Reg2,
}

/// Is Dataflow Output-Stationary(OS) or Weight-Stationary(WS)?
#[derive(Debug, Clone, Copy)]
pub enum Dataflow {
    /// Output Stationary
    OS,
    /// Weight Stationary
    WS,
}

impl Default for Dataflow {
    fn default() -> Self {
        Self::OS
    }
}

impl From<U<1>> for Dataflow {
    fn from(value: U<1>) -> Self {
        Dataflow::from(value[0])
    }
}

impl From<bool> for Dataflow {
    fn from(value: bool) -> Self {
        match value {
            false => Self::OS,
            true => Self::WS,
        }
    }
}

/// PE Control
#[derive(Debug, Clone, Copy)]
pub struct PeControl {
    /// DataFlow
    pub dataflow: Dataflow,

    /// Propagate
    pub propagate: Propagate,

    /// Shift
    pub shift: U<5>,
}

/// PE column control.
///
/// NOTE: column data and control should be separated because of the `flush` operation.
/// <https://github.com/ucb-bar/gemmini/blob/be2e9f26181658895ebc7ca7f7d6be6210f5cdef/src/main/scala/gemmini/ExecuteController.scala#L189-L207>
#[derive(Debug, Clone, Copy)]
pub struct PeColControl {
    /// id
    pub id: U<ID_BITS>,
    /// is this last row?
    pub last: bool,
    /// pe control
    pub control: PeControl,
    /// bad_dataflow
    pub bad_dataflow: bool,
}

/// PE state.
#[derive(Debug, Default, Clone, Copy)]
pub struct PeS {
    /// Register 1
    pub reg1: U<32>,
    /// Register 2
    pub reg2: U<32>,
    /// Same as `last_s` in the Chisel implementation.
    pub propagate: Propagate,
}

impl PeS {
    /// Creates a new PE state.
    pub fn new(reg1: U<32>, reg2: U<32>, propagate: Propagate) -> Self {
        Self { reg1, reg2, propagate }
    }
}

/// MAC unit (computes `a * b + c`).
fn mac(a: U<8>, b: U<8>, c: U<32>) -> U<OUTPUT_BITS> {
    todo!("Assignment 4")
}

/// Returns whether there was a change in the propagate option.
///
/// NOTE: This is equivalent to `prev != curr`, but hazardflow compiler does not support it (ICE).
fn propagate_flipped(prev: Propagate, curr: Propagate) -> bool {
    matches!(prev, Propagate::Reg1) ^ matches!(curr, Propagate::Reg1)
}

/// Same as `(val >> shamt).clippedToWidthOf(20)`.
fn shift_and_clip(val: U<32>, shamt: U<5>) -> U<OUTPUT_BITS> {
    let shifted = rounding_shift(val, shamt);
    super::arithmetic::clip_with_saturation::<32, 20>(shifted)
}

/// PE.
#[synthesize]
pub fn pe(
    _in_left: Valid<PeRowData>,
    (_in_top_data, _in_top_control): (Valid<PeColData>, Valid<PeColControl>),
) -> (Valid<PeRowData>, (Valid<PeColData>, Valid<PeColControl>)) {
    todo!("Assignment 4")
}

/// Chisel PE Wrapper.
/// This module allows students to proceed with future assignments even if they have not completed Assignment4.
#[magic(ffi::PE256Wrapper())]
pub fn pe_256_chisel(
    _in_left: Valid<PeRowData>,
    (_in_top_data, _in_top_control): (Valid<PeColData>, Valid<PeColControl>),
) -> (Valid<PeRowData>, (Valid<PeColData>, Valid<PeColControl>)) {
    todo!("PE256Wrapper.v")
}
