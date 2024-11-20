//! Configurations.

use crate::std::*;

/* From external projects (e.g., rocket-chip) */

/// TODO: Documentation
pub const CORE_MAX_ADDR_BITS: usize = 40;

/* From `defaultConfig` in `Configs.scala`. */

/// Mesh rows.
pub const MESH_ROWS: usize = 16;
/// Mesh columns.
pub const MESH_COLS: usize = 16;
/// Tile rows.
pub const TILE_ROWS: usize = 1;
/// Tile columns.
pub const TILE_COLS: usize = 1;

/// Block Size
pub const BLOCK_SIZE: usize = MESH_ROWS * TILE_ROWS;

/// Number of banks in the scratchpad
pub const SP_BANKS: usize = 4;
/// Number of banks in the accumulator
pub const ACC_BANKS: usize = 2;

/// Reservation station load queue entries.
pub const RS_ENTRIES_LD: usize = 8;
/// Reservation station store queue entries.
pub const RS_ENTRIES_ST: usize = 4;
/// Reservation station execution queue entries.
pub const RS_ENTRIES_EX: usize = 16;

/// TODO: Documentation
pub const DMA_MAX_BYTES: usize = 64;

/* From `GemminiConfigs.scala`. */

/// Scratchpad width.
pub const SP_WIDTH: usize = MESH_COLS * TILE_COLS * 8;
/// Scratchpad bank entries.
pub const SP_BANK_ENTRIES: usize = 256 * 1024 * 8 / (SP_BANKS * SP_WIDTH);
/// Accumulator bank entries.
pub const ACC_BANK_ENTRIES: usize = 64 * 1024 * 8 / (ACC_BANKS * MESH_COLS * TILE_COLS * 32);

/// TODO: Documentation
pub const MVIN_SCALE_BITS: usize = 32;

/// TODO: Documentation
pub const ACC_SCALE_BITS: usize = 32;

/// TODO: Documentation
pub const MVIN_COLS_BITS: usize = clog2(max(DMA_MAX_BYTES, MESH_COLS * TILE_COLS) + 1);
/// TODO: Documentation
pub const MVIN_ROWS_BITS: usize = clog2(MESH_ROWS * TILE_ROWS + 1);
/// TODO: Documentation
pub const MVOUT_COLS_BITS: usize = clog2(max(DMA_MAX_BYTES, MESH_COLS * TILE_COLS) + 1);
/// TODO: Documentation
pub const MVOUT_ROWS_BITS: usize = clog2(MESH_ROWS * TILE_ROWS + 1);

/// Number of load states.
pub const LOAD_STATES: usize = 3;
/// TODO: Documentation
pub const BLOCK_STRIDE_BITS: usize =
    min(16, max(clog2(ACC_BANKS * ACC_BANK_ENTRIES), clog2(SP_BANKS * SP_BANK_ENTRIES)));

/// TODO: Documentation
pub const A_STRIDE_BITS: usize = min(16, max(clog2(ACC_BANKS * ACC_BANK_ENTRIES), clog2(SP_BANKS * SP_BANK_ENTRIES)));
/// TODO: Documentation
pub const C_STRIDE_BITS: usize = min(16, max(clog2(ACC_BANKS * ACC_BANK_ENTRIES), clog2(SP_BANKS * SP_BANK_ENTRIES)));

/// TODO: Documentation
pub const PIXEL_REPEATS_BITS: usize = min(8, clog2(MESH_COLS * TILE_COLS + 1));

/// Number of reservation station entries.
pub const RS_ENTRIES: usize = RS_MAX_PER_TYPE * 3;
/// Number of reorder buffer entries.
pub const ROB_ENTRIES: usize = RS_ENTRIES;

/// Maximum value of `RS_ENTRIES_LD`, `RS_ENTRIES_ST`, `RS_ENTRIES_EX`.
pub const RS_MAX_PER_TYPE: usize = max(RS_ENTRIES_LD, max(RS_ENTRIES_EX, RS_ENTRIES_ST));
/// Bitwidth for representing `RS_MAX_PER_TYPE`.
pub const CL_RS_MAX_PER_TYPE: usize = clog2(RS_MAX_PER_TYPE);

/// Bit width of inputType.
pub const INPUT_BITS: usize = 8;
/// Bit width of the PE register type.
pub const ACC_BITS: usize = 32;
/// Bit width of outputType.
pub const OUTPUT_BITS: usize = 20;
