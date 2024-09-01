//! Tranpose preload unroller.
// TODO: Below implementation is inefficient in throughput-wise. Need to implement `pipe` option for `fsm_ingress` and `fsm_egress`.

use super::*;
use crate::gemmini::isa::*;

fn update_config(cmd: GemminiCmd, b_transposed_and_ws: bool) -> bool {
    let config_cmd_type = ConfigCmd::from(cmd.cmd.rs1.clip_const::<2>(0));
    let is_config = matches!(cmd.cmd.inst.funct, Funct::ConfigCmd) && matches!(config_cmd_type, ConfigCmd::Ex);

    if is_config {
        let set_only_strides = cmd.cmd.rs1[7];

        if !set_only_strides {
            // TODO: Add condition `dataflow == Dataflow::WS`
            let ws = Dataflow::WS;
            (U::from(cmd.cmd.rs1[2]) == (ws as usize).into_u()) && cmd.cmd.rs1[9]
        } else {
            b_transposed_and_ws
        }
    } else {
        b_transposed_and_ws
    }
}

/// Accumulates up to 2 commands and send it once.
///
/// It accumulates 2 commands when the first command is `PRELOAD` and `b_transposed_and_ws` is turned on.
/// Otherwise, it passthrough the incoming command directly to the egress.
fn accumulate_cmds(
    (cmd, b_transposed_and_ws): (GemminiCmd, bool),
    (cmds, _): (Array<HOption<GemminiCmd>, 2>, bool),
) -> ((Array<HOption<GemminiCmd>, 2>, bool), bool) {
    let cmds_next = if cmds[0].is_none() { cmds.set(0, Some(cmd)) } else { cmds.set(1, Some(cmd)) };

    let first_preload = cmds_next[0].is_some_and(|cmd| matches!(cmd.cmd.inst.funct, Funct::PreloadCmd));
    let unroll_preload =
        b_transposed_and_ws && cmds_next[1].is_some_and(|cmd| matches!(cmd.cmd.inst.funct, Funct::ComputeAndFlipCmd));

    let done = cmds_next[1].is_some() || !(first_preload && b_transposed_and_ws);

    ((cmds_next, unroll_preload), done)
}

/// Chunks into 1, 2, or 4 commands.
///
/// If `unroll_preload` is true, chunks 2 commands into 4 commands. For more details, consult `TransposePreloadUnroller.scala`.
fn chunk_cmds(
    (cmds, unroll_preload): (Array<HOption<GemminiCmd>, 2>, bool),
    counter: U<2>,
) -> (GemminiCmd, U<2>, bool) {
    let (cmd, is_last) = if unroll_preload {
        if counter == 0.into_u() {
            let cmd = cmds[0].unwrap();

            let rs2 = cmd.cmd.rs2 | GARBAGE_ADDR.into_u();
            let first_preload_cmd = GemminiCmd { cmd: rocc::RoCCCommand { rs2, ..cmd.cmd }, rob_id: None, ..cmd };

            (first_preload_cmd, false)
        } else if counter == 1.into_u() {
            let cmd = cmds[1].unwrap();

            let rs1 = cmd.cmd.inst.rs1 | GARBAGE_ADDR.into_u();
            let rs2 = cmd.cmd.inst.rs2 | GARBAGE_ADDR.into_u();
            let inst = rocc::RoCCInstruction { rs1, rs2, funct: Funct::ComputeAndStayCmd, ..cmd.cmd.inst };
            let first_compute_cmd = GemminiCmd { cmd: rocc::RoCCCommand { inst, ..cmd.cmd }, rob_id: None, ..cmd };

            (first_compute_cmd, false)
        } else if counter == 2.into_u() {
            let cmd = cmds[0].unwrap();

            let rs1 = cmd.cmd.rs1 | GARBAGE_ADDR.into_u();
            let second_preload_cmd = GemminiCmd { cmd: rocc::RoCCCommand { rs1, ..cmd.cmd }, ..cmd };

            (second_preload_cmd, false)
        } else {
            (cmds[1].unwrap(), true)
        }
    } else {
        (cmds[counter].unwrap(), counter == 1.into_u() || cmds[1].is_none())
    };

    (cmd, (counter + 1.into_u()).resize(), is_last)
}

/// Transpose preload unroller.
///
/// This module is responsible for unrolling the transpose preload.
// #[synthesize]
pub fn transpose_preload_unroller(cmd: Vr<GemminiCmd>) -> Vr<GemminiCmd> {
    cmd.fsm_map::<(GemminiCmd, bool), bool>(false, |ip, s| ((ip, s), update_config(ip, s)))
        .fsm_ingress::<(Array<HOption<GemminiCmd>, 2>, bool)>((None.repeat(), false), |ip, _, s| accumulate_cmds(ip, s))
        .fsm_egress::<GemminiCmd, U<2>>(0.into_u(), true, chunk_cmds)
}
