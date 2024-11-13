//! Mesh with delays.

#![allow(unused)] // Added for assignment.
#![allow(warnings)] // Added for assignment.

use super::utils::*;
use super::*;

/// Max simultaneous matrix multiplications.
pub const MAX_SIMULTANEOUS_MATMULS: usize = 5;
/// Matmul id bits.
pub const ID_BITS: usize = clog2(MAX_SIMULTANEOUS_MATMULS);
/// Total rows.
pub const TOTAL_ROWS: usize = BLOCK_SIZE;

const FIFO_LENGTH: usize = MAX_SIMULTANEOUS_MATMULS + 1;

/// Type of data of `a`.
pub type A = Array<Array<S<INPUT_BITS>, TILE_ROWS>, MESH_ROWS>;
/// Type of data of `b`.
pub type B = Array<Array<S<INPUT_BITS>, TILE_COLS>, MESH_COLS>;
/// Type of data of `d`.
pub type D = Array<Array<S<INPUT_BITS>, TILE_COLS>, MESH_COLS>;

/// Resolver signal.
///
/// Represents active computations in the Mesh.
pub type TagsInProgress = Array<MeshTag, FIFO_LENGTH>;

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
    pub data: Array<S<OUTPUT_BITS>, MESH_COLS>,
}

/// Helper type to update configurations.
#[derive(Debug, Default, Clone, Copy)]
pub struct Config {
    /// Matmul ID.
    pub matmul_id: U<ID_BITS>,
    /// Propagation.
    pub in_prop: bool,
}

/// Helper type to manage fire_counter and flush_counter.
#[derive(Debug, Default, Clone, Copy)]
struct Counter {
    fire_counter: U<{ clog2(BLOCK_SIZE) }>,
    flush_counter: U<2>,
}

#[derive(Debug, Clone, Copy)]
struct ReqExtended {
    req: MeshReq,
    config: Config,
}

/// Helper function
///
/// Updates the global configuration of `mesh_delay`.
/// This function should be used in a combinator that manages internal state.
///
/// # Arguments
///
/// * `prop` - Propagation information from `MeshReq`.
/// * `config` - Previously stored configuration state.
///
/// # Returns
///
/// New configuration state.
///
/// # Behavior
///
/// For more details, refer to section 2.3.1 of the assignment description.
fn update_config(prop: Propagate, config: Config) -> Config {
    todo!("assignment 6")
}

/// Helper function
///
/// Increase fire counter whenever all data (a, b, and d) comes in.
/// This function should be used in a `fsm_egress`.
///
/// # Arguments
///
/// * `req_ext` - Request and updated configuration.
/// * `counter` - Previously stored conter state.
///
/// # Returns
///
/// * `(ReqExtended, bool)` - request and 1-bit signal indicating last fire.
/// * `Counter` - Updated counter state.
/// * `bool` - The 1-bit signal indicating whether `fsm_egress` is ready to receieve new request or not.
///
/// # Behavior
///
/// For more details, refer to section 2.3.2 of the assignment description.
#[allow(clippy::type_complexity)]
fn update_counter(req_ext: ReqExtended, counter: Counter) -> ((ReqExtended, bool), Counter, bool) {
    let req = req_ext.req;

    let last_fire: bool = todo!("Is this last fire?");
    let s_next: Counter = todo!("Calculate next fire_counter and flush_counter");

    // `is_last` indicates fsm_egress is ready to receive next payload.
    let is_last: bool = todo!("is_last for WS dataflow") || todo!("is_last for OS dataflow");

    ((req_ext, last_fire), s_next, is_last)
}

/// Helper function to manage two fifo.
///
/// Manage `tag_fifo` and `total_rows_fifo`.
/// The `tag` and `total_rows` are metadata to store results in SRAM (Scratchpad or Accumulator).
///
/// # Arguments
///
/// * `req` - Request which contains `tag` and `total_rows`.
/// Resolver signal which contains active computation in systolic array.
/// * `control` - control signal which contains `mesh_id` and `last` signals.
///
/// # Returns
///
/// * `MeshTag` - Tag for the just finished computation in systolic array.
/// *  `U<{ clog2(BLOCK_SIZE + 1) }>` - Total rows for the just finished computation in systolic array.
///
/// # Behavior
///
/// For more details, refer to section 2.3.6 of the assignment description.
fn fifos(
    req: I<VrH<ReqExtended, TagsInProgress>, { Dep::Helpful }>,
    control: Valid<PeColControl>,
) -> Valid<(MeshTag, U<{ clog2(BLOCK_SIZE + 1) }>)> {
    // Duplicate control signal and request, because we need to address two fifo.
    let (control_tag, control_row) = control.lfork();

    // Refer to section 2.3.8 (1)
    let req: I<VrH<ReqExtended, TagsInProgress>, { Dep::Helpful }> = todo!("filter flush operation");
    let (req_tagq, req_rowq) = req.map_resolver_inner::<(TagsInProgress, ())>(|(tags, _)| tags).lfork();

    // Refer to section 2.3.8 (2)
    // Calculate future `matmul_id` when the computation is completed.
    let req_tag: I<VrH<(U<ID_BITS>, ReqExtended), TagsInProgress>, { Dep::Helpful }> =
        todo!("Calculate matmul_id_of_output");
    let req_row: Vr<(U<ID_BITS>, ReqExtended)> = todo!("Calculate matmul id of current");

    // Refer to section 2.3.8 (3)
    // Convert the resolver type to use `fifo` family combinator
    let req_tag: I<VrH<(U<ID_BITS>, MeshTag), ((), FifoS<(U<ID_BITS>, MeshTag), FIFO_LENGTH>)>, { Dep::Helpful }> =
        todo!("Caculate the resolver signal `TagsInProgress` here");
    let req_row = todo!("Refer `req_tag` and convert resolver type");

    // FIFO
    let tag_fifo = req_tag.multi_headed_transparent_fifo();
    let row_fifo = todo!("Use this -> req_row.multi_headed_transparent_fifo();");

    // Refer to the section 2.3.8 (4)
    // We need PeColControl signal.
    let tag = (tag_fifo, control_tag).join();
    let total_rows = todo!("Use this -> (row_fifo, control_row).join();");

    let tag: Valid<MeshTag> = todo!("Carefully read condition for popping and transferring data.");
    let total_rows: Valid<U<5>> = todo!("Carefully read condition for popping and transferring data.");

    // Refer to the section 2.3.8 (5)
    // Return metadata
    (tag, total_rows).zip_any_valid().map(|(tag, total_rows)| {
        let tag: MeshTag = todo!("If tag is invalid payload, replace it with garbage tag signal");
        let total_rows: U<5> = todo!("If total_rows in invalid payload, replace it with garbage total_rows signal");

        (tag, total_rows)
    })
}

/// Helper function to invoke transposer.
///
/// Transpose 0 to 1 matrix.
///
/// # Arguments
///
/// * `data` - It contains request and 3 matrices. The request contains `dataflow`, `transpose_a` and `transpose_bd`.
///
/// # Returns
///
///  Three matrices. Either one is transposed, or none are transposed.
///
/// # Behavior
///
/// For more details, refer to section 2.3.5 of the assignment description.
/// The figure would be pretty helpful!
fn transpose(data: Valid<(MeshReq, A, B, D)>) -> (Valid<A>, Valid<B>, Valid<D>) {
    // You need to attach selector bit to use `branch` combinator later.
    // Selector bit for whether the matrix should be tranposed or not.
    // Refer to section 2.3.5 (1).
    let a_with_sel: Valid<(A, BoundedU<2>)> = todo!("attach selector bit for A matrix");
    let b_with_sel: Valid<(B, BoundedU<2>)> = todo!("attach selector bit for B matrix");
    let d_with_sel: Valid<(D, BoundedU<2>)> = todo!("attach selector bit for D matrix");

    // Perform branch based on selector bit.
    // Refer to section 2.3.5 (2)
    let [a, a_transpose]: [Valid<A>; 2] = todo!("branch from a_with_sel");
    let [b, b_transpose]: [Valid<B>; 2] = todo!("branch from b_with_sel");
    let [d, d_transpose]: [Valid<D>; 2] = todo!("branch from d_with_sel");

    // Attach tag where transposed data come from.
    // Refer to section 2.3.5 (3)
    let a_transpose: Valid<(TransposeFlag, A)> = todo!("attach tag");
    let b_transpose: Valid<(TransposeFlag, B)> = todo!("attach tag");
    let d_transpose: Valid<(TransposeFlag, D)> =
        todo!("attach tag. It should be reversed before going into transposer!");

    // Get transpose_target.
    // section 2.3.5 (4)
    let flag: Valid<TransposeFlag> = todo!("Which matrices among A, B, and D go into the Transposer?");
    // Valid<A> doesn't mean that A should be tranposed. Actually the type of A, B, and D are the same.
    let transpose_target: Valid<A> = todo!("Select matrix among a, b, and d");

    let transposed = transpose_target.map(|vec| vec.concat()).comb(transposer_ffi);

    // Identify which matrix is transposed among A, B, or D.
    // section 2.3.5 (5)
    let [a_transposed, b_transposed, d_transposed]: [Valid<A>; 3] =
        (flag, transposed).join_valid().map(|(flag, arr)| todo!("which matrix?")).branch();

    // Select one among `a` and `a_transposed`
    // Section 2.3.5 (6)
    let a: Valid<A> = todo!("Select one among `a` and `a_transposed`");
    let b: Valid<B> = todo!("Select one among `b` and `b_transposed`");
    let d: Valid<D> = todo!("Select one among `d` and `d_transposed`. It should be reversed before going out.");

    (a, b, d)
}

/// Mesh with delays.
pub fn mesh_with_delays(
    data: Vr<Array<A, 3>>,
    req: I<VrH<MeshReq, TagsInProgress>, { Dep::Helpful }>,
) -> Valid<MeshResp> {
    // Section 2.3.1 update configurations.
    // Update configuration and fork interfaces.
    let req: I<VrH<ReqExtended, TagsInProgress>, { Dep::Helpful }> = todo!("update configuration. from req");

    let (mesh_req, fifo_req) = req.map_resolver_inner::<((), TagsInProgress)>(|(_, tags)| tags).lfork();

    // Section 2.3.2 manage request buffer
    let mesh_req: Vr<(ReqExtended, bool)> = todo!("Request buffer. from mesh_req");

    // Section 2.3.3 perform a branch with either flush_req or matmul_req
    // It could be (flush_req, matmul_req)
    let [flush_req, matmul_req]: [Vr<(ReqExtended, bool)>; 2] = todo!("Branch request. from mesh_req");

    let (matmul_req, req_and_data) = (matmul_req, data).join_vr().always_into_valid().lfork();

    // Section 2.3.4 merge `flush_req` and `matmul_req`.
    // Only one request between `flush_req` and `matmul_req` can be transferred to mesh.
    let matmul_req = matmul_req.map(|(req, _)| req);
    let flush_req = flush_req.always_into_valid();
    let req: Valid<(MeshReq, Config, bool)> = todo!("Merged Req");

    // Section 2.3.5 Invoke transposer.
    let (a, b, d): (Valid<A>, Valid<B>, Valid<D>) =
        todo!("Use matmul_req_and_data. Get these Valid interfaces from `transpose` helper function.");

    // Preprocessing for mesh input such as,
    // Applying ShiftRegister and interface type conversion
    let (in_left, in_top) = (a, b, d, req).comb(mesh_i).comb(shift_i);

    let mesh_output = mesh_ffi(in_left, in_top);

    // Preprocessing for mesh output such as,
    // Applying ShiftRegister and interface type conversion
    let (output_data, output_config) = mesh_output.comb(shift_o).comb(mesh_o);
    let (fifo_control, last) = output_config.lfork();

    // FIFO
    let metadata = fifos(fifo_req, fifo_control);

    let output = (output_data, last.map(|p| p.last)).join_valid();

    // Refer to the section 2.3.9
    // Return `MeshResp`
    todo!("Use `metadata` and `output.`")
}

/// Mesh with delays.
#[synthesize]
pub fn mesh_with_delays_default(
    a: Vr<A>,
    b: Vr<B>,
    d: Vr<D>,
    req: I<VrH<MeshReq, TagsInProgress>, { Dep::Helpful }>,
) -> Valid<MeshResp> {
    let data = [a, b, d].join_vr();
    mesh_with_delays(data, req)
}
