//! Utility functions used in `mesh_with_delays` module.

use super::mesh::*;
use super::mesh_with_delays::*;
use super::pe::*;
use super::*;

/// Macro rules to apply `shift_reg_fwd` while looping with index.
macro_rules! shift_reg {
    ($first: ident, $( $x:ident ),*) => {{
        [ [$first], $(
                [ $x.shift_reg_fwd::<{ ${index()} + 1 }>() ]
        ), *]
    }};

    (($fx:ident, $fy: ident), $(($x:ident, $y:ident)), *) => {{
        [ [($fx, $fy)], $(
                [ ($x.shift_reg_fwd::<{ ${index()} + 1 }>(), $y.shift_reg_fwd::<{ ${index()} + 1 }>()) ]
        ), *]
    }};
}
macro_rules! shift_reg_rev {
    ($($x:ident),* ; $last:ident) => {{
        [ $( [ $x.shift_reg_fwd::<{ TOTAL_ROWS - 1 - ${index()} }>() ]
        ),*, [ $last ] ]
    }};

    ($(($x:ident, $y:ident)),* ; ($lx:ident, $ly:ident)) => {{
        [ $( [ ($x.shift_reg_fwd::<{ TOTAL_ROWS - 1 - ${index()} }>(), $y.shift_reg_fwd::<{ TOTAL_ROWS - 1 - ${index()} }>()) ]
        ),*, [ ($lx, $ly) ] ]
    }};
}

/// Shift input interface.
pub fn preprocess_shift((in_row, in_col): (MeshRowData, MeshColData)) -> (MeshRowData, MeshColData) {
    let [[r0], [r1], [r2], [r3], [r4], [r5], [r6], [r7], [r8], [r9], [r10], [r11], [r12], [r13], [r14], [r15]] = in_row;
    let [[(c0d, c0c)], [(c1d, c1c)], [(c2d, c2c)], [(c3d, c3c)], [(c4d, c4c)], [(c5d, c5c)], [(c6d, c6c)], [(c7d, c7c)], [(c8d, c8c)], [(c9d, c9c)], [(c10d, c10c)], [(c11d, c11c)], [(c12d, c12c)], [(c13d, c13c)], [(c14d, c14c)], [(c15d, c15c)]] =
        in_col;
    (
        shift_reg!(r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, r13, r14, r15),
        shift_reg!(
            (c0d, c0c),
            (c1d, c1c),
            (c2d, c2c),
            (c3d, c3c),
            (c4d, c4c),
            (c5d, c5c),
            (c6d, c6c),
            (c7d, c7c),
            (c8d, c8c),
            (c9d, c9c),
            (c10d, c10c),
            (c11d, c11c),
            (c12d, c12c),
            (c13d, c13c),
            (c14d, c14c),
            (c15d, c15c)
        ),
    )
}

/// Shift output interface.
pub fn postprocess_shift((out_row, out_col): (MeshRowData, MeshColData)) -> (MeshRowData, MeshColData) {
    let [[r0], [r1], [r2], [r3], [r4], [r5], [r6], [r7], [r8], [r9], [r10], [r11], [r12], [r13], [r14], [r15]] =
        out_row;
    let [[(c0d, c0c)], [(c1d, c1c)], [(c2d, c2c)], [(c3d, c3c)], [(c4d, c4c)], [(c5d, c5c)], [(c6d, c6c)], [(c7d, c7c)], [(c8d, c8c)], [(c9d, c9c)], [(c10d, c10c)], [(c11d, c11c)], [(c12d, c12c)], [(c13d, c13c)], [(c14d, c14c)], [(c15d, c15c)]] =
        out_col;
    (
        shift_reg_rev!(r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, r13, r14; r15),
        shift_reg_rev!(
            (c0d, c0c),
            (c1d, c1c),
            (c2d, c2c),
            (c3d, c3c),
            (c4d, c4c),
            (c5d, c5c),
            (c6d, c6c),
            (c7d, c7c),
            (c8d, c8c),
            (c9d, c9c),
            (c10d, c10c),
            (c11d, c11c),
            (c12d, c12c),
            (c13d, c13c),
            (c14d, c14c);
            (c15d, c15c)
        ),
    )
}

/// Interface type conversion.
#[allow(clippy::type_complexity)]
pub fn preprocess_type((data, req): (Valid<(A, B, D)>, Valid<(ReqExtended, bool)>)) -> (MeshRowData, MeshColData) {
    // # Safety
    //
    // All the input and output interfaces are `Valid` type.
    unsafe {
        (data, req).fsm::<(MeshRowData, MeshColData), ()>((), |(data_in, req_in), _, ()| {
            let default_row = None::<PeRowData>.repeat::<1>().repeat::<MESH_ROWS>();
            let default_col = (None::<PeColData>, None::<PeColControl>).repeat::<1>().repeat::<MESH_COLS>();

            let a_in = data_in.map(|p| p.0);
            let b_in = data_in.map(|p| p.1);
            let d_in = data_in.map(|p| p.2);

            let col_in = match (b_in.zip(d_in), req_in) {
                (Some(bd), Some(req)) => Some((Some(bd), Some(req))),
                (None, Some(req)) => Some((None, Some(req))),
                _ => None,
            };

            let in_left = col_in.map_or(default_row, |_| {
                if let Some(mesh_row) = a_in {
                    mesh_row.map(|tile_row| tile_row.map(|a| Some(PeRowData { a })))
                } else {
                    range::<MESH_ROWS>().map(|_| Some(PeRowData { a: S::from(0.into_u::<INPUT_BITS>()) }).repeat::<1>())
                }
            });

            let in_top = col_in.map_or(default_col, |(bd, mesh_req)| {
                // Reqeust is always valid due to the match statement above.
                let (bd, (ReqExtended { req, config }, last)) = mesh_req.map(|req| (bd, req)).unwrap();
                let pe_control = PeControl { dataflow: req.dataflow, propagate: config.propagate, shift: req.shift };
                let column_control = Some(PeColControl { control: pe_control, id: config.matmul_id, last });

                if let Some((b, d)) = bd {
                    b.zip(d).map(|(b, d)| {
                        let column_data =
                            Some(PeColData { b: b[0].sext::<OUTPUT_BITS>(), d: d[0].sext::<OUTPUT_BITS>() });
                        column_data.repeat::<1>().zip(column_control.repeat::<1>())
                    })
                } else {
                    range::<MESH_COLS>().map(|_| {
                        let column_data = Some(PeColData {
                            b: S::from(0.into_u::<OUTPUT_BITS>()),
                            d: S::from(0.into_u::<OUTPUT_BITS>()),
                        });
                        column_data.repeat::<1>().zip(column_control.repeat::<1>())
                    })
                }
            });

            ((in_left, in_top), ((), ()), ())
        })
    }
}

/// Interface type conversion.
pub fn postprocess_type((out_row, out_col): (MeshRowData, MeshColData)) -> Valid<(C, PeColControl)> {
    // # Safety
    //
    // All the input and output interfaces are `Valid` type.
    unsafe {
        (out_row, out_col).fsm::<Valid<(C, PeColControl)>, ()>((), |(_, col_data), _, ()| {
            let out_valid = col_data[0][0].0.is_some();
            let dataflow_os = col_data[0][0].1.is_some_and(|v| matches!(v.control.dataflow, Dataflow::OS));

            let out_b = col_data
                .map(|tile_r| tile_r.map(|(data, _)| data.map_or(0.into_u(), |v| U::from(v.b))).concat())
                .map(|v| S::from(v).repeat::<TILE_COLS>());
            let out_c = col_data
                .map(|tile_r| tile_r.map(|(data, _)| data.map_or(0.into_u(), |v| U::from(v.d))).concat())
                .map(|v| S::from(v).repeat::<TILE_COLS>());

            let matmul_result = if dataflow_os { out_c } else { out_b };

            let ep = if out_valid { Some((matmul_result, col_data[0][0].1.unwrap())) } else { None };
            let ir0 = ().repeat::<1>().repeat::<MESH_COLS>();
            let ir1 = ((), ()).repeat::<1>().repeat::<MESH_COLS>();

            (ep, (ir0, ir1), ())
        })
    }
}
