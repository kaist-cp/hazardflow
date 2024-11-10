//! Mesh.

#![allow(unused)] // Added for assignment.

use super::tile::*;
use super::*;

/// Mesh row data. It consists of `MESH_ROWS` tile row data.
pub type MeshRowData = [TileRowData; MESH_ROWS];

/// Mesh column data. It consists of `MESH_COLS` tile column data.
pub type MeshColData = [TileColData; MESH_COLS];

/// Mesh.
pub fn mesh(in_left: MeshRowData, in_top: MeshColData) -> (MeshRowData, MeshColData) {
    todo!("assignment 5")
}

/// Debug
#[synthesize]
pub fn mesh_4_4(in_left: MeshRowData, in_top: MeshColData) -> (MeshRowData, MeshColData) {
    mesh(in_left, in_top)
}
