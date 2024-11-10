//! Tile.

use super::pe::*;
use super::*;

/// Tile row data. It consists of `TILE_ROWS` PE row data.
pub type TileRowData = [Valid<PeRowData>; TILE_ROWS];

/// Tile column data. It consists of `TILE_COLS` PE column data and control.
pub type TileColData = [(Valid<PeColData>, Valid<PeColControl>); TILE_COLS];

/// Tile.
pub fn tile(in_left: TileRowData, in_top: TileColData) -> (TileRowData, TileColData) {
    // Constructs row of the tile, which has `1 x TILE_COLS` size.
    let row = flip(seq(from_fn(flip(pe_ffi)))); // Using `pe_ffi` instead of `pe` for now.

    // Constructs tile, which has `TILE_ROWS x TILE_COLS` size.
    let tile = seq(from_fn(row));

    tile(in_left, in_top)
}

/// Tile with default Gemmini configuration (1 x 1 PEs).
#[synthesize]
pub fn tile_default(in_left: TileRowData, in_top: TileColData) -> (TileRowData, TileColData) {
    tile(in_left, in_top)
}
