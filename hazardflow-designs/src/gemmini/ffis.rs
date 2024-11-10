//! Modules implemented as FFI.

#![allow(unused_variables)]

use super::execute::systolic_array::mesh::*;
use super::execute::systolic_array::mesh_with_delays::*;
use super::execute::systolic_array::pe::*;
use super::*;

/// Chisel MeshWithDelays Wrapper.
#[magic(ffi::MeshWithDelaysWrapper())]
pub fn mesh_with_delays_ffi(
    a: Vr<A>,
    b: Vr<B>,
    d: Vr<D>,
    req: I<VrH<MeshReq, TagsInProgress>, { Dep::Helpful }>,
) -> Valid<MeshResp> {
    ffi!("MeshWithDelaysWrapper.v")
}

/// Chisel Mesh Wrapper.
///
/// This module allows students to proceed with future assignments even if they have not completed assignment 5.
#[magic(ffi::MeshWrapper())]
pub fn mesh_ffi(in_left: MeshRowData, in_top: MeshColData) -> (MeshRowData, MeshColData) {
    ffi!("MeshWrapper.v")
}

/// Chisel Transposer Wrapper.
///
/// This module allows students to proceed with future assignments even if they have not completed assignment 5.
#[magic(ffi::TransposerWrapper())]
pub fn transposer_ffi(in_row: Valid<Array<S<INPUT_BITS>, 16>>) -> Valid<Array<S<INPUT_BITS>, 16>> {
    ffi!("TransposerWrapper.v")
}

/// Chisel PE Wrapper.
///
/// This module allows students to proceed with future assignments even if they have not completed assignment 4.
#[magic(ffi::PE256Wrapper())]
pub fn pe_ffi(
    in_left: Valid<PeRowData>,
    (in_top_data, in_top_control): (Valid<PeColData>, Valid<PeColControl>),
) -> (Valid<PeRowData>, (Valid<PeColData>, Valid<PeColControl>)) {
    ffi!("PE256Wrapper.v")
}
