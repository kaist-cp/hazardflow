//! Transposer.

#![allow(unused)] // Added for assignment.

use super::*;

/// Indicates the direction of the Transposer PE.
#[derive(Debug, Default, Clone, Copy)]
enum Dir {
    /// Selects data from row side.
    #[default]
    Row,

    /// Selects data from column side.
    Col,
}

impl Dir {
    fn flip(self) -> Self {
        match self {
            Dir::Row => Dir::Col,
            Dir::Col => Dir::Row,
        }
    }
}

/// Transposer PE.
fn transposer_pe(
    in_row: Valid<S<INPUT_BITS>>,
    (in_col, in_dir): (Valid<S<INPUT_BITS>>, Valid<Dir>),
) -> (Valid<S<INPUT_BITS>>, (Valid<S<INPUT_BITS>>, Valid<Dir>)) {
    todo!("assignment 5")
}

/// Systolic array of Transposer PEs.
#[allow(clippy::type_complexity)]
fn transposer_pes<const DIM: usize>(
    in_row: [Valid<S<INPUT_BITS>>; DIM],
    in_col_with_dir: [(Valid<S<INPUT_BITS>>, Valid<Dir>); DIM],
) -> ([Valid<S<INPUT_BITS>>; DIM], [(Valid<S<INPUT_BITS>>, Valid<Dir>); DIM]) {
    todo!("assignment 5")
}

/// Unzips the valid interfaces in the array.
fn unzip_tuple_arr<P1: Copy, P2: Copy, const N: usize>(i: [Valid<(P1, P2)>; N]) -> [(Valid<P1>, Valid<P2>); N] {
    // NOTE: The `array_map!` macro currently does not accept closures as arguments, so we explicitly used `Valid::<(P1, P2)>::unzip`
    //       instead of `move |i| i.unzip()`.
    array_map!(i, Valid::<(P1, P2)>::unzip)
}

/// Zips the valid interfaces in the array.
fn zip_tuple_arr<P1: Copy, P2: Copy, const N: usize>(i: [(Valid<P1>, Valid<P2>); N]) -> [Valid<(P1, P2)>; N] {
    // NOTE: The `array_map!` macro currently does not accept closures as arguments, so we explicitly used `JoinValidExt::join_valid`
    //       instead of `move |(i1, i2)| (i1, i2).join_valid()`.
    array_map!(i, JoinValidExt::join_valid)
}

/// Transposer.
pub fn transposer<const DIM: usize>(i: Valid<Array<S<INPUT_BITS>, DIM>>) -> Valid<Array<S<INPUT_BITS>, DIM>>
where
    [(); clog2(DIM)]:,
    [(); clog2(DIM) + 1]:,
{
    todo!("assignment 5")
}

/// Transposer with default Gemmini configuration (16 x 16 Transposer PEs).
#[synthesize]
pub fn transposer_default(in_row: Valid<Array<S<INPUT_BITS>, 16>>) -> Valid<Array<S<INPUT_BITS>, 16>> {
    transposer::<16>(in_row)
}
