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
/// Type of data of `c`.
pub type C = Array<Array<S<OUTPUT_BITS>, TILE_COLS>, MESH_COLS>;
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
    /// ROB ID.
    pub rob_id: HOption<U<{ clog2(RS_ENTRIES) }>>,
    /// SRAM write address.
    pub addr: LocalAddr,
    /// Number of rows.
    pub rows: U<{ clog2(BLOCK_SIZE + 1) }>,
    /// Number of cols.
    pub cols: U<{ clog2(BLOCK_SIZE + 1) }>,
}

impl Default for MeshTag {
    /// Returns garbage tag.
    fn default() -> Self {
        Self { rob_id: None, addr: LocalAddr::from(GARBAGE_ADDR.into_u()), rows: 0.into_u(), cols: 0.into_u() }
    }
}

/// Request signals to the mesh.
#[derive(Debug, Clone, Copy)]
pub struct MeshReq {
    /// Dataflow value used in the PE.
    pub dataflow: Dataflow,
    /// Indicates whether the propagate value should be flipped.
    pub propagate_flip: bool,
    /// Shift value used in the PE.
    pub shift: U<{ clog2(ACC_BITS) }>,
    /// Indicates that `A` should be transposed, used to invoke a transposer.
    pub transpose_a: bool,
    /// Indicates that either `B` or `D` should be transposed, used to invoke a transposer.
    pub transpose_bd: bool,
    /// Specifies the number of rows in the matmul operation.
    pub total_rows: U<{ clog2(BLOCK_SIZE + 1) }>,
    /// Tag.
    pub tag: MeshTag,
    /// Indicates whether the request represents a flush.
    pub flush: bool,
}

/// Response signals from the mesh.
#[derive(Debug, Clone, Copy)]
pub struct MeshResp {
    /// Specifies the number of rows in the matmul operation.
    pub total_rows: U<{ clog2(BLOCK_SIZE + 1) }>,
    /// Tag.
    pub tag: MeshTag,
    /// Indicates that the row represents the last row.
    pub last: bool,
    /// Output data.
    pub data: C,
}

/// Matmul operation configuration.
#[derive(Debug, Default, Clone, Copy)]
pub struct Config {
    /// Matmul ID.
    pub matmul_id: U<ID_BITS>,
    /// Propagation.
    pub propagate: Propagate,
}

impl Config {
    /// Creates a new configuration.
    pub fn new(matmul_id: U<ID_BITS>, propagate: Propagate) -> Self {
        Self { matmul_id, propagate }
    }

    /// Returns the updated global configuration based on the incoming request.
    ///
    /// For more details, see Section 2.3.1 of the assignment documentation.
    ///
    /// # Arguments
    ///
    /// - `self`: The current configuration state.
    /// - `propagate_flip`: A boolean indicating whether to toggle the propagate value in processing elements (PEs).
    pub fn update(self, propagate_flip: bool) -> Self {
        todo!("assignment 6")
    }
}

/// Wrapper type of request and configuration.
#[derive(Debug, Clone, Copy)]
pub struct ReqExtended {
    /// Mesh request.
    pub req: MeshReq,
    /// Matmul operation configuration.
    pub config: Config,
}

/// Manages two FIFOs containing the mesh tag and total rows, and returns the metadata transferred from the FIFO.
///
/// These metadata are used to store the Mesh output in the SRAM (Scratchpad + Accumulator).
///
/// For more details, see Section 2.3.8 of the assignment documentation.
///
/// # Arguments
///
/// - `req`: A request containing metadata. It sends the tags in the `tags_fifo` as the resolver.
/// - `control`: Control signals from the Mesh output.
fn fifos(
    req: I<VrH<ReqExtended, TagsInProgress>, { Dep::Helpful }>,
    control: Valid<PeColControl>,
) -> (Valid<MeshTag>, Valid<U<{ clog2(BLOCK_SIZE + 1) }>>) {
    // Duplicate control signal and request, because we need to address two fifo.
    let (control_to_tag_fifo, control_to_total_rows_fifo) = control.lfork();

    // Section 2.3.8 (1) Filter out flush request.
    let req: I<VrH<ReqExtended, TagsInProgress>, { Dep::Helpful }> = todo!("assignment 6");

    // Section 2.3.8 (2) Calculate future `matmul_id`.
    let (tag, total_rows) = req
        .map_resolver_inner::<(TagsInProgress, ())>(|(tags, _)| tags)
        .map(|ReqExtended { req, config }| {
            let tag_id = todo!("assignment 6");
            let total_rows_id = todo!("assignment 6");
            ((tag_id, req.tag), (total_rows_id, req.total_rows))
        })
        .unzip();

    // Section 2.3.8 (3) Convert resolver type and calculate `TagsInProgress`.
    let tag: I<VrH<(U<ID_BITS>, MeshTag), ((), FifoS<(U<ID_BITS>, MeshTag), FIFO_LENGTH>)>, { Dep::Helpful }> =
        todo!("Caculate the resolver signal `TagsInProgress` here");
    let total_rows = total_rows.map_resolver_inner::<((), FifoS<(U<ID_BITS>, U<5>), FIFO_LENGTH>)>(|_| ());

    // FIFO
    let tag_fifo = tag.multi_headed_transparent_fifo().filter_map(|p| p.head());
    let total_rows_fifo = total_rows.multi_headed_transparent_fifo().filter_map(|p| p.head());

    // Section 2.3.8 (4) Pop one element and get metadata.
    let tag = (tag_fifo, control_to_tag_fifo)
        .join()
        .map_resolver_inner_with_p::<()>(|ip, _| {
            let pop: bool = todo!("assignment 6");
            ((), if pop { 1.into_u() } else { 0.into_u() })
        })
        .filter_map::<MeshTag>(|((head_id, tag), mesh_out_control)| {
            let transfer: bool = todo!("assignment 6");
            if transfer {
                Some(tag)
            } else {
                None
            }
        });
    let total_rows = (total_rows_fifo, control_to_total_rows_fifo)
        .join()
        .map_resolver_inner_with_p::<()>(|ip, _| {
            let pop: bool = todo!("assignment 6");
            ((), if pop { 1.into_u() } else { 0.into_u() })
        })
        .filter_map::<U<{ clog2(BLOCK_SIZE + 1) }>>(|((head_id, total_rows), mesh_out_control)| {
            let transfer: bool = todo!("assignment 6");
            if transfer {
                Some(total_rows)
            } else {
                None
            }
        });

    // NOTE: Converting to the valid interface is safe as there are no longer any hazards.
    (tag.always_into_valid(), total_rows.always_into_valid())
}

/// Invokes a Transposer.
///
/// Returns three matrices, with at most one matrix is transposed.
///
/// For more details, see Section 2.3.5 of the assignment documentation.
///
/// # Arguments
///
/// - `data`: It contains request and 3 matrices. The request contains `dataflow`, `transpose_a` and `transpose_bd`.
fn transpose(data: Valid<(MeshReq, A, B, D)>) -> Valid<(A, B, D)> {
    // Section 2.3.5 (1) Attach selector.
    let (a_with_sel, b_with_sel, d_with_sel): (
        Valid<(A, BoundedU<2>)>,
        Valid<(B, BoundedU<2>)>,
        Valid<(D, BoundedU<2>)>,
    ) = todo!("assignment 6");

    // Section 2.3.5 (2) Branch interface.
    let [a, a_transpose] = a_with_sel.branch();
    let [b, b_transpose] = b_with_sel.branch();
    let [d, d_transpose] = d_with_sel.branch();

    let a_transpose = a_transpose.map(|p| (TransposeFlag::A, p));
    let b_transpose = b_transpose.map(|p| (TransposeFlag::B, p));
    let d_transpose = d_transpose.map(|p| (TransposeFlag::D, p.reverse()));

    // Section 2.3.5 (3) Select transpose target.
    // NOTE: `Valid<A>` does not mean that `A` should be transposed, actually the types `A`, `B`, and `D` are the same.
    let (flag, transpose_target): (Valid<TransposeFlag>, Valid<A>) = todo!("assignment 6");

    let transposed = transpose_target.map(|vec| vec.concat()).comb(transposer_ffi);

    // Section 2.3.5 (4) Identify which matrix is transposed among A, B, or D.
    let [a_transposed, b_transposed, d_transposed]: [Valid<A>; 3] =
        (flag, transposed).join_valid().map(|(flag, arr)| todo!("assignment 6")).branch();

    // Section 2.3.5 (5) Select one among `X` and `X_transposed`.
    let a = [a_transposed, a].merge();
    let b = [b_transposed, b].merge();
    let d = [d_transposed.map(|p| p.reverse()), d].merge();

    (a, b, d).join_valid()
}

/// Mesh with delays.
pub fn mesh_with_delays(
    data: Vr<(A, B, D)>,
    req: I<VrH<MeshReq, TagsInProgress>, { Dep::Helpful }>,
) -> Valid<MeshResp> {
    // Section 2.3.1 Update Configurations.
    let req: I<VrH<ReqExtended, TagsInProgress>, { Dep::Helpful }> = todo!("assignment 6");

    let (mesh_req, fifo_req) = req.map_resolver_inner::<((), TagsInProgress)>(|(_, tags)| tags).lfork();

    // Section 2.3.2 Request Buffer.
    let mesh_req: Vr<(ReqExtended, bool)> = mesh_req.fsm_egress::<(ReqExtended, bool), U<{ clog2(BLOCK_SIZE) }>>(
        U::default(),
        true,
        true,
        |req_ext, counter| todo!("assignment 6"),
    );

    // Section 2.3.3 Branch Request.
    let [mesh_req_flush, mesh_req_matmul]: [Vr<(ReqExtended, bool)>; 2] = todo!("assignment 6");

    // NOTE: Converting to the valid interface is safe as there are no longer any hazards.
    let mesh_req_flush = mesh_req_flush.always_into_valid();
    let (mesh_req_matmul, mesh_data) = (mesh_req_matmul, data)
        .join_vr()
        .always_into_valid()
        .map(|((req_ext, last), (a, b, d))| {
            let matmul_req = (req_ext, last);
            let mesh_data = (req_ext.req, a, b, d);
            (matmul_req, mesh_data)
        })
        .unzip();

    // Section 2.3.4 Merging Requests.
    let mesh_req: Valid<(ReqExtended, bool)> = todo!("assignment 6");

    // Section 2.3.5 Invoke a Transposer.
    let mesh_data_transposed = mesh_data.comb(transpose);

    // Section 2.3.6 Mesh IO type conversion + Section 2.3.7 Shift.
    let mesh_out = (mesh_data_transposed, mesh_req)
        .comb(preprocess_type)
        .comb(preprocess_shift)
        .comb(move |(in_row, in_col)| mesh_ffi(in_row, in_col))
        .comb(postprocess_shift)
        .comb(postprocess_type);

    let (mesh_out, mesh_out_control) = mesh_out.map(|p| (p, p.1)).unzip();

    // Section 2.3.8 FIFO.
    let (tag, total_rows) = fifos(fifo_req, mesh_out_control);

    // Section 2.3.9 Return Mesh Response.
    (tag, total_rows, mesh_out).zip_any_valid().filter_map(|(tag, total_rows, mesh_out)| todo!("assignment 6"))
}

/// Mesh with delays.
#[synthesize]
pub fn mesh_with_delays_default(
    a: Vr<A>,
    b: Vr<B>,
    d: Vr<D>,
    req: I<VrH<MeshReq, TagsInProgress>, { Dep::Helpful }>,
) -> Valid<MeshResp> {
    let data = (a, b, d).join_vr();
    mesh_with_delays(data, req)
}
