//! Utility functions used in `mesh_with_delays` module.

use super::mesh::*;
use super::mesh_with_delays::*;
use super::pe::*;
use super::*;

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

/// Shift input interface.
pub fn shift_i((in_left, in_top): (MeshRowData, MeshColData)) -> (MeshRowData, MeshColData) {
    let [[in_left0], [in_left1], [in_left2], [in_left3], [in_left4], [in_left5], [in_left6], [in_left7], [in_left8], [in_left9], [in_left10], [in_left11], [in_left12], [in_left13], [in_left14], [in_left15]] =
        in_left;
    let [[(t0d, t0c)], [(t1d, t1c)], [(t2d, t2c)], [(t3d, t3c)], [(t4d, t4c)], [(t5d, t5c)], [(t6d, t6c)], [(t7d, t7c)], [(t8d, t8c)], [(t9d, t9c)], [(t10d, t10c)], [(t11d, t11c)], [(t12d, t12c)], [(t13d, t13c)], [(t14d, t14c)], [(t15d, t15c)]] =
        in_top;
    (
        shift_reg!(
            in_left0, in_left1, in_left2, in_left3, in_left4, in_left5, in_left6, in_left7, in_left8, in_left9,
            in_left10, in_left11, in_left12, in_left13, in_left14, in_left15
        ),
        shift_reg!(
            (t0d, t0c),
            (t1d, t1c),
            (t2d, t2c),
            (t3d, t3c),
            (t4d, t4c),
            (t5d, t5c),
            (t6d, t6c),
            (t7d, t7c),
            (t8d, t8c),
            (t9d, t9c),
            (t10d, t10c),
            (t11d, t11c),
            (t12d, t12c),
            (t13d, t13c),
            (t14d, t14c),
            (t15d, t15c)
        ),
    )
}

/// Shift output interface.
pub fn shift_o((row_output, col_output): (MeshRowData, MeshColData)) -> (MeshRowData, MeshColData) {
    let [[row0], [row1], [row2], [row3], [row4], [row5], [row6], [row7], [row8], [row9], [row10], [row11], [row12], [row13], [row14], [row15]] =
        row_output;
    let [[(c0d, c0c)], [(c1d, c1c)], [(c2d, c2c)], [(c3d, c3c)], [(c4d, c4c)], [(c5d, c5c)], [(c6d, c6c)], [(c7d, c7c)], [(c8d, c8c)], [(c9d, c9c)], [(c10d, c10c)], [(c11d, c11c)], [(c12d, c12c)], [(c13d, c13c)], [(c14d, c14c)], [(c15d, c15c)]] =
        col_output;
    (
        shift_reg_reverse!(
            row0, row1, row2, row3, row4, row5, row6, row7, row8, row9, row10, row11, row12, row13, row14; row15
        ),
        shift_reg_reverse!(
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
pub fn mesh_i(
    (a, b, d, req): (Valid<A>, Valid<B>, Valid<D>, Valid<(MeshReq, Config, bool)>),
) -> (MeshRowData, MeshColData) {
    // # Safety
    //
    // All the input and output interfaces are `Valid` type.
    unsafe {
        (a, b, d, req).fsm::<(MeshRowData, MeshColData), ()>((), |(a_in, b_in, d_in, req_in), _, ()| {
            let default_row = None::<PeRowData>.repeat::<1>().repeat::<MESH_ROWS>();
            let default_col = (None::<PeColData>, None::<PeColControl>).repeat::<1>().repeat::<MESH_COLS>();

            let col_in = match (b_in.zip(d_in), req_in) {
                (Some(bd), Some(req)) => Some((Some(bd), Some(req))),
                (None, Some(req)) => Some((None, Some(req))),
                _ => None,
            };

            let in_left = col_in.map_or(default_row, |_| {
                if let Some(a_in) = a_in {
                    a_in.map(|tile_v| tile_v.map(|v| Some(PeRowData { a: v })))
                } else {
                    range::<MESH_ROWS>().map(|_| Some(PeRowData { a: S::from(0.into_u::<INPUT_BITS>()) }).repeat::<1>())
                }
            });

            let in_top = col_in.map_or(default_col, |(bd, mesh_req)| {
                // Reqeust is always valid due to the match statement above.
                let (bd, (req, config, last_fire)) = mesh_req.map(|req| (bd, req)).unwrap();
                let pe_control = PeControl {
                    dataflow: req.pe_control.dataflow,
                    propagate: if config.in_prop { Propagate::Reg1 } else { Propagate::Reg2 },
                    shift: req.pe_control.shift,
                };
                let column_control = Some(PeColControl { control: pe_control, id: config.matmul_id, last: last_fire });

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

            ((in_left, in_top), ((), (), (), ()), ())
        })
    }
}

/// Interface type conversion.
pub fn mesh_o(
    (row_output, col_output): (MeshRowData, MeshColData),
) -> (Valid<Array<S<OUTPUT_BITS>, MESH_COLS>>, Valid<PeColControl>) {
    // # Safety
    //
    // All the input and output interfaces are `Valid` type.
    unsafe {
        (row_output, col_output).fsm::<(Valid<Array<S<OUTPUT_BITS>, MESH_COLS>>, Valid<PeColControl>), ()>(
            (),
            |(_, col_data), _, ()| {
                let out_valid = col_data[0][0].0.is_some();
                let dataflow_os = col_data[0][0].1.is_some_and(|v| matches!(v.control.dataflow, Dataflow::OS));

                let out_b = col_data
                    .map(|tile_r| tile_r.map(|(data, _)| data.map_or(0.into_u(), |v| U::from(v.b))).concat())
                    .map(S::from);
                let out_c = col_data
                    .map(|tile_r| tile_r.map(|(data, _)| data.map_or(0.into_u(), |v| U::from(v.d))).concat())
                    .map(S::from);

                let matmul_result = if dataflow_os { out_c } else { out_b };

                let matmul_result = if out_valid { Some(matmul_result) } else { None };
                let output_control = if out_valid { col_data[0][0].1 } else { None };

                let ir0 = ().repeat::<1>().repeat::<MESH_COLS>();
                let ir1 = ((), ()).repeat::<1>().repeat::<MESH_COLS>();
                ((matmul_result, output_control), (ir0, ir1), ())
            },
        )
    }
}
