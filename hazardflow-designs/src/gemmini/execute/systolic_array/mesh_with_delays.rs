//! Mesh With Delays
//!
//! <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/MeshWithDelays.scala>
#![allow(unused)]

use systolic_array::transposer::*;

use super::mesh::*;
use super::*;

/// Max simultaneous matrix multiplications.
pub const MAX_SIMULTANEOUS_MATMULS: usize = 5;
/// Matmul id bits.
pub const ID_BITS: usize = clog2(MAX_SIMULTANEOUS_MATMULS);

const TOTAL_ROWS: usize = BLOCK_SIZE;

const FIFO_LENGTH: usize = MAX_SIMULTANEOUS_MATMULS + 1;

/// Type of data of `a`.
pub type A = Array<Array<U<INPUT_BITS>, TILE_ROWS>, MESH_ROWS>;
/// Type of data of `b`.
pub type B = Array<Array<U<INPUT_BITS>, TILE_COLS>, MESH_COLS>;
/// Type of data of `d`.
pub type D = Array<Array<U<INPUT_BITS>, TILE_COLS>, MESH_COLS>;

/// Resolver.
pub type TagsInProgress = Array<MeshTag, FIFO_LENGTH>;

/// Macro rules to apply `shift_reg_fwd` while looping with index.
macro_rules! shift_reg {
    ($first: ident, $( $x:ident ),*) => {{
        [ [$first], $(
                [ $x.shift_reg_fwd::<{${index()} + 1}>() ]
        ), *]
    }};

    (($fx:ident, $fy: ident), $(($x:ident, $y:ident)), *) => {{
        [ [($fx, $fy)], $(
                [ ($x.shift_reg_fwd::<{${index()} + 1}>(), $y.shift_reg_fwd::<{${index()} + 1}>()) ]
        ), *]
    }};
}
macro_rules! shift_reg_reverse {
    ($($x:ident),* ; $last:ident) => {{
        [ $( [ $x.shift_reg_fwd::<{TOTAL_ROWS - 1 - ${index()}}>() ]
        ),*, [ $last ] ]
    }};

    ($(($x:ident, $y:ident)),* ; ($lx:ident, $ly:ident)) => {{
        [ $( [ ($x.shift_reg_fwd::<{TOTAL_ROWS - 1 - ${index()}}>(), $y.shift_reg_fwd::<{TOTAL_ROWS - 1 - ${index()}}>()) ]
        ),*, [ ($lx, $ly) ] ]
    }};
}

/// Flag for determining which matrix comes out from transposer.
#[derive(Debug, Clone, Copy)]
pub enum TransposeFlag {
    /// A is transposed
    A,
    /// B is transposed
    B,
    /// D is transposed
    D,
}

/// Mesh tag
#[derive(Debug, Clone, Copy)]
pub struct MeshTag {
    /// rob_id
    pub rob_id: HOption<U<{ clog2(RS_ENTRIES) }>>,
    /// local_addr
    pub addr: LocalAddr,
    /// rows
    pub rows: U<{ clog2(BLOCK_SIZE + 1) }>,
    /// cols
    pub cols: U<{ clog2(BLOCK_SIZE + 1) }>,
}

impl MeshTag {
    /// Generate garbage tag.
    pub fn get_garbage_tag() -> Self {
        let garbage_addr = LocalAddr::from(GARBAGE_ADDR.into_u());
        Self { rob_id: None, addr: garbage_addr, rows: 0.into_u(), cols: 0.into_u() }
    }
}

/// Request signals to the mesh.
#[derive(Debug, Clone, Copy)]
pub struct MeshReq {
    /// pe_control
    pub pe_control: PeControl,
    /// a_transpose
    pub transpose_a: bool,
    /// bd_transpos
    pub transpose_bd: bool,
    /// total_rows
    pub total_rows: U<{ clog2(BLOCK_SIZE + 1) }>,
    /// tag
    pub tag: MeshTag,
    /// flush
    pub flush: U<2>,
}

/// Response signals from the mesh.
#[derive(Debug, Clone, Copy)]
pub struct MeshResp {
    /// total_rows
    pub total_rows: U<{ clog2(BLOCK_SIZE + 1) }>,
    /// tag
    pub tag: MeshTag,
    /// last
    pub last: bool,
    /// data
    pub data: Array<U<OUTPUT_BITS>, MESH_COLS>,
}

/// Helper type to update configurations.
#[derive(Debug, Default, Clone, Copy)]
struct Config {
    matmul_id: U<ID_BITS>,
    in_prop: bool,
}

/// Helper type to manage fire_counter and flush_counter.
#[derive(Debug, Default, Clone, Copy)]
struct Counter {
    fire_counter: U<{ clog2(BLOCK_SIZE) }>,
    flush: U<2>,
}

/// Helper function to update configurations.
fn update_config(req: MeshReq, config: Config) -> Config {
    todo!("Assignment 6")
}

/// Helper funtion to manage fire counter.
///
/// This function increases counter whenever all data(a, b, and d) comes in.
#[allow(clippy::type_complexity)]
fn update_fire_counter(
    (req, config): (MeshReq, Config),
    counter: Counter,
) -> (((MeshReq, Config, bool), BoundedU<2>), Counter, bool) {
    todo!("Assignment 6")
}

/// Shift input interface.
fn shift_i((in_left, in_top): (MeshRowData, MeshColData)) -> (MeshRowData, MeshColData) {
    todo!("Assignment 6")
}

/// Shift output interface.
fn shift_o((row_output, col_output): (MeshRowData, MeshColData)) -> (MeshRowData, MeshColData) {
    todo!("Assignment 6")
}

/// Input interface type conversion.
#[allow(clippy::type_complexity)]
fn mesh_i(
    (a, b, d, req): (Valid<A>, Valid<B>, Valid<D>, Valid<(MeshReq, Config, bool)>),
) -> (MeshRowData, MeshColData) {
    // # Safety
    //
    // All the input and output interfaces are `Valid` type.
    unsafe {
        (a, b, d, req).fsm::<(MeshRowData, MeshColData), ()>((), |(a_in, b_in, d_in, req_in), _, ()| {
            //
            todo!("Assignment 6")
        })
    }
}

/// Output interface type conversion.
fn mesh_o(
    (row_output, col_output): (MeshRowData, MeshColData),
) -> (Valid<Array<U<OUTPUT_BITS>, MESH_COLS>>, Valid<PeColControl>) {
    // # Safety
    //
    // All the input and output interfaces are `Valid` type.
    unsafe {
        (row_output, col_output).fsm::<(Valid<Array<U<OUTPUT_BITS>, MESH_COLS>>, Valid<PeColControl>), ()>(
            (),
            |(_, col_data), _, ()| {
                //
                todo!("Assignment 6")
            },
        )
    }
}

/// Helper function to manage tag fifo and total_rows fifo.
fn fifos(
    req: I<VrH<(MeshReq, Config), TagsInProgress>, { Dep::Helpful }>,
    id_and_last: Valid<(U<ID_BITS>, bool)>,
) -> Valid<(MeshTag, U<{ clog2(BLOCK_SIZE + 1) }>)> {
    todo!("Assignment 6")
}

/// This moudle is in charge of synchronizing inputs, managing metadata regarding where and how to store the results at SRAM,
/// and producing informations for pe.rs (e.g, last, id, propagate, etc.)
/// You shouldn't change function signature.
pub fn mwd_inner(
    a: Vr<A>,
    b: Vr<B>,
    d: Vr<D>,
    req: I<VrH<MeshReq, TagsInProgress>, { Dep::Helpful }>,
) -> Valid<MeshResp> {
    todo!("Assignment 6")
}

/// Debug
#[synthesize]
pub fn mwd(a: Vr<A>, b: Vr<B>, d: Vr<D>, req: I<VrH<MeshReq, TagsInProgress>, { Dep::Helpful }>) -> Valid<MeshResp> {
    mwd_inner(a, b, d, req)
}
