//! Execute controller.

use crate::{array_map, hpanic};

pub mod systolic_array;
pub mod transpose_preload_unroller;

use systolic_array::mesh_with_delays::*;
use systolic_array::pe::*;
use transpose_preload_unroller::*;

use crate::gemmini::isa::*;
use crate::gemmini::local_addr::*;
use crate::gemmini::sram::accumulator::*;
use crate::gemmini::sram::scratchpad::*;
use crate::gemmini::*;

#[derive(Debug, Clone, Copy)]
struct Operand {
    addr: LocalAddr,
    is_valid: bool,
    counter: U<4>,
    started: bool,
    priority: U<2>,
}

/// Configuration state.
#[derive(Debug, Clone, Copy, Default)]
struct ConfigS {
    dataflow: Dataflow,

    in_shift: U<5>,
    acc_scale: U<32>,
    activation: U<3>,
    transpose_a: bool,
    transpose_bd: bool,

    a_addr_stride: U<16>,
    c_addr_stride: U<16>,
}

/// Computation state.
#[derive(Debug, Clone, Copy, Default)]
struct CounterS {
    in_prop_flush: bool,

    a_fire_counter: U<{ clog2(BLOCK_SIZE) }>,
    b_fire_counter: U<{ clog2(BLOCK_SIZE) }>,
    d_fire_counter: U<{ clog2(BLOCK_SIZE) }>,

    a_fire_started: bool,
    b_fire_started: bool,
    d_fire_started: bool,

    a_fire_done: bool,
    b_fire_done: bool,
    d_fire_done: bool,

    a_addr_offset: U<{ 16 + clog2(BLOCK_SIZE) }>,
}

/// Decoded command.
/// TODO: Make each fields as methods.
#[derive(Default, Debug, Clone, Copy)]
struct CmdDecoded<const EX_QUEUE_LENGTH: usize> {
    cmds: Array<HOption<GemminiCmd>, EX_QUEUE_LENGTH>,
    rs1s: Array<HOption<U<64>>, EX_QUEUE_LENGTH>,
    rs2s: Array<HOption<U<64>>, EX_QUEUE_LENGTH>,

    // TODO: This can be refactored as `Array<HOption<Config/Compute/Preload>, EX_QUEUE_LENGTH>`
    do_config: bool,
    do_computes: Array<bool, EX_QUEUE_LENGTH>,
    do_preloads: Array<bool, EX_QUEUE_LENGTH>,

    in_prop: bool,
}

/// Command types in Execute module.
#[derive(Debug, Clone, Copy)]
struct ExeCmd<const EX_QUEUE_LENGTH: usize> {
    typ: ExeCmdType,
    cmd: CmdDecoded<EX_QUEUE_LENGTH>,
}

#[derive(Debug, Clone, Copy, HEq)]
enum ExeCmdType {
    Config,
    Preload,
    Compute,
    ComputeAndPreload,
    Flush,
}

#[derive(Debug, Clone, Copy)]
struct OpShape {
    rows: U<{ clog2(BLOCK_SIZE + 1) }>,
    cols: U<{ clog2(BLOCK_SIZE + 1) }>,
}

#[derive(Debug, Clone, Copy, HEq)]
enum OpBank {
    Spad(U<{ clog2(SP_BANKS) }>),
    Acc(U<{ clog2(ACC_BANKS) }>),
}

/// Extended ComputeControlSignals.
#[derive(Debug, Clone, Copy)]
struct ControlSignals {
    perform_single_mul: bool, // TODO: Delete this.

    a_bank: HOption<OpBank>,
    b_bank: HOption<OpBank>,
    d_bank: HOption<OpBank>,

    a_address: LocalAddr,
    b_address: LocalAddr,
    d_address: LocalAddr,

    a_shape: HOption<OpShape>,
    b_shape: HOption<OpShape>,
    d_shape: HOption<OpShape>,

    c_address: LocalAddr,

    spad_reads: Array<(bool, bool, bool), SP_BANKS>, // TODO: Make HOption<Enum {A, B, D}>.
    acc_reads: Array<(bool, bool, bool), ACC_BANKS>, // TODO: Make HOption<Enum {A, B, D}>.

    accumulate_zeros: bool,
    preload_zeros: bool,

    c_shape: HOption<OpShape>,

    transpose_a: bool,
    transpose_bd: bool,

    total_rows: U<5>,

    rob_id: HOption<U<{ clog2(RS_ENTRIES) }>>,

    dataflow: Dataflow,
    prop: bool,
    shift: U<5>,

    first: bool,
}

/// Package of whole configs and signals.
#[derive(Debug, Clone, Copy)]
struct MeshControlSignals<const EX_QUEUE_LENGTH: usize> {
    cmd: ExeCmd<EX_QUEUE_LENGTH>,
    config: ConfigS,
    counters: CounterS,
    signals: ControlSignals,
}

/// Information for SRAM write.
#[derive(Debug, Clone, Copy)]
pub struct MeshRespExtended {
    /// Response from the Mesh (what to write)
    pub mesh_resp: MeshResp,
    /// SRAM write counter
    pub output_counter: U<4>,
    /// Is it valid to write to the SRAM? (`!mesh_resp.tag.addr.is_garbage_addr`)
    pub start_array_outputting: bool,
}

fn decode_cmd<const EX_QUEUE_LENGTH: usize>(
    cmds: Array<HOption<GemminiCmd>, EX_QUEUE_LENGTH>,
) -> CmdDecoded<EX_QUEUE_LENGTH> {
    let functs = cmds.map(|p| p.map(|p| p.cmd.inst.funct));

    let do_config = functs[0].is_some_and(|f| f == Funct::ConfigCmd);
    let do_computes =
        functs.map(|f| f.is_some_and(|f| (f == Funct::ComputeAndFlipCmd || f == Funct::ComputeAndStayCmd)));
    let do_preloads = functs.map(|f| f.is_some_and(|f| f == Funct::PreloadCmd));
    let in_prop = functs[0].is_some_and(|f| f == Funct::ComputeAndFlipCmd);

    let rs1s = cmds.map(|p| p.map(|p| p.cmd.rs1));
    let rs2s = cmds.map(|p| p.map(|p| p.cmd.rs2));

    CmdDecoded { cmds, rs1s, rs2s, do_config, do_computes, do_preloads, in_prop }
}

fn update_ex_config<const EX_QUEUE_LENGTH: usize>(cmd: ExeCmd<EX_QUEUE_LENGTH>, config: ConfigS) -> ConfigS {
    if cmd.typ == ExeCmdType::Config {
        let cmds = cmd.cmd;
        // Default mode is FP32 -> acc_scale should be 32.
        let config_ex_rs1 = ConfigExRs1::<32>::new(cmds.rs1s[0]);
        let config_ex_rs2 = ConfigExRs2::new(cmds.rs2s[0]);

        if config_ex_rs1.cmd_type != ConfigCmd::Ex {
            return config;
        }

        // next states.
        let in_shift = config_ex_rs2.in_shift.clip_const::<5>(0);
        let acc_scale = config_ex_rs1.acc_scale;
        let activation = config_ex_rs1.activation.resize::<3>();
        let transpose_a = config_ex_rs1.transpose_a;
        let transpose_bd = config_ex_rs1.transpose_bd;
        let dataflow = config_ex_rs1.dataflow;

        let a_addr_stride = config_ex_rs1.a_stride;
        let c_addr_stride = config_ex_rs2.c_stride;

        let s_next = if !config_ex_rs1.set_only_strides {
            ConfigS { in_shift, acc_scale, activation, transpose_a, transpose_bd, dataflow, ..config }
        } else {
            config
        };

        ConfigS { a_addr_stride, c_addr_stride, ..s_next }
    } else {
        config
    }
}

fn get_exe_cmd_type<const EX_QUEUE_LENGTH: usize, const N: usize>(
    cmd_decoded: CmdDecoded<EX_QUEUE_LENGTH>,
    tags_in_progress: Array<MeshTag, N>,
    any_pending_rob_ids: bool,
    config: ConfigS,
) -> HOption<ExeCmdType> {
    let any_matmul_in_progress = tags_in_progress.any(|tag| tag.rob_id.is_some());

    if cmd_decoded.cmds[0].is_some() {
        if cmd_decoded.do_config && !any_matmul_in_progress && !any_pending_rob_ids {
            Some(ExeCmdType::Config)
        } else if cmd_decoded.do_preloads[0] && cmd_decoded.do_computes[1] {
            let raw_hazard = tags_in_progress.any(|tag| {
                let pre_rs1_addr = LocalAddr::from(cmd_decoded.rs1s[0]);
                let mul_rs1_addr = LocalAddr::from(cmd_decoded.rs1s[1]);
                let mul_rs2_addr = LocalAddr::from(cmd_decoded.rs2s[1]);

                let pre_raw_hazard = tag.addr.is_same_addr(pre_rs1_addr) && !pre_rs1_addr.is_garbage();
                let mul_raw_hazard = (tag.addr.is_same_addr(mul_rs1_addr) && !mul_rs1_addr.is_garbage())
                    || (tag.addr.is_same_addr(mul_rs2_addr) && !mul_rs2_addr.is_garbage());

                !tag.addr.is_garbage() && (pre_raw_hazard || mul_raw_hazard)
            });

            if !raw_hazard {
                Some(ExeCmdType::Preload)
            } else if config.dataflow == Dataflow::OS {
                Some(ExeCmdType::Flush)
            } else {
                None
            }
        } else if cmd_decoded.do_computes[0] && cmd_decoded.do_preloads[1] && cmd_decoded.do_computes[2] {
            let pre_rs1_addr = LocalAddr::from(cmd_decoded.rs1s[1]);
            let mul_rs1_addr = LocalAddr::from(cmd_decoded.rs1s[2]);
            let mul_rs2_addr = LocalAddr::from(cmd_decoded.rs2s[2]);

            let raw_hazard = tags_in_progress.any(|tag| {
                let pre_raw_hazard = tag.addr.is_same_addr(pre_rs1_addr) && !pre_rs1_addr.is_garbage();
                let mul_raw_hazard = (tag.addr.is_same_addr(mul_rs1_addr) && !mul_rs1_addr.is_garbage())
                    || (tag.addr.is_same_addr(mul_rs2_addr) && !mul_rs2_addr.is_garbage());

                !tag.addr.is_garbage() && (pre_raw_hazard || mul_raw_hazard)
            });

            if !raw_hazard {
                Some(ExeCmdType::ComputeAndPreload)
            } else {
                Some(ExeCmdType::Compute)
            }
        } else if cmd_decoded.do_computes[0] {
            Some(ExeCmdType::Compute)
        } else if any_matmul_in_progress && (config.dataflow == Dataflow::OS || cmd_decoded.do_config) {
            Some(ExeCmdType::Flush)
        } else {
            None
        }
    } else if any_matmul_in_progress && config.dataflow == Dataflow::OS {
        Some(ExeCmdType::Flush)
    } else {
        None
    }
}

/// Decode the command from the reservation station.
#[allow(clippy::type_complexity)]
fn cmd_decoder<const EX_QUEUE_LENGTH: usize>(
    cmd: Vr<GemminiCmd>,
) -> (
    Vr<(ExeCmd<EX_QUEUE_LENGTH>, ConfigS), { Dep::Demanding }>,
    I<VrH<(ExeCmd<EX_QUEUE_LENGTH>, ConfigS), (TagsInProgress, bool)>, { Dep::Demanding }>,
)
where
    [(); clog2(EX_QUEUE_LENGTH) + 1]:,
    [(); clog2(EX_QUEUE_LENGTH + 1) + 1]:,
{
    let cmd_fifo = cmd.map_resolver_inner::<FifoS<GemminiCmd, EX_QUEUE_LENGTH>>(|_| ()).multi_headed_transparent_fifo();

    let cmd_decoded = cmd_fifo.map(|fifo_s| decode_cmd::<EX_QUEUE_LENGTH>(fifo_s.inner_with_valid()));

    let exe_cmd = cmd_decoded
        .map_resolver_drop_with_p::<VrH<CmdDecoded<EX_QUEUE_LENGTH>, (TagsInProgress, bool, ConfigS)>>(|ip, er| {
            let (tags_in_progress, any_pending_rob_ids, config) = er.inner;

            let Some(cmd_decoded) = ip else {
                return Ready::new(false, 0.into_u());
            };

            let exe_cmd_type = get_exe_cmd_type(cmd_decoded, tags_in_progress, any_pending_rob_ids, config);

            if let Some(exe_cmd_type) = exe_cmd_type {
                let pop_count = match exe_cmd_type {
                    ExeCmdType::Config | ExeCmdType::Preload | ExeCmdType::Compute => 1.into_u(),
                    ExeCmdType::ComputeAndPreload => 2.into_u(),
                    ExeCmdType::Flush => 0.into_u(),
                };

                Ready::new(er.ready, pop_count)
            } else {
                Ready::new(false, 0.into_u())
            }
        })
        .filter_map_drop_with_r_inner::<ExeCmd<EX_QUEUE_LENGTH>>(|cmd_decoded, er| {
            let (tags_in_progress, any_pending_rob_ids, config) = er;
            let exe_cmd_type = get_exe_cmd_type(cmd_decoded, tags_in_progress, any_pending_rob_ids, config);
            exe_cmd_type.map(|typ| ExeCmd { typ, cmd: cmd_decoded })
        });

    let exe_cmd = exe_cmd
        .map_resolver_inner::<((TagsInProgress, bool), ConfigS)>(|((tags_in_progress, any_pending_rob_ids), config)| {
            (tags_in_progress, any_pending_rob_ids, config)
        })
        .transparent_fsm_map::<(ExeCmd<EX_QUEUE_LENGTH>, ConfigS)>(ConfigS::default(), |cmd, config| {
            let config_next = update_ex_config(cmd, config);
            ((cmd, config_next), config_next)
        });

    let (config_cmd, compute_cmd) = exe_cmd.map_resolver_inner::<((), (TagsInProgress, bool))>(|(_, er)| er).lfork();

    (
        config_cmd.filter(|(exe_cmd, _)| exe_cmd.typ == ExeCmdType::Config),
        compute_cmd.filter(|(exe_cmd, _)| exe_cmd.typ != ExeCmdType::Config),
    )
}

#[derive(Debug, Clone, Copy, Default)]
struct MeshInpGenState {
    a_fire_counter: U<{ clog2(BLOCK_SIZE + 1) }>,
    b_fire_counter: U<{ clog2(BLOCK_SIZE + 1) }>,
    d_fire_counter: U<{ clog2(BLOCK_SIZE + 1) }>,
    a_fire_done: bool,
    b_fire_done: bool,
    d_fire_done: bool,
}

fn filter_map_spad_resp(i: Vr<ScratchpadReadResp>) -> Vr<U<SP_DATA_WIDTH>> {
    i.filter_map(|v| if !v.from_dma { Some(v.data) } else { None })
}
fn filter_map_acc_resp(i: Vr<AccumulatorReadResp>) -> Vr<U<SP_DATA_WIDTH>> {
    i.filter_map(|v| if !v.from_dma { Some(v.data) } else { None })
}

fn filter_sram_readies(i: Vr<U<SP_DATA_WIDTH>>) -> I<VrH<U<SP_DATA_WIDTH>, bool>, { Dep::Helpful }> {
    i.map_resolver_with_p::<bool>(|_p, er| Ready::new(er.ready && er.inner, ()))
}

fn filter_sram_resps(
    spad_resps: [Vr<ScratchpadReadResp>; SP_BANKS],
    acc_resps: [Vr<AccumulatorReadResp>; ACC_BANKS],
) -> [Vr<U<SP_DATA_WIDTH>>; SP_BANKS + ACC_BANKS] {
    let spad_resps = array_map!(spad_resps, filter_map_spad_resp);
    let acc_resps = array_map!(acc_resps, filter_map_acc_resp);

    // TODO: Inelegant (I need `concat` function that is applicable for array of interfaces.)
    let [spad_resp0, spad_resp1, spad_resp2, spad_resp3] = spad_resps;
    let [acc_resp0, acc_resp1] = acc_resps;
    [spad_resp0, spad_resp1, spad_resp2, spad_resp3, acc_resp0, acc_resp1]
}

fn chunk_and_pad_zeros(
    data: U<SP_DATA_WIDTH>,
    unpadded_cols: U<{ clog2(BLOCK_SIZE + 1) }>,
) -> Array<Array<S<INPUT_BITS>, TILE_ROWS>, MESH_ROWS> {
    data.chunk::<8>()
        .enumerate()
        .map(|(idx, value)| if idx.resize() < unpadded_cols { value } else { 0.into_u::<8>() })
        .map(|v| v.chunk::<8>().map(S::from))
}

#[derive(Debug, Clone, Copy, HEq)]
enum SramReadRespInfo {
    Spad(U<SP_BANK_BITS>, U<{ clog2(BLOCK_SIZE + 1) }>), // Bank and Unpaded cols
    Acc(U<ACC_BANK_BITS>, U<{ clog2(BLOCK_SIZE + 1) }>), // Bank and Unpadded cols
    AllZero,
    None,
}

#[derive(Debug, Clone, Copy)]
struct SramReadRespInfos {
    a: SramReadRespInfo,
    b: SramReadRespInfo,
    d: SramReadRespInfo,
}

/// generate inputs for mesh_with_delays
#[allow(clippy::type_complexity)]
fn mesh_inputs(
    cntl: I<VrH<ControlSignals, TagsInProgress>, { Dep::Helpful }>,
    spad_resps: [Vr<ScratchpadReadResp>; SP_BANKS],
    acc_resps: [Vr<AccumulatorReadResp>; ACC_BANKS],
) -> (Vr<(A, B, D)>, I<VrH<MeshReq, TagsInProgress>, { Dep::Helpful }>) {
    let (mesh_req, cntl) = cntl.map_resolver_inner::<(TagsInProgress, ())>(|er| er.0).lfork();

    let req: I<VrH<MeshReq, TagsInProgress>, { Dep::Helpful }> = mesh_req.map(|cntl| MeshReq {
        total_rows: cntl.total_rows,
        tag: MeshTag {
            rob_id: cntl.rob_id,
            addr: if cntl.perform_single_mul { cntl.c_address.make_this_garbage() } else { cntl.c_address },
            rows: cntl.c_shape.map(|c| c.rows).unwrap_or(0.into_u()),
            cols: cntl.c_shape.map(|c| c.cols).unwrap_or(0.into_u()),
        },
        dataflow: cntl.dataflow,
        propagate_flip: cntl.prop,
        shift: cntl.shift,
        transpose_a: cntl.transpose_a,
        transpose_bd: cntl.transpose_bd,
        flush: false,
    });

    let (info, zeros) = cntl
        .fsm_egress(MeshInpGenState::default(), true, true, |cntl, s| {
            let (read_a_from_sram, read_b_from_sram, read_d_from_sram) = read_from_sram(cntl, s);
            let (a_unpadded_cols, b_unpadded_cols, d_unpadded_cols) = unpadded_cols(cntl, s);

            let all_zeros = cntl.preload_zeros || cntl.accumulate_zeros;

            let a_info = if s.a_fire_done && all_zeros {
                SramReadRespInfo::None
            } else if cntl.a_bank.is_none() || (a_unpadded_cols == 0.into_u() && !s.a_fire_done) {
                SramReadRespInfo::AllZero
            } else if read_a_from_sram {
                if let Some(a_bank) = cntl.a_bank {
                    match a_bank {
                        OpBank::Spad(bank) => SramReadRespInfo::Spad(bank, a_unpadded_cols),
                        OpBank::Acc(bank) => SramReadRespInfo::Acc(bank, a_unpadded_cols),
                    }
                } else {
                    display!("Unreachable");
                    SramReadRespInfo::None
                }
            } else {
                SramReadRespInfo::None
            };

            let b_info = if s.b_fire_done && all_zeros {
                SramReadRespInfo::None
            } else if cntl.b_bank.is_none() || (b_unpadded_cols == 0.into_u() && !s.b_fire_done) {
                SramReadRespInfo::AllZero
            } else if read_b_from_sram {
                if let Some(b_bank) = cntl.b_bank {
                    match b_bank {
                        OpBank::Spad(bank) => SramReadRespInfo::Spad(bank, b_unpadded_cols),
                        OpBank::Acc(bank) => SramReadRespInfo::Acc(bank, b_unpadded_cols),
                    }
                } else {
                    display!("Unreachable");
                    SramReadRespInfo::None
                }
            } else {
                SramReadRespInfo::None
            };

            let d_info = if s.d_fire_done && all_zeros {
                SramReadRespInfo::None
            } else if cntl.d_bank.is_none() || (d_unpadded_cols == 0.into_u() && !s.d_fire_done) {
                SramReadRespInfo::AllZero
            } else if read_d_from_sram {
                if let Some(d_bank) = cntl.d_bank {
                    match d_bank {
                        OpBank::Spad(bank) => SramReadRespInfo::Spad(bank, d_unpadded_cols),
                        OpBank::Acc(bank) => SramReadRespInfo::Acc(bank, d_unpadded_cols),
                    }
                } else {
                    display!("Unreachable");
                    SramReadRespInfo::None
                }
            } else {
                SramReadRespInfo::None
            };

            let next_a_fire_counter = (s.a_fire_counter + (a_info != SramReadRespInfo::None).into_u()).resize();
            let next_b_fire_counter = (s.b_fire_counter + (b_info != SramReadRespInfo::None).into_u()).resize();
            let next_d_fire_counter = (s.d_fire_counter + (d_info != SramReadRespInfo::None).into_u()).resize();

            // In `preload_zeros` and `accumulate_zeros`, we have to count how many rows are fired.
            let a_fire_done =
                s.a_fire_done || (cntl.a_bank.is_none() && !all_zeros) || next_a_fire_counter >= cntl.total_rows;
            let b_fire_done =
                s.b_fire_done || (cntl.b_bank.is_none() && !all_zeros) || next_b_fire_counter >= cntl.total_rows;
            let d_fire_done =
                s.d_fire_done || (cntl.d_bank.is_none() && !all_zeros) || next_d_fire_counter >= cntl.total_rows;

            let done = a_fire_done && b_fire_done && d_fire_done;

            let ep: SramReadRespInfos = SramReadRespInfos { a: a_info, b: b_info, d: d_info };
            let s_next: MeshInpGenState = MeshInpGenState {
                a_fire_counter: next_a_fire_counter,
                b_fire_counter: next_b_fire_counter,
                d_fire_counter: next_d_fire_counter,
                a_fire_done,
                b_fire_done,
                d_fire_done,
            };

            (ep, s_next, done)
        })
        .map(|p| {
            if p.a == SramReadRespInfo::AllZero && p.b == SramReadRespInfo::AllZero && p.d == SramReadRespInfo::AllZero
            {
                // If `preload_zeros` or `accumulate_zeros` is true, we have to fire zeros.
                (p, BoundedU::new(1.into_u()))
            } else {
                (p, BoundedU::new(0.into_u()))
            }
        })
        .map_resolver_inner(|(_, er1)| er1)
        .branch();

    let (a_zero, b_zero, d_zero) = zeros
        .map(|_| {
            let zeros = Some(chunk_and_pad_zeros(0.into_u(), 0.into_u()));
            (zeros, zeros, zeros)
        })
        .unzip();

    let sram_resps = filter_sram_resps(spad_resps, acc_resps);
    let sram_resps = array_map!(sram_resps, filter_sram_readies).zip_any_i_vr_h();

    let (a_data, b_data, d_data) = (info, sram_resps)
        .join()
        .map_resolver_with_p::<((), (), ())>(|p, er| {
            let readies_to_sram = if let Some((info, data)) = p {
                let a_bank: HOption<U<3>> = match info.a {
                    SramReadRespInfo::AllZero | SramReadRespInfo::None => None,
                    SramReadRespInfo::Spad(bank, ..) => Some(bank.resize()),
                    SramReadRespInfo::Acc(bank, ..) => Some(bank.resize::<2>() + SP_BANKS.into_u()),
                };
                let b_bank: HOption<U<3>> = match info.b {
                    SramReadRespInfo::AllZero | SramReadRespInfo::None => None,
                    SramReadRespInfo::Spad(bank, ..) => Some(bank.resize()),
                    SramReadRespInfo::Acc(bank, ..) => Some(bank.resize::<2>() + SP_BANKS.into_u()),
                };
                let d_bank: HOption<U<3>> = match info.d {
                    SramReadRespInfo::AllZero | SramReadRespInfo::None => None,
                    SramReadRespInfo::Spad(bank, ..) => Some(bank.resize()),
                    SramReadRespInfo::Acc(bank, ..) => Some(bank.resize::<2>() + SP_BANKS.into_u()),
                };

                range::<6>().map(|idx| {
                    let not_a = a_bank.is_none() || a_bank.is_some_and(|a_bank| a_bank != idx);
                    let a_is_ready = a_bank.is_some_and(|a_bank| (a_bank == idx && data[idx].is_some()));

                    let not_b = b_bank.is_none() || b_bank.is_some_and(|b_bank| b_bank != idx);
                    let b_is_ready = b_bank.is_some_and(|b_bank| (b_bank == idx && data[idx].is_some()));

                    let not_d = d_bank.is_none() || d_bank.is_some_and(|d_bank| d_bank != idx);
                    let d_is_ready = d_bank.is_some_and(|d_bank| (d_bank == idx && data[idx].is_some()));

                    (not_a && not_b && not_d, a_is_ready || b_is_ready || d_is_ready)
                })
            } else {
                (false, false).repeat::<{ SP_BANKS + ACC_BANKS }>()
            };

            let all_ready = readies_to_sram.all(|(p0, p1)| p0 || p1);
            let selective_readies = [
                readies_to_sram[0].1,
                readies_to_sram[1].1,
                readies_to_sram[2].1,
                readies_to_sram[3].1,
                readies_to_sram[4].1,
                readies_to_sram[5].1,
            ];

            if all_ready {
                er.map(|_| ((), selective_readies))
            } else {
                Ready::new(false, ((), [false; SP_BANKS + ACC_BANKS]))
            }
        })
        .filter_map::<(HOption<A>, HOption<B>, HOption<D>)>(|(cntl, data)| {
            let a_bank = match cntl.a {
                SramReadRespInfo::AllZero | SramReadRespInfo::None => None,
                SramReadRespInfo::Spad(bank, unpadded) => Some((bank, unpadded)),
                SramReadRespInfo::Acc(bank, unpadded) => Some((bank + SP_BANKS.into_u(), unpadded)),
            };
            let b_bank = match cntl.b {
                SramReadRespInfo::AllZero | SramReadRespInfo::None => None,
                SramReadRespInfo::Spad(bank, unpadded) => Some((bank, unpadded)),
                SramReadRespInfo::Acc(bank, unpadded) => Some((bank + SP_BANKS.into_u(), unpadded)),
            };
            let d_bank = match cntl.d {
                SramReadRespInfo::AllZero | SramReadRespInfo::None => None,
                SramReadRespInfo::Spad(bank, unpadded) => Some((bank, unpadded)),
                SramReadRespInfo::Acc(bank, unpadded) => Some((bank + SP_BANKS.into_u(), unpadded)),
            };

            let a_data: HOption<A> = if a_bank.is_some_and(|(bank, _)| data[bank].is_some()) {
                if let Some((bank, unpadded)) = a_bank {
                    data[bank].map(|data| chunk_and_pad_zeros(data, unpadded))
                } else {
                    // Unreachable
                    None
                }
            } else if cntl.a == SramReadRespInfo::None {
                None
            } else if cntl.a == SramReadRespInfo::AllZero {
                Some(chunk_and_pad_zeros(0.into_u(), 0.into_u()))
            } else {
                None
            };

            let b_data: HOption<B> = if b_bank.is_some_and(|(bank, _)| data[bank].is_some()) {
                if let Some((bank, unpadded)) = b_bank {
                    data[bank].map(|data| chunk_and_pad_zeros(data, unpadded))
                } else {
                    // Unreachable
                    None
                }
            } else if cntl.b == SramReadRespInfo::None {
                None
            } else if cntl.b == SramReadRespInfo::AllZero {
                Some(chunk_and_pad_zeros(0.into_u(), 0.into_u()))
            } else {
                None
            };

            let d_data: HOption<D> = if d_bank.is_some_and(|(bank, _)| data[bank].is_some()) {
                if let Some((bank, unpadded)) = d_bank {
                    data[bank].map(|data| chunk_and_pad_zeros(data, unpadded))
                } else {
                    // Unreachable
                    None
                }
            } else if cntl.d == SramReadRespInfo::None {
                None
            } else if cntl.d == SramReadRespInfo::AllZero {
                Some(chunk_and_pad_zeros(0.into_u(), 0.into_u()))
            } else {
                None
            };

            if a_data.is_some() || b_data.is_some() || d_data.is_some() {
                Some((a_data, b_data, d_data))
            } else {
                None
            }
        })
        .unzip_some();

    let a_data = [a_data, a_zero].merge();
    let b_data = [b_data, b_zero].merge();
    let d_data = [d_data, d_zero].merge();

    let data = (
        a_data.filter_map(|p| p).reg_fwd(true),
        b_data.filter_map(|p| p).reg_fwd(true),
        d_data.filter_map(|p| p).reg_fwd(true),
    )
        .join_vr();

    (data, req.reg_fwd(true))
}

// Helper function of `mesh_inputs`
fn read_from_sram(cntl: ControlSignals, s: MeshInpGenState) -> (bool, bool, bool) {
    let read_a_from_sram = !s.a_fire_done
        && cntl.a_bank.is_some()
        && cntl.a_shape.map(|a| a.rows).is_some_and(|a_rows| s.a_fire_counter < a_rows);
    let read_b_from_sram = !s.b_fire_done
        && cntl.b_bank.is_some()
        && cntl.b_shape.map(|b| b.rows).is_some_and(|b_rows| s.b_fire_counter < b_rows);
    let read_d_from_sram = !s.d_fire_done
        && cntl.d_bank.is_some()
        && cntl.d_shape.map(|d| d.rows).is_some_and(|d_rows| s.d_fire_counter >= (BLOCK_SIZE.into_u() - d_rows));

    let ab_conflict = (read_a_from_sram && read_b_from_sram) && (cntl.a_bank == cntl.b_bank);
    let ad_conflict = (read_a_from_sram && read_d_from_sram) && (cntl.a_bank == cntl.d_bank);
    let bd_confict = (read_b_from_sram && read_d_from_sram) && (cntl.b_bank == cntl.d_bank);

    let block_a_read =
        (ab_conflict && s.a_fire_counter > s.b_fire_counter) || (ad_conflict && s.a_fire_counter > s.d_fire_counter);
    let block_b_read =
        (ab_conflict && s.b_fire_counter >= s.a_fire_counter) || (bd_confict && s.b_fire_counter > s.d_fire_counter);
    let block_d_read =
        (ad_conflict && s.d_fire_counter >= s.a_fire_counter) || (bd_confict && s.d_fire_counter >= s.b_fire_counter);

    let read_a_from_sram = read_a_from_sram && !block_a_read;
    let read_b_from_sram = read_b_from_sram && !block_b_read;
    let read_d_from_sram = read_d_from_sram && !block_d_read;

    (read_a_from_sram, read_b_from_sram, read_d_from_sram)
}

// Helper function of `mesh_inputs`
fn unpadded_cols(
    cntl: ControlSignals,
    s: MeshInpGenState,
) -> (U<{ clog2(BLOCK_SIZE + 1) }>, U<{ clog2(BLOCK_SIZE + 1) }>, U<{ clog2(BLOCK_SIZE + 1) }>) {
    let a_row_is_not_all_zeros = cntl.a_shape.map(|a| a.rows).is_some_and(|a_rows| s.a_fire_counter < a_rows);
    let b_row_is_not_all_zeros = cntl.b_shape.map(|b| b.rows).is_some_and(|b_rows| s.b_fire_counter < b_rows);
    let d_row_is_not_all_zeros =
        cntl.d_shape.map(|d| d.rows).is_some_and(|d_rows| s.d_fire_counter >= (BLOCK_SIZE.into_u() - d_rows));

    let a_unpadded_cols = if a_row_is_not_all_zeros { cntl.a_shape.map(|a| a.cols).unwrap() } else { 0.into_u() };
    let b_unpadded_cols = if b_row_is_not_all_zeros { cntl.b_shape.map(|b| b.cols).unwrap() } else { 0.into_u() };
    let d_unpadded_cols = if d_row_is_not_all_zeros { cntl.d_shape.map(|d| d.cols).unwrap() } else { 0.into_u() };

    (a_unpadded_cols, b_unpadded_cols, d_unpadded_cols)
}

struct WriteSignal {
    start_array_outputting: bool,
    w_address: LocalAddr,
    write_to_acc: bool,
    w_bank: U<2>,
    w_row: U<12>,
    is_garbage_addr: bool,
    write_this_row: bool,
    w_mask: U<BLOCK_SIZE>,
    // w_total_output_rows: U<5>,
}

fn compute_write_signal(resp: (MeshRespExtended, (Dataflow, U<3>, U<16>))) -> WriteSignal {
    let (MeshRespExtended { start_array_outputting, mesh_resp, output_counter }, (dataflow, _, c_addr_stride)) = resp;

    let w_total_output_rows = mesh_resp.total_rows;
    let is_garbage_addr = mesh_resp.tag.addr.is_garbage();
    let w_matrix_rows = mesh_resp.tag.rows;
    let w_matrix_cols = mesh_resp.tag.cols;

    let w_mask = range::<BLOCK_SIZE>().map(|p| p.resize() < w_matrix_cols);

    let w_address = if dataflow == Dataflow::WS {
        LocalAddr {
            data: (mesh_resp.tag.addr.data.resize() + output_counter * c_addr_stride).resize(),
            ..mesh_resp.tag.addr
        }
    } else {
        LocalAddr {
            data: (mesh_resp.tag.addr.data
                + (w_total_output_rows.resize() - 1.into_u() - output_counter * c_addr_stride).resize())
            .resize(),
            ..mesh_resp.tag.addr
        }
    };
    let write_to_acc = w_address.is_acc_addr;

    let (w_bank, w_row) = if write_to_acc {
        (w_address.acc_bank().resize::<2>(), w_address.acc_row().resize::<12>())
    } else {
        (w_address.sp_bank(), w_address.sp_row())
    };

    let write_this_row = if dataflow == Dataflow::WS {
        output_counter.resize() < w_matrix_rows
    } else {
        (w_total_output_rows - 1.into_u() - output_counter.resize()) < w_matrix_rows
    };

    WriteSignal {
        start_array_outputting,
        w_address,
        write_to_acc,
        w_bank,
        w_row,
        is_garbage_addr,
        write_this_row,
        w_mask,
    }
}

// Same as `clippedToWidthOf` function.
// <https://github.com/ucb-bar/gemmini/blob/be2e9f26181658895ebc7ca7f7d6be6210f5cdef/src/main/scala/gemmini/Arithmetic.scala#L122C20-L126>
#[allow(clippy::identity_op)]
fn clip_with_saturation(val: U<20>) -> U<8> {
    let val_msb = val[20 - 1];
    let sat_max = U::from(S::<8>::signed_max());
    let sat_min = U::from(S::<8>::signed_min());

    // TODO: Better way for signed comparison? Modify compiler for signed comparison.
    if !val_msb && val > sat_max.resize() {
        sat_max
    } else if val_msb && val > sat_min.resize() {
        sat_min
    } else {
        val.clip_const::<8>(0)
    }
}

fn spad_write_req(resp: (MeshRespExtended, (Dataflow, U<3>, U<16>)), bank_i: U<2>) -> HOption<ScratchpadWriteReq> {
    let write_signals = compute_write_signal(resp);
    let activation = resp.1 .1;
    let resp = resp.0;

    // TODO: support multiple tiles.
    let activated_wdata: U<128> = resp
        .mesh_resp
        .data
        .map(|e| {
            let e_clipped = clip_with_saturation(U::from(e[0])); // Lower 8 bits
            if activation == 1.into_u() {
                // Check MSB for signedness.
                if e_clipped[8 - 1] {
                    0.into_u()
                } else {
                    e_clipped
                }
            } else {
                e_clipped
            }
        })
        .concat();

    if write_signals.start_array_outputting
        && write_signals.w_bank.resize() == bank_i
        && !write_signals.write_to_acc
        && !write_signals.is_garbage_addr
        && write_signals.write_this_row
    {
        Some(ScratchpadWriteReq {
            addr: write_signals.w_row.resize(),
            data: activated_wdata,
            mask: write_signals.w_mask,
        })
    } else {
        None
    }
}

fn acc_write_req(resp: (MeshRespExtended, (Dataflow, U<3>, U<16>)), bank_i: U<1>) -> HOption<AccumulatorWriteReq> {
    let write_signals = compute_write_signal(resp);
    let resp = resp.0;

    let wdata = resp.mesh_resp.data.map(|v| U::from(v[0].sext::<32>()));
    let wmask = write_signals.w_mask.map(|v| v.repeat::<4>()).concat();

    if write_signals.start_array_outputting
        && write_signals.w_bank == bank_i.resize()
        && write_signals.write_to_acc
        && !write_signals.is_garbage_addr
        && write_signals.write_this_row
    {
        Some(AccumulatorWriteReq {
            addr: write_signals.w_row.resize(),
            data: wdata,
            acc: write_signals.w_address.accumulate,
            mask: wmask,
        })
    } else {
        None
    }
}

fn get_address_place<const EX_QUEUE_LENGTH: usize>(
    cmd: ExeCmd<EX_QUEUE_LENGTH>,
    should_be_fed_into_transposer: bool,
) -> U<2> {
    if matches!(cmd.typ, ExeCmdType::Preload) {
        1.into_u()
    } else if should_be_fed_into_transposer {
        2.into_u()
    } else {
        0.into_u()
    }
}

fn get_row_and_cols(cmd: HOption<U<64>>, transpose: bool) -> HOption<OpShape> {
    let rows = get_rows(cmd);
    let cols = get_cols(cmd);

    if transpose { cols.zip(rows) } else { rows.zip(cols) }.map(|(rows, cols)| OpShape { rows, cols })
}

fn get_rows(cmd: HOption<U<64>>) -> HOption<U<{ clog2(BLOCK_SIZE + 1) }>> {
    cmd.map(|p| p.clip_const::<{ clog2(BLOCK_SIZE + 1) }>(48))
}

fn get_cols(cmd: HOption<U<64>>) -> HOption<U<{ clog2(BLOCK_SIZE + 1) }>> {
    cmd.map(|p| p.clip_const::<{ clog2(BLOCK_SIZE + 1) }>(32))
}

// Check if the operands are valid to send the request to the SRAM.
// The operand should wait if
// - There exists the other operand with same bank, same counter and higher priority (`a` has the strongest priority).
// - There exists the other operand with the same bank and one counter lagged behind.
fn orchestrate_operand_read_availabilities(operands: Array<Operand, 3>, total_rows: U<5>) -> (bool, bool, bool) {
    let valids = operands.map(|op| {
        let others = operands.map(|other_op| if other_op.priority != op.priority { Some(other_op) } else { None });

        let same_bank = others.map(|other| {
            other.is_some_and(|other| {
                let both_valid = op.is_valid && other.is_valid;
                let same_acc_bank =
                    op.addr.is_acc_addr && other.addr.is_acc_addr && op.addr.acc_bank() == other.addr.acc_bank();
                let same_sp_bank =
                    !op.addr.is_acc_addr && !other.addr.is_acc_addr && op.addr.sp_bank() == other.addr.sp_bank();

                both_valid && (same_acc_bank || same_sp_bank)
            })
        });
        let same_counter =
            others.map(|other| other.is_some_and(|other| op.started == other.started && op.counter == other.counter));
        let higher_priorities = others.map(|other| other.is_some_and(|other| other.priority < op.priority));

        let one_ahead = others.map(|other| {
            other.is_some_and(|other| {
                (op.is_valid && op.started)
                    && (other.is_valid && (op.counter == wrapping_inc::<4>(other.counter, total_rows)))
            })
        });

        let zipped = (same_bank.zip(same_counter)).zip(one_ahead.zip(higher_priorities));
        let must_wait_for = others
            .zip(zipped)
            .map(|x| {
                x.0.is_some_and(|_| {
                    let ((same_bank, same_counter), (one_ahead, higher_priority)) = x.1;

                    (same_bank && higher_priority && same_counter) || (same_bank && one_ahead)
                })
            })
            .any(|is_wait| is_wait);

        !must_wait_for
    });

    (valids[0], valids[1], valids[2])
}

/// Compute control signals.
fn compute_control_signals<const EX_QUEUE_LENGTH: usize>(
    cmd: ExeCmd<EX_QUEUE_LENGTH>,
    config: ConfigS,
    counter_curr: CounterS,
    sram_read_req_readies: Array<bool, 6>,
) -> (ControlSignals, bool, CounterS) {
    let perform_single_mul = matches!(cmd.typ, ExeCmdType::Compute);

    let a_should_be_fed_into_transposer =
        if config.dataflow == Dataflow::WS { config.transpose_a } else { !config.transpose_a };
    let b_should_be_fed_into_transposer = config.dataflow == Dataflow::OS && config.transpose_bd;
    let d_should_be_fed_into_transposer = config.dataflow == Dataflow::WS && config.transpose_bd;

    let preload_cmd_place: U<2> = if matches!(cmd.typ, ExeCmdType::Preload) { 0.into_u() } else { 1.into_u() };
    let a_address_place = get_address_place(cmd, a_should_be_fed_into_transposer);
    let b_address_place = get_address_place(cmd, b_should_be_fed_into_transposer);

    // SRAM addresses of matmul operands
    let a_address_rs1 = LocalAddr::from(cmd.cmd.rs1s[a_address_place]);
    let b_address_rs2 = LocalAddr::from(cmd.cmd.rs2s[b_address_place]);
    let d_address_rs1 = LocalAddr::from(cmd.cmd.rs1s[preload_cmd_place]);
    let c_address_rs2 = LocalAddr::from(cmd.cmd.rs2s[preload_cmd_place]);

    let a_shape = get_row_and_cols(cmd.cmd.rs1s[a_address_place], config.transpose_a);
    let b_shape = get_row_and_cols(cmd.cmd.rs2s[b_address_place], b_should_be_fed_into_transposer);
    let d_shape = get_row_and_cols(cmd.cmd.rs1s[preload_cmd_place], d_should_be_fed_into_transposer);
    let c_shape = get_row_and_cols(cmd.cmd.rs2s[preload_cmd_place], false);

    let (send_a_to_mesh, send_b_to_mesh, send_d_to_mesh) = match cmd.typ {
        ExeCmdType::Config => hpanic!("Config command is not allowed here."),
        ExeCmdType::Preload => (a_should_be_fed_into_transposer, b_should_be_fed_into_transposer, true),
        ExeCmdType::ComputeAndPreload => (true, true, true),
        ExeCmdType::Compute => (!a_should_be_fed_into_transposer, !b_should_be_fed_into_transposer, false),
        ExeCmdType::Flush => (false, false, false),
    };

    let a_is_not_from_sram = a_address_rs1.is_garbage() || !send_a_to_mesh || counter_curr.a_fire_done;
    let b_is_not_from_sram = b_address_rs2.is_garbage() || !send_b_to_mesh || counter_curr.b_fire_done;
    let d_is_not_from_sram = d_address_rs1.is_garbage() || !send_d_to_mesh || counter_curr.d_fire_done;

    // What is this condition for?
    // config.transpose_a : false
    // config.transpose_bd: false
    let total_rows: U<5> = if config.dataflow == Dataflow::WS
        && d_is_not_from_sram
        && !a_should_be_fed_into_transposer
        && !b_should_be_fed_into_transposer
        && !d_should_be_fed_into_transposer
    {
        let rows_a: U<5> = if a_is_not_from_sram { 1.into_u() } else { a_shape.map(|a| a.rows).unwrap() };
        let rows_b: U<5> = if b_is_not_from_sram { 1.into_u() } else { b_shape.map(|b| b.rows).unwrap() };

        let total_rows: U<5> = if rows_a < rows_b { rows_b } else { rows_a };
        let total_rows: U<5> = if total_rows < 4.into_u() { 4.into_u() } else { total_rows };

        total_rows // max(a_rows, b_rows, 4)
    } else {
        BLOCK_SIZE.into_u()
    };

    let a_bank = a_address_rs1.sp_bank();
    let b_bank = b_address_rs2.sp_bank();
    let d_bank = d_address_rs1.sp_bank();

    let a_bank_acc = a_address_rs1.acc_bank();
    let b_bank_acc = b_address_rs2.acc_bank();
    let d_bank_acc = d_address_rs1.acc_bank();

    let a_read_from_acc = a_address_rs1.is_acc_addr;
    let b_read_from_acc = b_address_rs2.is_acc_addr;
    let d_read_from_acc = d_address_rs1.is_acc_addr;

    let a_operand = Operand {
        addr: a_address_rs1,
        is_valid: !a_is_not_from_sram,
        counter: counter_curr.a_fire_counter,
        started: counter_curr.a_fire_started,
        priority: 0.into_u(),
    };
    let b_operand = Operand {
        addr: b_address_rs2,
        is_valid: !b_is_not_from_sram,
        counter: counter_curr.b_fire_counter,
        started: counter_curr.b_fire_started,
        priority: 1.into_u(),
    };
    let d_operand = Operand {
        addr: d_address_rs1,
        is_valid: !d_is_not_from_sram,
        counter: counter_curr.d_fire_counter,
        started: counter_curr.d_fire_started,
        priority: 2.into_u(),
    };
    let operands = Array::from([a_operand, b_operand, d_operand]);
    let (can_read_a_from_sram, can_read_b_from_sram, can_read_d_from_sram) =
        orchestrate_operand_read_availabilities(operands, total_rows);

    let a_address =
        LocalAddr { data: (a_address_rs1.data + counter_curr.a_addr_offset.resize()).resize(), ..a_address_rs1 };
    let b_address =
        LocalAddr { data: (b_address_rs2.data + counter_curr.b_fire_counter.resize()).resize(), ..b_address_rs2 };
    let d_address = LocalAddr {
        data: (d_address_rs1.data + ((BLOCK_SIZE - 1).into_u() - counter_curr.d_fire_counter).resize()).resize(),
        ..d_address_rs1
    };

    let a_row_is_not_all_zeros = counter_curr.a_fire_counter.resize() < a_shape.map(|a| a.rows).unwrap_or(0.into_u());
    let b_row_is_not_all_zeros = counter_curr.b_fire_counter.resize() < b_shape.map(|b| b.rows).unwrap_or(0.into_u());
    let d_row_is_not_all_zeros = ((BLOCK_SIZE - 1).into_u() - counter_curr.d_fire_counter).resize()
        < d_shape.map(|d| d.rows).unwrap_or(0.into_u());

    // Special cases
    let accumulate_zeros = cmd.typ == ExeCmdType::Compute
        && send_b_to_mesh
        && (a_is_not_from_sram && b_is_not_from_sram && d_is_not_from_sram); // In this case, send 0 to b.
    let preload_zeros = matches!(cmd.typ, ExeCmdType::Preload | ExeCmdType::ComputeAndPreload)
        && (a_is_not_from_sram && b_is_not_from_sram && d_is_not_from_sram); // In this case, send 0 to d.

    let a_send_read_req_to_sram = !a_is_not_from_sram && a_row_is_not_all_zeros && can_read_a_from_sram;
    let b_send_read_req_to_sram = !b_is_not_from_sram && b_row_is_not_all_zeros && can_read_b_from_sram;
    let d_send_read_req_to_sram = !d_is_not_from_sram && d_row_is_not_all_zeros && can_read_d_from_sram;

    let spad_reads = range::<4>().map(|bank_i| {
        let read_a = a_send_read_req_to_sram && !a_read_from_acc && a_bank == bank_i;
        let read_b = b_send_read_req_to_sram && !b_read_from_acc && b_bank == bank_i;
        let read_d = d_send_read_req_to_sram && !d_read_from_acc && d_bank == bank_i;

        (read_a, read_b, read_d)
    });
    let acc_reads = range::<2>().map(|bank_i| {
        let read_a = a_send_read_req_to_sram && a_read_from_acc && a_bank_acc.resize() == bank_i;
        let read_b = b_send_read_req_to_sram && b_read_from_acc && b_bank_acc.resize() == bank_i;
        let read_d = d_send_read_req_to_sram && d_read_from_acc && d_bank_acc.resize() == bank_i;

        (read_a, read_b, read_d)
    });

    let a_fired_to_sram = a_send_read_req_to_sram
        && !((spad_reads.enumerate().map(|(idx, (read_a, ..))| read_a && !sram_read_req_readies[idx]).any(|x| x))
            || (acc_reads
                .enumerate()
                .map(|(idx, (read_a, ..))| read_a && !sram_read_req_readies[idx + U::from(4)])
                .any(|x| x)));
    let b_fired_to_sram = b_send_read_req_to_sram
        && !((spad_reads.enumerate().map(|(idx, (_, read_b, _))| read_b && !sram_read_req_readies[idx]).any(|x| x))
            || (acc_reads
                .enumerate()
                .map(|(idx, (_, read_b, _))| read_b && !sram_read_req_readies[idx + U::from(4)])
                .any(|x| x)));
    let d_fired_to_sram = d_send_read_req_to_sram
        && !((spad_reads.enumerate().map(|(idx, (_, _, read_d))| read_d && !sram_read_req_readies[idx]).any(|x| x))
            || (acc_reads
                .enumerate()
                .map(|(idx, (_, _, read_d))| read_d && !sram_read_req_readies[idx + U::from(4)])
                .any(|x| x)));

    let update_a_counter = a_fired_to_sram || a_is_not_from_sram || !a_row_is_not_all_zeros;
    let update_b_counter = b_fired_to_sram || b_is_not_from_sram || !b_row_is_not_all_zeros;
    let update_d_counter = d_fired_to_sram || d_is_not_from_sram || !d_row_is_not_all_zeros;

    let a_fire_done = counter_curr.a_fire_done
        || (counter_curr.a_fire_counter.resize() == (total_rows - 1.into_u()) && update_a_counter);
    let b_fire_done = counter_curr.b_fire_done
        || (counter_curr.b_fire_counter.resize() == (total_rows - 1.into_u()) && update_b_counter);
    let d_fire_done = counter_curr.d_fire_done
        || (counter_curr.d_fire_counter.resize() == (total_rows - 1.into_u()) && update_d_counter);
    let last = (a_fire_done && b_fire_done && d_fire_done)
        && (counter_curr.a_fire_started || counter_curr.b_fire_started || counter_curr.d_fire_started);

    let (a_fire_counter, a_addr_offset, a_fire_started) = if update_a_counter {
        let a_fire_counter = wrapping_inc::<4>(counter_curr.a_fire_counter, total_rows);
        let a_addr_offset: U<20> = if counter_curr.a_fire_counter == (total_rows - 1.into_u()).resize() {
            0.into_u()
        } else {
            (counter_curr.a_addr_offset + config.a_addr_stride.resize()).resize()
        };

        (a_fire_counter, a_addr_offset, true)
    } else {
        (counter_curr.a_fire_counter, counter_curr.a_addr_offset, counter_curr.a_fire_started)
    };

    let (b_fire_counter, b_fire_started) = if update_b_counter {
        (wrapping_inc::<4>(counter_curr.b_fire_counter, total_rows), true)
    } else {
        (counter_curr.b_fire_counter, counter_curr.b_fire_started)
    };

    let (d_fire_counter, d_fire_started) = if update_d_counter {
        (wrapping_inc::<4>(counter_curr.d_fire_counter, total_rows), true)
    } else {
        (counter_curr.d_fire_counter, counter_curr.d_fire_started)
    };

    let a_bank = if a_is_not_from_sram {
        None
    } else if a_address_rs1.is_acc_addr {
        Some(OpBank::Acc(a_address_rs1.acc_bank()))
    } else {
        Some(OpBank::Spad(a_address_rs1.sp_bank()))
    };
    let b_bank = if b_is_not_from_sram {
        None
    } else if b_address_rs2.is_acc_addr {
        Some(OpBank::Acc(b_address_rs2.acc_bank()))
    } else {
        Some(OpBank::Spad(b_address_rs2.sp_bank()))
    };
    let d_bank = if d_is_not_from_sram {
        None
    } else if d_address_rs1.is_acc_addr {
        Some(OpBank::Acc(d_address_rs1.acc_bank()))
    } else {
        Some(OpBank::Spad(d_address_rs1.sp_bank()))
    };

    let last = last || matches!(cmd.typ, ExeCmdType::Flush);

    let signals = ControlSignals {
        perform_single_mul,

        a_shape,
        b_shape,
        d_shape,

        a_address,
        b_address,
        d_address,

        c_address: c_address_rs2,

        a_bank,
        b_bank,
        d_bank,

        spad_reads,
        acc_reads,

        c_shape,

        accumulate_zeros,
        preload_zeros,

        total_rows,

        rob_id: None,

        dataflow: config.dataflow,
        shift: config.in_shift,
        transpose_a: config.transpose_a,
        transpose_bd: config.transpose_bd,

        prop: cmd.cmd.in_prop,

        first: !counter_curr.a_fire_started && !counter_curr.b_fire_started && !counter_curr.d_fire_started,
    };

    let counter_next = CounterS {
        a_fire_counter,
        b_fire_counter,
        d_fire_counter,

        a_fire_started: a_fire_started && !last,
        b_fire_started: b_fire_started && !last,
        d_fire_started: d_fire_started && !last,

        a_fire_done: a_fire_done && !last,
        b_fire_done: b_fire_done && !last,
        d_fire_done: d_fire_done && !last,

        a_addr_offset,
        in_prop_flush: false, // To be udpated in the next `fsm_map`
    };

    (signals, last, counter_next)
}

fn filter_req<P: Copy, const D: Dep>(p: I<VrH<(bool, P), bool>, { D }>) -> Vr<P, { D }> {
    p.map_resolver(|er| er.ready).filter_map(|(is_valid, req)| if is_valid { Some(req) } else { None })
}

/// Generate scratchpad read requests
fn spad_read_req<const EX_QUEUE_LENGTH: usize>(
    cmd_mesh_spad: I<VrH<MeshControlSignals<EX_QUEUE_LENGTH>, Array<bool, SP_BANKS>>, { Dep::Demanding }>,
) -> [Vr<ScratchpadReadReq, { Dep::Demanding }>; SP_BANKS] {
    let (req0, req1, req2, req3) = cmd_mesh_spad
        .map(|spad| {
            let arr = range::<4>().map(|i| {
                let (read_a, read_b, read_d) = spad.signals.spad_reads[i];

                let addr = if read_a {
                    spad.signals.a_address.sp_row()
                } else if read_b {
                    spad.signals.b_address.sp_row()
                } else if read_d {
                    spad.signals.d_address.sp_row()
                } else {
                    0.into_u()
                };
                let spad_req = ScratchpadReadReq { addr: addr.resize(), from_dma: false };

                (read_a || read_b || read_d, spad_req)
            });

            (arr[0], arr[1], arr[2], arr[3])
        })
        .map_resolver_inner(|(r0, r1, r2, r3)| Array::from([r0, r1, r2, r3]))
        .unzip_some();

    let reqs = [req0, req1, req2, req3];
    array_map!(reqs, filter_req)
}

/// Generate accumulator read requests.
fn acc_read_req<const EX_QUEUE_LENGTH: usize>(
    cmd_mesh_acc: I<VrH<MeshControlSignals<EX_QUEUE_LENGTH>, Array<bool, ACC_BANKS>>, { Dep::Demanding }>,
) -> [Vr<AccumulatorReadReq, { Dep::Demanding }>; ACC_BANKS] {
    let (req0, req1) = cmd_mesh_acc
        .map(|acc| {
            let arr = range::<2>().map(|i| {
                let (read_a, read_b, read_d) = acc.signals.acc_reads[i];

                let addr = if read_a {
                    acc.signals.a_address.acc_row()
                } else if read_b {
                    acc.signals.b_address.acc_row()
                } else if read_d {
                    acc.signals.d_address.acc_row()
                } else {
                    0.into_u()
                };

                let acc_req = AccumulatorReadReq {
                    scale: acc.config.acc_scale,
                    full: false,
                    act: acc.config.activation,
                    from_dma: false,
                    addr: addr.resize(),
                };

                (read_a || read_b || read_d, acc_req)
            });

            (arr[0], arr[1])
        })
        .map_resolver_inner(|(r0, r1)| Array::from([r0, r1]))
        .unzip_some();

    let reqs = [req0, req1];
    array_map!(reqs, filter_req)
}

/// Execute the mesh computation.
///
/// This module is responsible for executing "execute"-type ISA commands, such as matrix multiplications.
/// It includes a systolic array for dot-products, and a transposer.
///
/// The execute module is responsible for the following:
/// - Take the command from the Reservation Station (`cmd_raw`)
/// - Decode the command
/// - Read the operands from the SRAM (By using `spad_readers` and `acc_readers`)
/// - Run the mesh(systolic array)
/// - Write the result back to the SRAM (By using `spad_writers` and `acc_writers`)
pub fn execute<const MR: usize, const TR: usize, const MC: usize, const TC: usize, const EX_QUEUE_LENGTH: usize>(
    cmd: Vr<GemminiCmd>,
    spad_readers: impl FnOnce([Vr<ScratchpadReadReq, { Dep::Demanding }>; SP_BANKS]) -> [Vr<ScratchpadReadResp>; SP_BANKS],
    spad_writers: impl FnOnce([Valid<ScratchpadWriteReq>; SP_BANKS]),
    acc_readers: impl FnOnce(
        [Vr<AccumulatorReadReq, { Dep::Demanding }>; ACC_BANKS],
    ) -> [Vr<AccumulatorReadResp>; ACC_BANKS],
    acc_writers: impl FnOnce([Valid<AccumulatorWriteReq>; ACC_BANKS]),
) -> Valid<U<{ clog2(RS_ENTRIES) }>>
where
    [(); clog2(RS_ENTRIES)]:,
    [(); clog2(EX_QUEUE_LENGTH) + 1]:,
    [(); clog2(EX_QUEUE_LENGTH + 1) + 1]:,
{
    // 1. Decode the command
    let (config_cmd, compute_cmd) = cmd.comb(transpose_preload_unroller).comb(cmd_decoder::<EX_QUEUE_LENGTH>);

    // 2. Process the config command.
    //
    // It just return the ROB id of the command. The configuration information was parsed in the decode stage (step 1).
    let config_rob_id = config_cmd.reg_fwd(false).filter_map(|(cmd, _)| {
        if cmd.typ == ExeCmdType::Config {
            cmd.cmd.cmds[0].and_then(|cmd| cmd.rob_id)
        } else {
            None
        }
    });

    // 3. Compute all cofiguruations and signals. Also, wait for finishing fire all rows.
    let compute_cmd = compute_cmd
        .map_resolver_inner::<(TagsInProgress, bool, U<6>)>(|(tags_in_progress, any_pending_rob_ids, _)| {
            (tags_in_progress, any_pending_rob_ids)
        })
        .fsm_egress_with_r::<(bool, MeshControlSignals<EX_QUEUE_LENGTH>), CounterS>(
            CounterS::default(),
            false,
            |(cmd, config), er, counters| {
                let (_, _, sram_read_req_readies) = er;
                let (signals, last, s_next) = compute_control_signals(cmd, config, counters, sram_read_req_readies);
                let ep = (last, MeshControlSignals { cmd, config, counters, signals });

                (ep, s_next, last)
            },
        )
        .fsm_map(false, |(last, mesh_cntl_signals), s_in_prop_flush| {
            let next_in_prop_flush = if mesh_cntl_signals.config.dataflow == Dataflow::OS && last {
                let cmd = mesh_cntl_signals.cmd;
                match cmd.typ {
                    ExeCmdType::Preload => !LocalAddr::from(cmd.cmd.rs2s[0]).is_garbage(),
                    ExeCmdType::ComputeAndPreload => !LocalAddr::from(cmd.cmd.rs2s[1]).is_garbage(),
                    _ => s_in_prop_flush,
                }
            } else {
                s_in_prop_flush
            };

            let ep = (last, MeshControlSignals {
                counters: CounterS {
                    in_prop_flush: if mesh_cntl_signals.cmd.typ == ExeCmdType::Preload
                        || mesh_cntl_signals.cmd.typ == ExeCmdType::Flush
                    {
                        s_in_prop_flush
                    } else {
                        mesh_cntl_signals.signals.prop
                    },
                    ..mesh_cntl_signals.counters
                },
                ..mesh_cntl_signals
            });

            (ep, next_in_prop_flush)
        });

    let (compute_cmd, pending_completed_rob_ids) = compute_cmd
        .map_resolver_inner::<((TagsInProgress, U<6>), bool)>(
            |((tags_in_progress, sram_read_readies), any_pending_rob_ids)| {
                (tags_in_progress, any_pending_rob_ids, sram_read_readies)
            },
        )
        .lfork();

    // 4. Process the pending completed rob ids.
    let pending_completed_rob_ids = pending_completed_rob_ids
        .filter_map(|ip: (bool, MeshControlSignals<EX_QUEUE_LENGTH>)| {
            let (last, mesh_control_signals) = ip;

            if !last {
                None
            } else {
                let cmd = mesh_control_signals.cmd.cmd;
                match mesh_control_signals.cmd.typ {
                    ExeCmdType::Config | ExeCmdType::Flush => None,
                    ExeCmdType::Preload => {
                        let pending_completed_rob_ids_0 = cmd.cmds[0]
                            .and_then(|cmd| cmd.rob_id)
                            .filter(|_| mesh_control_signals.signals.c_address.is_garbage());

                        Some(Array::from([pending_completed_rob_ids_0, None]))
                    }
                    ExeCmdType::Compute => {
                        let pending_completed_rob_ids_0 = cmd.cmds[0].and_then(|cmd| cmd.rob_id);

                        Some(Array::from([pending_completed_rob_ids_0, None]))
                    }
                    ExeCmdType::ComputeAndPreload => {
                        let pending_completed_rob_ids_0 = cmd.cmds[0].and_then(|cmd| cmd.rob_id);
                        let pending_completed_rob_ids_1 = cmd.cmds[1]
                            .and_then(|cmd| cmd.rob_id)
                            .filter(|_| mesh_control_signals.signals.c_address.is_garbage());

                        Some(Array::from([pending_completed_rob_ids_0, pending_completed_rob_ids_1]))
                    }
                }
            }
        })
        .map_resolver(|er| !er.ready)
        .fsm_egress::<HOption<U<{ clog2(RS_ENTRIES) }>>, U<3>>(
            0.into_u(),
            true,
            true,
            |pending_rob_ids: Array<HOption<U<{ clog2(RS_ENTRIES) }>>, 2>, ptr: U<3>| {
                let num_elements =
                    (U::from(pending_rob_ids[0].is_some()) + U::from(pending_rob_ids[1].is_some())).resize::<3>();
                let ptr_next = wrapping_inc::<3>(ptr, 3.into_u());
                let is_last = ptr_next >= num_elements;

                (pending_rob_ids[ptr], ptr_next, is_last)
            },
        )
        .filter_map::<U<{ clog2(RS_ENTRIES) }>>(|p| p)
        .reg_fwd(true);

    // 5. Process the mesh(compute, flush) command.
    //
    // We have to do the following:
    // 1) Return the rob id of the mesh command.
    // 2) Compute with the mesh: read the operands from SRAM -> run the mesh -> write the result back to SRAM.
    let (compute_cmd, write_req_config) = compute_cmd.map(|p| p.1).lfork_uni();
    let (nonflush_compute_cmd, flush_cmd) = compute_cmd.lfork_uni();

    let mesh_flush_req = flush_cmd
        .filter_map(|p| if p.cmd.typ == ExeCmdType::Flush { Some(p) } else { None })
        .discard_into_vr()
        .map(|p| {
            let MeshControlSignals { counters, signals, .. } = p;

            MeshReq {
                dataflow: signals.dataflow,
                propagate_flip: counters.in_prop_flush,
                shift: signals.shift,
                transpose_a: false,
                transpose_bd: false,
                total_rows: BLOCK_SIZE.into_u(),
                tag: MeshTag { rob_id: None, addr: LocalAddr::garbage(), rows: 0.into_u(), cols: 0.into_u() },
                flush: true,
            }
        })
        .map_resolver_inner(|_| ());

    let (cmd_mesh_cntl, cmd_mesh_mem) =
        nonflush_compute_cmd.filter_map(|p| if p.cmd.typ == ExeCmdType::Flush { None } else { Some(p) }).lfork();

    // 6. Read
    let (cmd_mesh_spad, cmd_mesh_acc) = cmd_mesh_mem
        .map_resolver_inner::<(Array<bool, SP_BANKS>, Array<bool, ACC_BANKS>)>(|(er_inner1, er_inner2)| {
            U::from([er_inner1[0], er_inner1[1], er_inner1[2], er_inner1[3], er_inner2[0], er_inner2[1]])
        })
        .lfork();
    let spad_resps = cmd_mesh_spad.comb(spad_read_req).comb(spad_readers);
    let acc_resps = cmd_mesh_acc.comb(acc_read_req).comb(acc_readers);

    // 7. Run Mesh
    let cmd_mesh_cntl = cmd_mesh_cntl
        .map(|mesh_cntl_signals: MeshControlSignals<EX_QUEUE_LENGTH>| {
            let cmd = mesh_cntl_signals.cmd.cmd;
            let signals = mesh_cntl_signals.signals;
            let prop = mesh_cntl_signals.counters.in_prop_flush;

            let rob_id = match mesh_cntl_signals.cmd.typ {
                ExeCmdType::Config | ExeCmdType::Flush | ExeCmdType::Compute => None,
                ExeCmdType::Preload => cmd.cmds[0].and_then(|cmd| cmd.rob_id),
                ExeCmdType::ComputeAndPreload => cmd.cmds[1].and_then(|cmd| cmd.rob_id),
            }.filter(|_| !signals.c_address.is_garbage());
            ControlSignals {
                rob_id,
                prop,
                ..signals
            }
        })
        .filter(|signals| signals.first) // Take only first request
        .fifo::<5>(); // TODO: Use `{ SPAD_READ_DELAY + 1 }` instead of `5`
    let cmd_mesh_cntl: I<VrH<ControlSignals, TagsInProgress>, { Dep::Helpful }> = cmd_mesh_cntl.reg_fwd(true);
    let (mesh_abd, mesh_compute_req) = mesh_inputs(cmd_mesh_cntl, spad_resps, acc_resps);

    let mesh_resp = mesh_with_delays(mesh_abd, [mesh_compute_req, mesh_flush_req].merge().reg_fwd(true));

    // let mesh_resp = mesh_with_delays_wrapper::<0>(mesh_a, mesh_b, mesh_d, mesh_req);
    let (mesh_resp, mesh_resp_rob_id) = mesh_resp.lfork_uni();

    // 8. Process mesh response rob id.
    let mesh_resp_rob_id =
        mesh_resp_rob_id.filter_map(|resp| if resp.last { resp.tag.rob_id } else { None }).discard_into_vr();

    // 9. Write
    let sram_write: Valid<MeshRespExtended> = mesh_resp
        .fsm_map::<MeshRespExtended, (U<4>, HOption<U<{ clog2(RS_ENTRIES) }>>)>(
            (0.into_u(), None),
            |ip, (output_counter, prv_rob_id)| {
                let start_array_outputting = !ip.tag.addr.is_garbage();
                let output_counter_next = match (ip.tag.rob_id, prv_rob_id) {
                    (Some(rob_id), Some(prv_rob_id)) if rob_id == prv_rob_id => {
                        wrapping_inc::<4>(output_counter, 16.into_u())
                    }
                    _ => 0.into_u(),
                };

                (
                    MeshRespExtended { mesh_resp: ip, output_counter: output_counter_next, start_array_outputting },
                    (output_counter_next, ip.tag.rob_id),
                )
            },
        );

    let (spad_write, acc_write) = sram_write.lfork();

    let (write_req_config_spad, write_req_config_acc) = write_req_config
        .generator(None, |ip, _er, _s| Some(ip), |s| s)
        .map(|ip| {
            let dataflow = ip.signals.dataflow;
            let act = ip.config.activation;
            let c_addr_stride = ip.config.c_addr_stride;

            (dataflow, act, c_addr_stride)
        })
        .lfork();

    let spad_write = (spad_write, write_req_config_spad).join_valid();
    let acc_write = (acc_write, write_req_config_acc).join_valid();

    // Scratchpad write
    // <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/ExecuteController.scala#L923-L944>
    let [spad_write0, spad_write1, spad_write2, spad_write3] = spad_write.map_resolver::<Array<(), 4>>(|_| ()).lfork();

    let spad_write: [Valid<ScratchpadWriteReq>; SP_BANKS] = [
        spad_write0.filter_map(|p| spad_write_req(p, 0.into_u())),
        spad_write1.filter_map(|p| spad_write_req(p, 1.into_u())),
        spad_write2.filter_map(|p| spad_write_req(p, 2.into_u())),
        spad_write3.filter_map(|p| spad_write_req(p, 3.into_u())),
    ];

    // Accumulator write
    // <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/ExecuteController.scala#L946-L960>
    let (acc_write0, acc_write1) = acc_write.lfork();

    let acc_write: [Valid<AccumulatorWriteReq>; ACC_BANKS] = [
        acc_write0.filter_map(|p| acc_write_req(p, 0.into_u())),
        acc_write1.filter_map(|p| acc_write_req(p, 1.into_u())),
    ];

    spad_write.comb(spad_writers);
    acc_write.comb(acc_writers);

    // Calculates the result ROB ID.
    [config_rob_id, mesh_resp_rob_id, pending_completed_rob_ids].merge().always_into_valid().into_helpful()
}

/// TODO: Documentation
#[synthesize]
pub fn execute_default(
    cmd_raw: Vr<GemminiCmd>,
    spad_readers: impl FnOnce([Vr<ScratchpadReadReq, { Dep::Demanding }>; SP_BANKS]) -> [Vr<ScratchpadReadResp>; SP_BANKS],
    spad_writers: impl FnOnce([Valid<ScratchpadWriteReq>; SP_BANKS]),
    acc_readers: impl FnOnce(
        [Vr<AccumulatorReadReq, { Dep::Demanding }>; ACC_BANKS],
    ) -> [Vr<AccumulatorReadResp>; ACC_BANKS],
    acc_writers: impl FnOnce([Valid<AccumulatorWriteReq>; ACC_BANKS]),
) -> Valid<U<{ clog2(RS_ENTRIES) }>> {
    execute::<16, 1, 16, 1, 8>(cmd_raw, spad_readers, spad_writers, acc_readers, acc_writers)
}
