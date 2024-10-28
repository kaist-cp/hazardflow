//! Transposer.

#![allow(unused)] // Added for assignment.

use super::*;

#[derive(Debug, Clone, Copy)]
enum Dir {
    Left,
    Up,
}

impl Dir {
    fn flip(self) -> Self {
        match self {
            Dir::Left => Dir::Up,
            Dir::Up => Dir::Left,
        }
    }
}

/// Returns `(out_right, (out_bottom, dir))`.
fn t_pe(
    in_left: Valid<U<INPUT_BITS>>,
    (in_top, dir): (Valid<U<INPUT_BITS>>, Valid<Dir>),
) -> (Valid<U<INPUT_BITS>>, (Valid<U<INPUT_BITS>>, Valid<Dir>)) {
    todo!("assignment 5")
}

// Helper functions to use `array_map`.
// Currently, array_map does not take closure as an argument, so we need to define a helper function.
fn unzip_tup_interface(i: Valid<(U<INPUT_BITS>, Dir)>) -> (Valid<U<INPUT_BITS>>, Valid<Dir>) {
    i.unzip()
}
fn extract_first(i: (Valid<U<INPUT_BITS>>, Valid<Dir>)) -> Valid<U<INPUT_BITS>> {
    i.0
}

/// Always out transposer.
pub fn transposer<const DIM: usize>(_in_row: Valid<Array<U<INPUT_BITS>, DIM>>) -> Valid<Array<U<INPUT_BITS>, DIM>>
where
    [(); max(clog2(DIM), 1)]:,
    [(); max(clog2(DIM), 1) + 1]:,
{
    todo!("assignment 5")
}

/// Debug
#[synthesize]
pub fn transposer_default(in_row: Valid<Array<U<INPUT_BITS>, 16>>) -> Valid<Array<U<INPUT_BITS>, 16>> {
    transposer::<16>(in_row)
}

/// Chisel Transposer Wrapper.
///
/// This module allows students to proceed with future assignments even if they have not completed assignment 5.
#[magic(ffi::TransposerWrapper())]
pub fn transposer_chisel(_in_row: Valid<Array<U<INPUT_BITS>, 16>>) -> Valid<Array<U<INPUT_BITS>, 16>> {
    todo!("TransposerWrapper.v")
}
