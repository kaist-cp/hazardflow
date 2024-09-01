//! Mesh.
#![allow(unused)] // Added for assignment.

use super::tile::*;
use super::*;

/// Mesh row data. It consists of `MESH_ROWS` tile row data.
pub type MeshRowData = [TileRowData; MESH_ROWS];

/// Mesh column data. It consists of `MESH_COLS` tile column data.
pub type MeshColData = [TileColData; MESH_COLS];

/// Mesh.
pub fn mesh<const LATENCY: usize>(in_left: MeshRowData, in_top: MeshColData) -> (MeshRowData, MeshColData)
where [(); 1 + LATENCY]: {
    todo!("Assignment 5")
}

/// Debug
#[synthesize]
pub fn mesh_4_4(in_left: MeshRowData, in_top: MeshColData) -> (MeshRowData, MeshColData) {
    // Only the Column data are used as output
    mesh::<1>(in_left, in_top)
}

/// Chisel Mesh Wrapper.
/// This module allows students to proceed with future assignments even if they have not completed Assignment5.
#[magic(ffi::MeshWrapper())]
pub fn mesh_chisel(_in_left: MeshRowData, _in_top: MeshColData) -> (MeshRowData, MeshColData) {
    todo!("MeshWrapper.v")
}
