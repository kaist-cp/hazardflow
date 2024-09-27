//! Load controller.

use crate::gemmini::dma::dma_command_tracker::*;
use crate::gemmini::isa::*;
use crate::gemmini::load::rocc::*;
use crate::gemmini::scratchpad::*;
use crate::gemmini::sram::dma::*;
use crate::gemmini::*;

const BLOCK_ROWS: usize = MESH_ROWS * TILE_ROWS;
// const BLOCK_COLS: usize = MESH_COLS * TILE_COLS;

#[derive(Debug, Default, Clone, Copy)]
struct LoadState {
    stride: U<CORE_MAX_ADDR_BITS>,
    scale: U<MVIN_SCALE_BITS>,
    shrink: bool,
    block_stride: U<BLOCK_STRIDE_BITS>,
    pixel_repeat: U<PIXEL_REPEATS_BITS>,
}

#[derive(Debug, Default, Clone, Copy)]
struct Config {
    load_states: Array<LoadState, LOAD_STATES>,
}

#[derive(Debug, Clone, Copy)]
struct CmdDecoded {
    cmd: GemminiCmd,

    vaddr: U<64>,
    mvin_rs2: MvinRs2<MVIN_ROWS_BITS, MVIN_COLS_BITS>,
    config_mvin_rs1: ConfigMvinRs1<MVIN_SCALE_BITS, BLOCK_STRIDE_BITS, PIXEL_REPEATS_BITS>,

    mstatus: MStatus,

    load_state: LoadState,

    all_zeros: bool,

    actual_rows_read: U<MVIN_ROWS_BITS>,
}

fn decode_cmd(cmd: GemminiCmd, config: Config) -> CmdDecoded {
    let vaddr = cmd.cmd.rs1;
    let mvin_rs2 = MvinRs2::<MVIN_ROWS_BITS, MVIN_COLS_BITS>::from(cmd.cmd.rs2);

    let config_mvin_rs1 = ConfigMvinRs1::<MVIN_SCALE_BITS, BLOCK_STRIDE_BITS, PIXEL_REPEATS_BITS>::from(cmd.cmd.rs1);

    let mstatus = cmd.cmd.status;

    let load_state_id: U<{ clog2(LOAD_STATES) }> = if matches!(cmd.cmd.inst.funct, Funct::Load2Cmd) {
        1.into_u()
    } else if matches!(cmd.cmd.inst.funct, Funct::Load3Cmd) {
        2.into_u()
    } else {
        0.into_u()
    };
    let config_state_id = config_mvin_rs1.state_id;
    let state_id = if matches!(cmd.cmd.inst.funct, Funct::ConfigCmd) { config_state_id } else { load_state_id };

    let load_state = config.load_states[state_id];

    let all_zeros = vaddr == 0.into_u();

    let actual_rows_read = if load_state.stride == 0.into_u() && !all_zeros { 1.into_u() } else { mvin_rs2.num_rows };

    CmdDecoded { cmd, vaddr, mvin_rs2, config_mvin_rs1, mstatus, load_state, all_zeros, actual_rows_read }
}

fn update_config(cmd_decoded: CmdDecoded, config: Config) -> Config {
    // If command is not changing config, return early.
    if !matches!(cmd_decoded.cmd.cmd.inst.funct, Funct::ConfigCmd) {
        return config;
    }

    Config {
        load_states: config.load_states.set(cmd_decoded.config_mvin_rs1.state_id, LoadState {
            stride: cmd_decoded.cmd.cmd.rs2.resize(),
            scale: cmd_decoded.config_mvin_rs1.scale.resize(),
            shrink: cmd_decoded.config_mvin_rs1.shrink,
            block_stride: cmd_decoded.config_mvin_rs1.stride.resize(),
            pixel_repeat: cmd_decoded.config_mvin_rs1.pixel_repeats.resize(),
        }),
    }
}

fn compute_alloc_req<const MAX_BYTES: usize>(cmd_decoded: CmdDecoded) -> AllocReq<U<{ clog2(RS_ENTRIES) }>, MAX_BYTES>
where [(); clog2(MAX_BYTES + 1)]: {
    let cols = cmd_decoded.mvin_rs2.num_cols;
    let actual_rows_read = cmd_decoded.actual_rows_read;

    AllocReq {
        bytes_to_read: (if cmd_decoded.mvin_rs2.local_addr.is_acc_addr && !cmd_decoded.load_state.shrink {
            cols * actual_rows_read * 32.into_u::<6>()
        } else {
            cols * actual_rows_read * 8.into_u::<6>()
        } >> 3)
            .resize(),
        // `unwrap()` always success because the ROB ID is inserted in the controller between reservation station and load controller.
        tag: cmd_decoded.cmd.rob_id.unwrap(),
    }
}

fn compute_dma_req<const NCMDS: usize>(
    cmd_id: U<{ clog2(NCMDS) }>,
    cmd_decoded: CmdDecoded,
    row_counter: U<{ clog2(BLOCK_ROWS) }>,
) -> ScratchpadMemReadReq<MVIN_SCALE_BITS> {
    let localaddr = cmd_decoded.mvin_rs2.local_addr;
    let localaddr_plus_row_counter = localaddr + row_counter.resize();

    let load_state = cmd_decoded.load_state;

    ScratchpadMemReadReq {
        vaddr: (u32::from(cmd_decoded.vaddr) + u32::from(row_counter) * u32::from(load_state.stride)).into_u(),
        laddr: localaddr_plus_row_counter,
        cols: cmd_decoded.mvin_rs2.num_cols.resize(),
        repeats: if load_state.stride == 0.into_u() && !cmd_decoded.all_zeros {
            cmd_decoded.mvin_rs2.num_rows - 1.into_u()
        } else {
            0.into_u()
        }
        .resize(),
        scale: load_state.scale.resize(),
        has_acc_bitwidth: localaddr_plus_row_counter.is_acc_addr && !load_state.shrink,
        all_zeros: cmd_decoded.all_zeros,
        block_stride: load_state.block_stride.resize(),
        pixel_repeats: load_state.pixel_repeat.resize(),
        cmd_id: cmd_id.resize(),
        status: cmd_decoded.mstatus,
    }
}

/// Load controller.
///
/// It manages commands that move data from main memory to gemmini's private scratchpad or accumulator.
/// It takes ingress command from the reservation station, and returns rob id to the reservation station.
///
/// Reference: <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/LoadController.scala>
pub fn load<const NCMDS: usize, const MAX_BYTES: usize>(
    cmd: Vr<GemminiCmd>,
    dma_accessor: impl FnOnce(Vr<ScratchpadMemReadReq<MVIN_SCALE_BITS>>) -> Valid<ScratchpadMemReadResp>,
) -> Vr<U<{ clog2(RS_ENTRIES) }>>
where
    [(); clog2(NCMDS)]:,
    [(); clog2(MAX_BYTES + 1)]:,
{
    let (alloc_m, complete_m) = module_split(dma_command_tracker::<U<{ clog2(RS_ENTRIES) }>, NCMDS, MAX_BYTES>);

    // TODO: Use `LD_QUEUE_LENGTH` instead of `8`.
    let cmd = cmd.fifo::<8>().fsm_map::<CmdDecoded, Config>(Config::default(), |ip, s| {
        let cmd_decoded = decode_cmd(ip, s);
        let s_next = update_config(cmd_decoded, s);

        (cmd_decoded, s_next)
    });

    let (cmd_config, cmd_load) = cmd
        .map::<(CmdDecoded, BoundedU<2>)>(|cmd_decoded| {
            let sel = if matches!(cmd_decoded.cmd.cmd.inst.funct, Funct::ConfigCmd) { 0.into_u() } else { 1.into_u() };

            (cmd_decoded, BoundedU::new(sel))
        })
        .map_resolver_inner::<((), ())>(|_| ())
        .branch();

    cmd_config.sink_fsm_map((), |_, s| (Ready::valid(()), s));

    let alloc_resp = cmd_load
        .map(|cmd_decoded| (compute_alloc_req::<MAX_BYTES>(cmd_decoded), cmd_decoded))
        .comb(attach_payload(attach_ready(alloc_m)));

    let dma_resp = alloc_resp
        .fsm_egress::<ScratchpadMemReadReq<MVIN_SCALE_BITS>, U<{ clog2(BLOCK_ROWS) }>>(
            0.into_u(),
            true,
            true,
            |(alloc_resp, cmd_decoded), row_counter| {
                let ep = compute_dma_req(alloc_resp.cmd_id, cmd_decoded, row_counter);
                let row_counter_next = (u32::from(row_counter) + 1).into_u();
                let is_last = row_counter == (cmd_decoded.actual_rows_read - 1.into_u()).resize();

                (ep, row_counter_next, is_last)
            },
        )
        .comb(dma_accessor);

    dma_resp
        .map(|p| RequestReturned { bytes_read: p.bytes_read.resize(), cmd_id: p.cmd_id.resize() })
        .comb(complete_m)
        .map(|p| p.tag)
}

/// Debug
#[synthesize]
pub fn load_default(
    cmd: Vr<GemminiCmd>,
    dma_accessor: impl FnOnce(Vr<ScratchpadMemReadReq<MVIN_SCALE_BITS>>) -> Valid<ScratchpadMemReadResp>,
) -> Vr<U<{ clog2(RS_ENTRIES) }>> {
    load::<2, 1024>(cmd, dma_accessor)
}
