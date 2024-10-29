//! Processing element.

#![allow(unused)] // Added for assignment.

use super::*;

/// PE row data signals.
#[derive(Debug, Clone, Copy)]
pub struct PeRowData {
    /// A.
    pub a: U<INPUT_BITS>,
}

/// PE column data signals.
#[derive(Debug, Clone, Copy)]
pub struct PeColData {
    /// B.
    pub b: U<OUTPUT_BITS>,

    /// D.
    pub d: U<OUTPUT_BITS>,
}

/// PE column control signals.
///
/// NOTE: The column data and control signals should be separated to handle the `flush` operation.
///       <https://github.com/ucb-bar/gemmini/blob/be2e9f26181658895ebc7ca7f7d6be6210f5cdef/src/main/scala/gemmini/ExecuteController.scala#L189-L207>
#[derive(Debug, Clone, Copy)]
pub struct PeColControl {
    /// ID.
    pub id: U<ID_BITS>,

    /// Is this last row?
    pub last: bool,

    /// PE control signals.
    pub control: PeControl,
}

/// Represents which register to use to preload the value.
#[derive(Debug, Default, Clone, Copy, HEq)]
pub enum Propagate {
    /// Use `Reg1` for preloading and `Reg2` for computation.
    #[default]
    Reg1,

    /// Use `Reg2` for preloading and `Reg1` for computation.
    Reg2,
}

/// Represents the dataflow.
#[derive(Debug, Default, Clone, Copy, HEq)]
pub enum Dataflow {
    /// Output Stationary.
    #[default]
    OS,

    /// Weight Stationary.
    WS,
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

/// PE control data.
#[derive(Debug, Clone, Copy)]
pub struct PeControl {
    /// Dataflow.
    pub dataflow: Dataflow,

    /// Propagate.
    pub propagate: Propagate,

    /// Shift.
    pub shift: U<5>,
}

/// PE state.
#[derive(Debug, Default, Clone, Copy)]
pub struct PeS {
    /// Register 1.
    pub reg1: U<32>,

    /// Register 2.
    pub reg2: U<32>,

    /// Propagate.
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
    todo!("assignment 4")
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
    todo!("assignment 4")
}
