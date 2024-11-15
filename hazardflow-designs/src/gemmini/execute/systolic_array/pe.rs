//! Processing element.

#![allow(unused)] // Added for assignment.

use super::*;

/// Bit width of the register type.
pub const ACC_BITS: usize = 32;

/// PE row data signals.
#[derive(Debug, Clone, Copy)]
pub struct PeRowData {
    /// A.
    ///
    /// Represents the activation value.
    pub a: S<INPUT_BITS>,
}

/// PE column data signals.
#[derive(Debug, Clone, Copy)]
pub struct PeColData {
    /// B.
    ///
    /// Represents the weight value (in OS dataflow) or the above PE's MAC result (in WS dataflow).
    pub b: S<OUTPUT_BITS>,

    /// D.
    ///
    /// Represents the preloading bias value (in OS dataflow) or the preloading weight value (in WS dataflow).
    pub d: S<OUTPUT_BITS>,
}

/// PE column control signals.
#[derive(Debug, Clone, Copy)]
pub struct PeColControl {
    /// Identifier for the matrix multiplication operation (not used in the PE logic).
    pub id: U<ID_BITS>,

    /// Indicates whether the current row is the last row (not used in the PE logic).
    pub last: bool,

    /// PE control signals.
    pub control: PeControl,
}

/// PE control signals.
#[derive(Debug, Clone, Copy)]
pub struct PeControl {
    /// Represents the dataflow.
    pub dataflow: Dataflow,

    /// Indicates which register to use for preloading the value.
    pub propagate: Propagate,

    /// The number of bits by which the accumulated result of matrix multiplication is right-shifted when leaving the
    /// systolic array, used to scale down the result.
    pub shift: U<{ clog2(ACC_BITS) }>,
}

/// Represents the dataflow.
#[derive(Debug, Default, Clone, Copy, HEq)]
pub enum Dataflow {
    /// Output stationary.
    #[default]
    OS,

    /// Weight stationary.
    WS,
}

/// Indicates which register to use for preloading the value.
#[derive(Debug, Default, Clone, Copy, HEq)]
pub enum Propagate {
    /// Use register 1 for preloading (and register 2 for the MAC unit input).
    #[default]
    Reg1,

    /// Use register 2 for preloading (and register 1 for the MAC unit input).
    Reg2,
}

/// PE state registers.
///
/// Each register stores values based on the dataflow and propagate signal:
///
/// - WS dataflow, preload: weight value for the next operation.
/// - WS dataflow, compute: weight value for the current operation.
/// - OS dataflow, preload: bias value for the next operation.
/// - OS dataflow, compute: partial sum value for the current operation.
///
/// NOTE: In OS dataflow, it outputs the matmul result when a change in the propagate value is detected.
#[derive(Debug, Default, Clone, Copy)]
pub struct PeS {
    /// Register 1.
    pub reg1: S<ACC_BITS>,

    /// Register 2.
    pub reg2: S<ACC_BITS>,

    /// The propagate value comes from the previous input.
    ///
    /// NOTE: In the PE logic, it is only used to check whether the current propagate value differs from the previous one.
    pub propagate: Propagate,
}

impl PeS {
    /// Creates a new PE state.
    pub fn new(reg1: S<ACC_BITS>, reg2: S<ACC_BITS>, propagate: Propagate) -> Self {
        Self { reg1, reg2, propagate }
    }

    /// Creates a new PE state for OS dataflow.
    ///
    /// # Arguments
    ///
    /// - `preload`: Bias value for the next operation.
    /// - `partial_sum`: MAC result of the current operation.
    /// - `propagate`: Propagate value.
    pub fn new_os(preload: S<OUTPUT_BITS>, partial_sum: S<OUTPUT_BITS>, propagate: Propagate) -> Self {
        let preload = preload.sext::<ACC_BITS>();
        let partial_sum = partial_sum.sext::<ACC_BITS>();

        match propagate {
            Propagate::Reg1 => PeS::new(preload, partial_sum, propagate),
            Propagate::Reg2 => PeS::new(partial_sum, preload, propagate),
        }
    }

    /// Creates a new PE state for WS dataflow.
    ///
    /// # Arguments
    ///
    /// - `preload`: Weight value for the next operation.
    /// - `weight`: Weight value for the current operation.
    /// - `propagate`: Propagate value.
    pub fn new_ws(preload: S<INPUT_BITS>, weight: S<INPUT_BITS>, propagate: Propagate) -> Self {
        let preload = preload.sext::<ACC_BITS>();
        let weight = weight.sext::<ACC_BITS>();

        match propagate {
            Propagate::Reg1 => PeS::new(preload, weight, propagate),
            Propagate::Reg2 => PeS::new(weight, preload, propagate),
        }
    }
}

/// MAC unit (computes `a * b + c`).
///
/// It preserves the signedness of operands.
fn mac(a: S<INPUT_BITS>, b: S<INPUT_BITS>, c: S<ACC_BITS>) -> S<OUTPUT_BITS> {
    super::arithmetic::mac(a, b, c)
}

/// Performs right-shift (`val >> shamt`) and then clips to `OUTPUT_BITS`.
///
/// It preserves the signedness of `val`.
fn shift_and_clip(val: S<ACC_BITS>, shamt: U<{ clog2(ACC_BITS) }>) -> S<OUTPUT_BITS> {
    let shifted = rounding_shift(val, shamt);
    super::arithmetic::clip_with_saturation::<ACC_BITS, OUTPUT_BITS>(shifted)
}

/// PE.
///
/// NOTE: It is assumed that all valid signals for the input interfaces have the same value.
#[synthesize]
pub fn pe(
    _in_left: Valid<PeRowData>,
    (_in_top_data, _in_top_control): (Valid<PeColData>, Valid<PeColControl>),
) -> (Valid<PeRowData>, (Valid<PeColData>, Valid<PeColControl>)) {
    todo!("assignment 4")
}
