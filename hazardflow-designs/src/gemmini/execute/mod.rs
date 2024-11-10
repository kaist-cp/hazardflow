//! Execute controller.

#![allow(clippy::diverging_sub_expression)]

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

/// Gemmini command hazard.
#[derive(Debug, Clone, Copy)]
struct ExeH<const EX_QUEUE_LENGTH: usize>;

impl<const EX_QUEUE_LENGTH: usize> Hazard for ExeH<EX_QUEUE_LENGTH> {
    type P = (ExeCmdT<EX_QUEUE_LENGTH>, ConfigS);
    type R = (U<2>, TagsInProgress, bool);

    // Check data hazard when doing preload command.
    fn ready((cmds, _): Self::P, (_, tags_in_progress, _): Self::R) -> bool {
        match cmds {
            ExeCmdT::Preload(cmd) => {
                let raw_hazard_pre = tags_in_progress.any(|tag| {
                    let pre_raw_haz = tag.addr.is_same_addr(LocalAddr::from(cmd.rs1s[0]));
                    let mul_raw_haz = tag.addr.is_same_addr(LocalAddr::from(cmd.rs1s[1]))
                        || tag.addr.is_same_addr(LocalAddr::from(cmd.rs2s[1]));

                    !tag.addr.is_garbage() && (pre_raw_haz || mul_raw_haz) // && !raw_hazards_are_impossible
                });
                !raw_hazard_pre
            }
            ExeCmdT::PreloadAndCompute(cmd) => {
                let raw_hazard_mulpre = tags_in_progress.any(|tag| {
                    let pre_raw_haz = tag.addr.is_same_addr(LocalAddr::from(cmd.rs1s[1]));
                    let mul_raw_haz = tag.addr.is_same_addr(LocalAddr::from(cmd.rs1s[2]))
                        || tag.addr.is_same_addr(LocalAddr::from(cmd.rs2s[2]));

                    !tag.addr.is_garbage() && (pre_raw_haz || mul_raw_haz) // && !raw_hazards_are_impossible
                });
                !raw_hazard_mulpre
            }
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Operand {
    addr: LocalAddr,
    is_garbage: bool,
    start_inputting: bool,
    counter: U<4>,
    started: bool,
    priority: U<2>,
}

/// Configuration state.
#[derive(Debug, Clone, Copy, Default)]
struct ConfigS {
    current_dataflow: Dataflow,

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

    a_addr_offset: U<{ 16 + clog2(BLOCK_SIZE) }>,
}

/// Decoded command.
#[derive(Debug, Clone, Copy)]
struct CmdDecoded<const EX_QUEUE_LENGTH: usize> {
    cmds: Array<HOption<GemminiCmd>, EX_QUEUE_LENGTH>,
    rs1s: Array<HOption<U<64>>, EX_QUEUE_LENGTH>,
    rs2s: Array<HOption<U<64>>, EX_QUEUE_LENGTH>,

    do_config: bool,
    do_computes: Array<bool, EX_QUEUE_LENGTH>,
    do_preloads: Array<bool, EX_QUEUE_LENGTH>,

    in_prop: bool,
}

/// Cmd types.
#[derive(Debug, Clone, Copy)]
enum ExeCmdT<const EX_QUEUE_LENGTH: usize> {
    Config(CmdDecoded<EX_QUEUE_LENGTH>),
    Preload(CmdDecoded<EX_QUEUE_LENGTH>),
    Compute(CmdDecoded<EX_QUEUE_LENGTH>),
    PreloadAndCompute(CmdDecoded<EX_QUEUE_LENGTH>),
    Flush(CmdDecoded<EX_QUEUE_LENGTH>),
}

impl<const EX_QUEUE_LENGTH: usize> ExeCmdT<EX_QUEUE_LENGTH> {
    fn gemmini_cmds(self) -> Array<HOption<GemminiCmd>, EX_QUEUE_LENGTH> {
        match self {
            ExeCmdT::Config(cmd)
            | ExeCmdT::Preload(cmd)
            | ExeCmdT::Compute(cmd)
            | ExeCmdT::PreloadAndCompute(cmd)
            | ExeCmdT::Flush(cmd) => cmd.cmds,
        }
    }
}

/// Extended ComputeControlSignals.
#[derive(Debug, Clone, Copy)]
struct ControlSignals {
    perform_single_mul: bool,
    perform_single_preload: bool,

    a_bank: U<{ clog2(SP_BANKS) }>,
    b_bank: U<{ clog2(SP_BANKS) }>,
    d_bank: U<{ clog2(SP_BANKS) }>,

    a_bank_acc: U<{ clog2(ACC_BANKS) }>,
    b_bank_acc: U<{ clog2(ACC_BANKS) }>,
    d_bank_acc: U<{ clog2(ACC_BANKS) }>,

    a_read_from_acc: bool,
    b_read_from_acc: bool,
    d_read_from_acc: bool,

    a_garbage: bool,
    b_garbage: bool,
    d_garbage: bool,

    a_address: LocalAddr,
    b_address: LocalAddr,
    d_address: LocalAddr,

    a_address_rs1: LocalAddr,
    b_address_rs2: LocalAddr,
    d_address_rs1: LocalAddr,
    c_address_rs2: LocalAddr,

    a_unpadded_cols: U<{ clog2(BLOCK_SIZE + 1) }>,
    b_unpadded_cols: U<{ clog2(BLOCK_SIZE + 1) }>,
    d_unpadded_cols: U<{ clog2(BLOCK_SIZE + 1) }>,

    a_fire: bool,
    b_fire: bool,
    d_fire: bool,

    spad_reads: Array<(bool, bool, bool), SP_BANKS>,
    acc_reads: Array<(bool, bool, bool), ACC_BANKS>,

    a_should_be_fed_into_transposer: bool,
    b_should_be_fed_into_transposer: bool,

    accumulate_zeros: bool,
    preload_zeros: bool,

    start_inputting_a: bool,
    start_inputting_b: bool,
    start_inputting_d: bool,

    c_addr: LocalAddr,
    c_rows: U<{ clog2(BLOCK_SIZE + 1) }>,
    c_cols: U<{ clog2(BLOCK_SIZE + 1) }>,

    transpose_a: bool,
    transpose_bd: bool,

    total_rows: U<5>,

    rob_id: HOption<U<{ clog2(RS_ENTRIES) }>>,

    dataflow: Dataflow,
    prop: bool,
    shift: U<5>,

    first: bool,

    flush: U<2>,
}

/// Package of whole configs and signals.
#[derive(Debug, Clone, Copy)]
struct MeshControlSignals<const EX_QUEUE_LENGTH: usize> {
    cmd_decoded: ExeCmdT<EX_QUEUE_LENGTH>,
    cfg: ConfigS,
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

    let do_config = functs[0].is_some_and(|f| matches!(f, Funct::ConfigCmd));
    let do_computes =
        functs.map(|f| f.is_some_and(|f| matches!(f, Funct::ComputeAndFlipCmd | Funct::ComputeAndStayCmd)));
    let do_preloads = functs.map(|f| f.is_some_and(|f| matches!(f, Funct::PreloadCmd)));
    let in_prop = functs[0].is_some_and(|f| matches!(f, Funct::ComputeAndFlipCmd));

    let rs1s = cmds.map(|p| p.map(|p| p.cmd.rs1));
    let rs2s = cmds.map(|p| p.map(|p| p.cmd.rs2));

    CmdDecoded { cmds, rs1s, rs2s, do_config, do_computes, do_preloads, in_prop }
}

fn update_ex_config<const EX_QUEUE_LENGTH: usize>(cmd: ExeCmdT<EX_QUEUE_LENGTH>, config: ConfigS) -> ConfigS {
    if let ExeCmdT::Config(cmds) = cmd {
        // Default mode is FP32 -> acc_scale should be 32.
        let config_ex_rs1 = ConfigExRs1::<32>::new(cmds.rs1s[0]);
        let config_ex_rs2 = ConfigExRs2::new(cmds.rs2s[0]);

        if !matches!(config_ex_rs1.cmd_type, ConfigCmd::Ex) {
            return config;
        }

        // next states.
        let in_shift = config_ex_rs2.in_shift.clip_const::<5>(0);
        let acc_scale = config_ex_rs1.acc_scale;
        let activation = config_ex_rs1.activation.resize::<3>();
        let transpose_a = config_ex_rs1.transpose_a;
        let transpose_bd = config_ex_rs1.transpose_bd;
        let current_dataflow = config_ex_rs1.dataflow;

        let a_addr_stride = config_ex_rs1.a_stride;
        let c_addr_stride = config_ex_rs2.c_stride;

        let s_next = if !config_ex_rs1.set_only_strides {
            ConfigS { in_shift, acc_scale, activation, transpose_a, transpose_bd, current_dataflow, ..config }
        } else {
            config
        };

        ConfigS { a_addr_stride, c_addr_stride, ..s_next }
    } else {
        config
    }
}

fn wrap_cmd_type<const EX_QUEUE_LENGTH: usize>(
    cmd_decoded: CmdDecoded<EX_QUEUE_LENGTH>,
    matmul_in_progress: bool,
    any_pending_robs: bool,
    dataflow: Dataflow,
) -> HOption<ExeCmdT<EX_QUEUE_LENGTH>> {
    if cmd_decoded.cmds[0].is_some() {
        if cmd_decoded.do_config && !matmul_in_progress && !any_pending_robs {
            Some(ExeCmdT::Config(cmd_decoded))
        } else if cmd_decoded.do_preloads[0] && cmd_decoded.cmds[1].is_some() {
            Some(ExeCmdT::Preload(cmd_decoded))
        } else if cmd_decoded.do_computes[0]
            && cmd_decoded.do_preloads[1]
            && cmd_decoded.cmds[1].is_some()
            && cmd_decoded.cmds[2].is_some()
        {
            Some(ExeCmdT::PreloadAndCompute(cmd_decoded))
        } else if cmd_decoded.do_computes[0] {
            Some(ExeCmdT::Compute(cmd_decoded))
        } else if matmul_in_progress && (matches!(dataflow, Dataflow::OS) || cmd_decoded.do_config) {
            Some(ExeCmdT::Flush(cmd_decoded))
        } else {
            None
        }
    } else if matmul_in_progress && matches!(dataflow, Dataflow::OS) {
        Some(ExeCmdT::Flush(cmd_decoded))
    } else {
        None
    }
}

/// Decode the command from the reservation station.
#[allow(clippy::type_complexity)]
fn cmd_decoder<const EX_QUEUE_LENGTH: usize>(
    cmd: Vr<GemminiCmd>,
) -> (
    Vr<(ExeCmdT<EX_QUEUE_LENGTH>, ConfigS)>,
    I<VrH<(ExeCmdT<EX_QUEUE_LENGTH>, ConfigS), (U<2>, TagsInProgress, bool)>, { Dep::Helpful }>,
)
where
    [(); clog2(EX_QUEUE_LENGTH) + 1]:,
    [(); clog2(EX_QUEUE_LENGTH + 1) + 1]:,
{
    let cmd = cmd
        .map_resolver_inner::<((), FifoS<GemminiCmd, EX_QUEUE_LENGTH>)>(|_| ())
        .multi_headed_transparent_fifo()
        .map(|fifo_s| {
            // Transforms `FifoS` into array of commands
            range::<EX_QUEUE_LENGTH>().map(|i| {
                if i.resize() < fifo_s.len {
                    let idx = wrapping_add::<{ clog2(EX_QUEUE_LENGTH) }>(fifo_s.raddr, i, EX_QUEUE_LENGTH.into_u());
                    Some(fifo_s.inner[idx])
                } else {
                    None
                }
            })
        })
        .map_resolver_drop(|er: Ready<((U<2>, bool, bool), ConfigS)>| {
            let pop_count: U<2> = er.inner.0 .0;
            Ready::valid(((), pop_count.resize()))
        })
        .fsm_ingress::<HOption<ExeCmdT<EX_QUEUE_LENGTH>>>(None, |ip, er, _s| {
            let ((_pop_count, any_matmul_in_progress, any_pending_robs), config) = er;

            let cmd_decoded = decode_cmd::<EX_QUEUE_LENGTH>(ip);
            let cmd_wrapped =
                wrap_cmd_type(cmd_decoded, any_matmul_in_progress, any_pending_robs, config.current_dataflow);

            (cmd_wrapped, cmd_wrapped.is_some())
        })
        .transparent_fsm_map::<(ExeCmdT<EX_QUEUE_LENGTH>, ConfigS)>(ConfigS::default(), |cmd, s_config| {
            // Update the configuration state if the command is a `ex_config` command.
            let cmd = cmd.unwrap();
            let config_updated = update_ex_config(cmd, s_config);

            ((cmd, config_updated), config_updated)
        })
        .fsm_filter_map::<(ExeCmdT<EX_QUEUE_LENGTH>, ConfigS), HOption<ExeCmdT<EX_QUEUE_LENGTH>>>(None, |ip, s| {
            let ep = if let Some(prev_cmd) = s {
                let ip_cmd = ip.0.gemmini_cmds();
                let prev_cmd = prev_cmd.gemmini_cmds();

                if ip_cmd.zip(prev_cmd).all(|(ip_cmd, prev_cmd)| match (ip_cmd, prev_cmd) {
                    (Some(ip_cmd), Some(prev_cmd)) => ip_cmd.rob_id == prev_cmd.rob_id,
                    (None, None) => true,
                    _ => false,
                }) {
                    None
                } else if ip_cmd.any(|ip_cmd| ip_cmd.is_some_and(|cmd| cmd.rob_id.is_some()))
                    || matches!(ip.0, ExeCmdT::Flush(_))
                {
                    Some(ip)
                } else {
                    None
                }
            } else {
                Some(ip)
            };

            (ep, Some(ip.0))
        })
        .map_resolver_inner_with_p(|ip, er| {
            let (pop_count, any_matmul_in_progress, any_pending_robs) = er;

            if let Some((cmd, _config)) = ip {
                match cmd {
                    ExeCmdT::Config(_) => (1.into_u(), any_matmul_in_progress, any_pending_robs),
                    _ => (pop_count, any_matmul_in_progress, any_pending_robs),
                }
            } else {
                (pop_count, any_matmul_in_progress, any_pending_robs)
            }
        });

    cmd.map_resolver_inner::<((U<2>, TagsInProgress, bool), _)>(|er| {
        let ((pop_count, tags_in_progress, any_pending_robs), _) = er;
        let any_matmul_in_progress = tags_in_progress.any(|mesh_tag| mesh_tag.rob_id.is_some());

        (pop_count, any_matmul_in_progress, any_pending_robs)
    })
    .transparent_reg_fwd::<ExeH<EX_QUEUE_LENGTH>>(true)
    .filter_map_drop_with_r::<VrH<((ExeCmdT<EX_QUEUE_LENGTH>, ConfigS), BoundedU<2>), (U<2>, TagsInProgress, bool)>>(
        |(cmd_decoded, config), _| match cmd_decoded {
            ExeCmdT::Config(_) => Some(((cmd_decoded, config), BoundedU::new(0.into_u()))),
            _ => Some(((cmd_decoded, config), BoundedU::new(1.into_u()))),
        },
    )
    .reg_fwd(true)
    .map_resolver_inner(|(_, er1)| er1)
    .branch()
}

/// generate inputs for mesh_with_delays. a, b, d, req.
/// unsafe?
#[allow(clippy::type_complexity)]
fn generate_mesh_inputs<const TN: usize, const MN: usize>(
    cntl: I<VrH<ControlSignals, TagsInProgress>, { Dep::Helpful }>,
    spad_resps: [Vr<ScratchpadReadResp>; SP_BANKS],
    acc_resps: [Vr<AccumulatorReadResp>; ACC_BANKS],
) -> (Vr<A>, Vr<B>, Vr<D>, I<VrH<MeshReq, TagsInProgress>, { Dep::Helpful }>) {
    // # Safety
    // - Dependency types of the egress interfaces are all `Helpful`.
    // - All egress payloads don't depend on egress resolver.
    unsafe {
        // TODO: Do not use magic number.
        (cntl, spad_resps, acc_resps)
            .fsm::<(Vr<A>, Vr<B>, Vr<D>, I<VrH<MeshReq, TagsInProgress>, { Dep::Helpful }>), _>(
                (),
                |(cntl, spad_resps, acc_resps), er, ()| {
                    let spad_data = spad_resps.map(|v| v.map(|e| e.data));
                    let acc_data = acc_resps.map(|v| v.map(|e| e.data));

                    let spad_valid = spad_resps.map(|v| v.is_some_and(|e| !e.from_dma));
                    let acc_valid = acc_resps.map(|v| v.is_some_and(|e| !e.from_dma));

                    let data_a_valid = cntl.is_some_and(|cntl| {
                        let data_a_valid = cntl.a_garbage
                            || cntl.a_unpadded_cols == 0.into_u()
                            || if cntl.a_read_from_acc { acc_valid[cntl.a_bank_acc] } else { spad_valid[cntl.a_bank] };

                        cntl.a_fire && data_a_valid
                    });
                    let data_b_valid = cntl.is_some_and(|cntl| {
                        let data_b_valid = cntl.b_garbage
                            || cntl.b_unpadded_cols == 0.into_u()
                            || if cntl.accumulate_zeros {
                                false
                            } else if cntl.b_read_from_acc {
                                acc_valid[cntl.b_bank_acc]
                            } else {
                                spad_valid[cntl.b_bank]
                            };

                        cntl.b_fire && data_b_valid
                    });
                    let data_d_valid = cntl.is_some_and(|cntl| {
                        let data_d_valid = cntl.d_garbage
                            || cntl.d_unpadded_cols == 0.into_u()
                            || if cntl.preload_zeros {
                                false
                            } else if cntl.d_read_from_acc {
                                acc_valid[cntl.d_bank_acc]
                            } else {
                                spad_valid[cntl.d_bank]
                            };

                        cntl.d_fire && data_d_valid
                    });

                    let data_a_unpadded = cntl.and_then(|cntl| {
                        if cntl.a_read_from_acc {
                            acc_data[cntl.a_bank_acc]
                        } else {
                            spad_data[cntl.a_bank]
                        }
                    });
                    let data_b_unpadded = cntl.and_then(|cntl| {
                        if cntl.accumulate_zeros {
                            Some(0.into_u())
                        } else if cntl.b_read_from_acc {
                            acc_data[cntl.b_bank_acc]
                        } else {
                            spad_data[cntl.b_bank]
                        }
                    });
                    let data_d_unpadded = cntl.and_then(|cntl| {
                        if cntl.preload_zeros {
                            Some(0.into_u())
                        } else if cntl.d_read_from_acc {
                            acc_data[cntl.d_bank_acc]
                        } else {
                            spad_data[cntl.d_bank]
                        }
                    });

                    let data_a = data_a_unpadded.and_then(|data| {
                        let data = data.chunk::<8>().enumerate().map(|(idx, value)| {
                            if cntl.is_some_and(|cntl| idx.resize() < cntl.a_unpadded_cols) {
                                value
                            } else {
                                0.into_u::<8>()
                            }
                        });
                        Some(data.map(|v| v.chunk::<8>()))
                    });

                    let data_b = data_b_unpadded.and_then(|data| {
                        let data = data.chunk::<8>().enumerate().map(|(idx, value)| {
                            if cntl.is_some_and(|cntl| idx.resize() < cntl.b_unpadded_cols) {
                                value
                            } else {
                                0.into_u::<8>()
                            }
                        });
                        Some(data.map(|v| v.chunk::<8>()))
                    });

                    let data_d = data_d_unpadded.and_then(|data| {
                        let data = data.chunk::<8>().enumerate().map(|(idx, value)| {
                            if cntl.is_some_and(|cntl| idx.resize() < cntl.d_unpadded_cols) {
                                value
                            } else {
                                0.into_u::<8>()
                            }
                        });
                        Some(data.map(|v| v.chunk::<8>()))
                    });

                    let (data_a, data_b) = cntl.map_or((data_a, data_b), |cntl| {
                        let all_zeros = Array::<Array<U<8>, 1>, 16>::default();

                        if cntl.perform_single_preload {
                            let data_a = if cntl.a_should_be_fed_into_transposer { data_a } else { Some(all_zeros) };
                            let data_b = if cntl.b_should_be_fed_into_transposer { data_b } else { Some(all_zeros) };

                            (data_a, data_b)
                        } else if cntl.perform_single_mul {
                            let data_a = if cntl.a_should_be_fed_into_transposer { Some(all_zeros) } else { data_a };
                            let data_b = if cntl.b_should_be_fed_into_transposer { Some(all_zeros) } else { data_b };

                            (data_a, data_b)
                        } else {
                            (data_a, data_b)
                        }
                    });

                    // If flush is true, data_d should be invalidated.
                    let (data_a, data_b, data_d) = if cntl.is_some_and(|cntl| cntl.flush == 1.into_u()) {
                        (None, None, None)
                    } else {
                        (data_a, data_b, data_d)
                    };

                    let req = cntl.map(|cntl| MeshReq {
                        total_rows: cntl.total_rows,
                        tag: MeshTag {
                            rob_id: cntl.rob_id,
                            addr: if cntl.perform_single_mul { cntl.c_addr.make_this_garbage() } else { cntl.c_addr },
                            rows: cntl.c_rows,
                            cols: cntl.c_cols,
                        },
                        pe_control: PeControl {
                            dataflow: cntl.dataflow,
                            propagate: if cntl.prop { Propagate::Reg1 } else { Propagate::Reg2 },
                            shift: cntl.shift,
                        },
                        transpose_a: cntl.transpose_a,
                        transpose_bd: cntl.transpose_bd,
                        flush: cntl.flush,
                    });

                    // Validity of inputs.
                    let data_a = if data_a_valid { data_a } else { None };
                    let data_b = if data_b_valid { data_b } else { None };
                    let data_d = if data_d_valid { data_d } else { None };

                    // Fire?
                    let mesh_a_fire = data_a.is_some() && er.0.ready;
                    let mesh_b_fire = data_b.is_some() && er.1.ready;
                    let mesh_d_fire = data_d.is_some() && er.2.ready;

                    let spad_readies = Ready::<()>::invalid().repeat::<4>(); // SP_BANKS == 4
                    let acc_readies = Ready::<()>::invalid().repeat::<2>(); // ACC_BANKS == 2

                    let fifo_ready = Ready::new(
                        cntl.is_some_and(|cntl| {
                            (!cntl.a_fire || mesh_a_fire || !er.0.ready)
                                && (!cntl.b_fire || mesh_b_fire || !er.1.ready)
                                && (!cntl.d_fire || mesh_d_fire || !er.2.ready)
                                && (!cntl.first || er.3.ready)
                        }),
                        er.3.inner,
                    );
                    let fifo_fire = cntl.is_some() && fifo_ready.ready;

                    let req = if (cntl.is_some_and(|cntl| cntl.a_fire || cntl.b_fire || cntl.d_fire) && fifo_fire)
                        || cntl.is_some_and(|cntl| cntl.flush != 0.into_u())
                    {
                        req
                    } else {
                        None
                    };

                    let (spad_readies, acc_readies) = if let (Some(cntl), true) = (cntl, fifo_fire) {
                        let (spad_readies, acc_readies) =
                            if cntl.a_fire && mesh_a_fire && !cntl.a_garbage && cntl.a_unpadded_cols > 0.into_u() {
                                if cntl.a_read_from_acc {
                                    let acc_readies = acc_readies.set(
                                        cntl.a_bank_acc,
                                        Ready::new(!acc_resps[cntl.a_bank_acc].is_some_and(|v| v.from_dma), ()),
                                    );
                                    (spad_readies, acc_readies)
                                } else {
                                    let spad_readies = spad_readies.set(
                                        cntl.a_bank,
                                        Ready::new(!spad_resps[cntl.a_bank].is_some_and(|v| v.from_dma), ()),
                                    );
                                    (spad_readies, acc_readies)
                                }
                            } else {
                                (spad_readies, acc_readies)
                            };

                        let (spad_readies, acc_readies) = if cntl.b_fire
                            && mesh_b_fire
                            && !cntl.b_garbage
                            && !cntl.accumulate_zeros
                            && cntl.b_unpadded_cols > 0.into_u()
                        {
                            if cntl.b_read_from_acc {
                                let acc_readies = acc_readies.set(
                                    cntl.b_bank_acc,
                                    Ready::new(!acc_resps[cntl.b_bank_acc].is_some_and(|v| v.from_dma), ()),
                                );
                                (spad_readies, acc_readies)
                            } else {
                                let spad_readies = spad_readies.set(
                                    cntl.b_bank,
                                    Ready::new(!spad_resps[cntl.b_bank].is_some_and(|v| v.from_dma), ()),
                                );
                                (spad_readies, acc_readies)
                            }
                        } else {
                            (spad_readies, acc_readies)
                        };

                        let (spad_readies, acc_readies) = if cntl.d_fire
                            && mesh_d_fire
                            && !cntl.d_garbage
                            && !cntl.preload_zeros
                            && cntl.d_unpadded_cols > 0.into_u()
                        {
                            if cntl.d_read_from_acc {
                                let acc_readies = acc_readies.set(
                                    cntl.d_bank_acc,
                                    Ready::new(!acc_resps[cntl.d_bank_acc].is_some_and(|v| v.from_dma), ()),
                                );
                                (spad_readies, acc_readies)
                            } else {
                                let spad_readies = spad_readies.set(
                                    cntl.d_bank,
                                    Ready::new(!spad_resps[cntl.d_bank].is_some_and(|v| v.from_dma), ()),
                                );
                                (spad_readies, acc_readies)
                            }
                        } else {
                            (spad_readies, acc_readies)
                        };

                        (spad_readies, acc_readies)
                    } else {
                        (spad_readies, acc_readies)
                    };

                    ((data_a, data_b, data_d, req), (fifo_ready, spad_readies, acc_readies), ())
                },
            )
    }
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

    let w_address = if matches!(dataflow, Dataflow::WS) {
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

    let write_this_row = if matches!(dataflow, Dataflow::WS) {
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
            let e_clipped = clip_with_saturation(e); // Lower 8 bits
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

    let wdata = resp.mesh_resp.data.map(|v| U::from(S::from(v).sext::<32>()));
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

/// Compute control signals.
fn compute_control_signals<const EX_QUEUE_LENGTH: usize>(
    cmd_w_type: ExeCmdT<EX_QUEUE_LENGTH>,
    cfg: ConfigS,
    counters: CounterS,
    sram_read_req_readies: Array<bool, 6>,
) -> ControlSignals {
    let cmd = match cmd_w_type {
        ExeCmdT::Compute(cmd) | ExeCmdT::PreloadAndCompute(cmd) | ExeCmdT::Preload(cmd) | ExeCmdT::Flush(cmd) => cmd,
        ExeCmdT::Config(_) => hpanic!("Config command is not allowed here."),
    };

    let rs1s = cmd.rs1s;
    let rs2s = cmd.rs2s;

    // Compute wires.
    let preload_cmd_place: U<2> = if cmd.do_preloads[0] { 0.into_u() } else { 1.into_u() };

    let a_should_be_fed_into_transposer =
        if matches!(cfg.current_dataflow, Dataflow::OS) { !cfg.transpose_a } else { cfg.transpose_a };
    let a_address_place: U<2> = if preload_cmd_place == 0.into_u() {
        1.into_u()
    } else if a_should_be_fed_into_transposer {
        2.into_u()
    } else {
        0.into_u()
    };

    let b_should_be_fed_into_transposer = matches!(cfg.current_dataflow, Dataflow::OS) && cfg.transpose_bd;
    let b_address_place: U<2> = if preload_cmd_place == 0.into_u() {
        1.into_u()
    } else if b_should_be_fed_into_transposer {
        2.into_u()
    } else {
        0.into_u()
    };

    let d_should_be_fed_into_transposer = matches!(cfg.current_dataflow, Dataflow::WS) && cfg.transpose_bd;

    // SRAM addresses of matmul operands
    let a_address_rs1 = LocalAddr::from(rs1s[a_address_place]);
    let b_address_rs2 = LocalAddr::from(rs2s[b_address_place]);
    let d_address_rs1 = LocalAddr::from(rs1s[preload_cmd_place]);
    let c_address_rs2 = LocalAddr::from(rs2s[preload_cmd_place]);

    let multiply_garbage = a_address_rs1.is_garbage();
    let accumulate_zeros = b_address_rs2.is_garbage();
    let preload_zeros = d_address_rs1.is_garbage();

    let a_cols_default = rs1s[a_address_place].map(|p| p.clip_const::<{ clog2(BLOCK_SIZE + 1) }>(32));
    let a_rows_default = rs1s[a_address_place].map(|p| p.clip_const::<{ clog2(BLOCK_SIZE + 1) }>(48));
    let b_cols_default = rs2s[b_address_place].map(|p| p.clip_const::<{ clog2(BLOCK_SIZE + 1) }>(32));
    let b_rows_default = rs2s[b_address_place].map(|p| p.clip_const::<{ clog2(BLOCK_SIZE + 1) }>(48));
    let d_cols_default = rs1s[preload_cmd_place].map(|p| p.clip_const::<{ clog2(BLOCK_SIZE + 1) }>(32));
    let d_rows_default = rs1s[preload_cmd_place].map(|p| p.clip_const::<{ clog2(BLOCK_SIZE + 1) }>(48));

    let a_cols = if cfg.transpose_a { a_rows_default } else { a_cols_default };
    let a_rows = if cfg.transpose_a { a_cols_default } else { a_rows_default };
    let b_cols =
        if matches!(cfg.current_dataflow, Dataflow::OS) && cfg.transpose_bd { b_rows_default } else { b_cols_default };
    let b_rows =
        if matches!(cfg.current_dataflow, Dataflow::OS) && cfg.transpose_bd { b_cols_default } else { b_rows_default };
    let d_cols =
        if matches!(cfg.current_dataflow, Dataflow::WS) && cfg.transpose_bd { d_rows_default } else { d_cols_default };
    let d_rows =
        if matches!(cfg.current_dataflow, Dataflow::WS) && cfg.transpose_bd { d_cols_default } else { d_rows_default };
    let c_cols = rs2s[preload_cmd_place].map(|p| p.clip_const::<{ clog2(BLOCK_SIZE + 1) }>(32));
    let c_rows = rs2s[preload_cmd_place].map(|p| p.clip_const::<{ clog2(BLOCK_SIZE + 1) }>(48));

    let (start_inputting_a, start_inputting_b, start_inputting_d) = match cmd_w_type {
        ExeCmdT::Config(_) => hpanic!("Config command is not allowed here."),
        ExeCmdT::Preload(_) => (a_should_be_fed_into_transposer, b_should_be_fed_into_transposer, true),
        ExeCmdT::PreloadAndCompute(_) => (true, true, true),
        ExeCmdT::Compute(_) => (!a_should_be_fed_into_transposer, !b_should_be_fed_into_transposer, false),
        ExeCmdT::Flush(_) => (false, false, false),
    };

    let a_garbage = a_address_rs1.is_garbage() || !start_inputting_a;
    let b_garbage = b_address_rs2.is_garbage() || !start_inputting_b;
    let d_garbage = d_address_rs1.is_garbage() || !start_inputting_d;

    let total_rows: U<5> = if matches!(cfg.current_dataflow, Dataflow::WS)
        && d_garbage
        && !a_should_be_fed_into_transposer
        && !b_should_be_fed_into_transposer
        && !d_should_be_fed_into_transposer
    {
        let rows_a: U<5> = if a_garbage { 1.into_u() } else { a_rows.unwrap() };
        let rows_b: U<5> = if b_garbage { 1.into_u() } else { b_rows.unwrap() };

        let total_rows: U<5> = if rows_a < rows_b { rows_b } else { rows_a };
        let total_rows: U<5> = if total_rows < 4.into_u() { 4.into_u() } else { total_rows };

        total_rows
    } else {
        BLOCK_SIZE.into_u()
    };

    let a_address =
        LocalAddr { data: (a_address_rs1.data + counters.a_addr_offset.resize()).resize(), ..a_address_rs1 };
    let b_address =
        LocalAddr { data: (b_address_rs2.data + counters.b_fire_counter.resize()).resize(), ..b_address_rs2 };
    let d_address = LocalAddr {
        data: (d_address_rs1.data + (15.into_u() - counters.d_fire_counter).resize()).resize(),
        ..d_address_rs1
    };

    let data_a_bank = a_address.sp_bank();
    let data_b_bank = b_address.sp_bank();
    let data_d_bank = d_address.sp_bank();

    let data_a_bank_acc = a_address.acc_bank();
    let data_b_bank_acc = b_address.acc_bank();
    let data_d_bank_acc = d_address.acc_bank();

    let a_read_from_acc = a_address_rs1.is_acc_addr;
    let b_read_from_acc = b_address_rs2.is_acc_addr;
    let d_read_from_acc = d_address_rs1.is_acc_addr;

    let a_row_is_not_all_zeros = counters.a_fire_counter.resize() < a_rows.unwrap_or(0.into_u());
    let b_row_is_not_all_zeros = counters.b_fire_counter.resize() < b_rows.unwrap_or(0.into_u());
    let d_row_is_not_all_zeros =
        (BLOCK_SIZE.into_u() - 1.into_u() - counters.d_fire_counter).resize() < d_rows.unwrap_or(0.into_u());

    let a_operand = Operand {
        addr: a_address,
        is_garbage: a_address_rs1.is_garbage(),
        start_inputting: start_inputting_a,
        counter: counters.a_fire_counter,
        started: counters.a_fire_started,
        priority: 0.into_u(),
    };
    let b_operand = Operand {
        addr: b_address,
        is_garbage: b_address_rs2.is_garbage(),
        start_inputting: start_inputting_b,
        counter: counters.b_fire_counter,
        started: counters.b_fire_started,
        priority: 1.into_u(),
    };
    let d_operand = Operand {
        addr: d_address,
        is_garbage: d_address_rs1.is_garbage(),
        start_inputting: start_inputting_d,
        counter: counters.d_fire_counter,
        started: counters.d_fire_started,
        priority: 2.into_u(),
    };
    let operands = Array::from([a_operand, b_operand, d_operand]);
    // a_valid, b_valid, d_valid
    // <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/ExecuteController.scala#L341-L357>
    let valids = operands.map(|op| {
        let others = operands.map(|counters| if counters.priority != op.priority { Some(counters) } else { None });

        let same_banks = others.map(|other| {
            other.is_some_and(|other| {
                let addr1_read_from_acc = op.addr.is_acc_addr;
                let addr2_read_from_acc = other.addr.is_acc_addr;

                let is_garbage = op.is_garbage || other.is_garbage || op.start_inputting || other.start_inputting;

                !is_garbage
                    && ((addr1_read_from_acc && addr2_read_from_acc)
                        || (!addr1_read_from_acc && !addr2_read_from_acc && op.addr.sp_bank() == other.addr.sp_bank()))
            })
        });
        let same_counter = others.map(|o| o.is_some_and(|o| o.started == op.started && o.counter == op.counter));
        let one_ahead =
            others.map(|o| o.is_some_and(|o| op.started && (op.counter == wrapping_inc::<4>(o.counter, 4.into_u())))); // TODO: bit width recheck, total_rows == 4?
        let higher_priorities = others.map(|o| o.is_some_and(|o| o.priority < op.priority));
        let zipped = (same_banks.zip(same_counter)).zip(one_ahead.zip(higher_priorities));
        let must_wait_for = others
            .zip(zipped)
            .map(|x| {
                x.0.is_some_and(|_| {
                    let ((sb, sc), (oa, hp)) = x.1;

                    (sb && hp && sc) || oa
                })
            })
            .any(|is_wait| is_wait);

        !must_wait_for
    });
    let (a_valid, b_valid, d_valid) = (valids[0], valids[1], valids[2]);

    let (perform_single_mul, perform_single_preload) = match cmd_w_type {
        ExeCmdT::PreloadAndCompute(_) => (false, false),
        ExeCmdT::Compute(_) => (true, false),
        ExeCmdT::Preload(_) => (false, true),
        _ => (false, false),
    };

    let spad_reads = range::<4>().map(|bank_i| {
        let read_a = a_valid
            && !a_read_from_acc
            && (data_a_bank == bank_i)
            && start_inputting_a
            && !multiply_garbage
            && a_row_is_not_all_zeros;
        let read_b = b_valid
            && !b_read_from_acc
            && (data_b_bank == bank_i)
            && start_inputting_b
            && !accumulate_zeros
            && b_row_is_not_all_zeros;
        let read_d = d_valid
            && !d_read_from_acc
            && (data_d_bank == bank_i)
            && start_inputting_d
            && !preload_zeros
            && d_row_is_not_all_zeros;

        (read_a, read_b, read_d)
    });
    let acc_reads = range::<2>().map(|bank_i| {
        let read_a = a_valid
            && a_read_from_acc
            && (data_a_bank_acc.resize() == bank_i)
            && start_inputting_a
            && !multiply_garbage
            && a_row_is_not_all_zeros;
        let read_b = b_valid
            && b_read_from_acc
            && (data_b_bank_acc.resize() == bank_i)
            && start_inputting_b
            && !accumulate_zeros
            && b_row_is_not_all_zeros;
        let read_d = d_valid
            && d_read_from_acc
            && (data_d_bank_acc.resize() == bank_i)
            && start_inputting_d
            && !preload_zeros
            && d_row_is_not_all_zeros;

        (read_a, read_b, read_d)
    });

    let a_ready = !((spad_reads
        .enumerate()
        .map(|(idx, (read_a, _read_b, _read_d))| read_a && !sram_read_req_readies[idx])
        .any(|x| x))
        || (acc_reads
            .enumerate()
            .map(|(idx, (read_a, _read_b, _read_d))| read_a && !sram_read_req_readies[idx + U::from(4)])
            .any(|x| x)));
    let b_ready = !((spad_reads
        .enumerate()
        .map(|(idx, (_read_a, read_b, _read_d))| read_b && !sram_read_req_readies[idx])
        .any(|x| x))
        || (acc_reads
            .enumerate()
            .map(|(idx, (_read_a, read_b, _read_d))| read_b && !sram_read_req_readies[idx + U::from(4)])
            .any(|x| x)));
    let d_ready = !((spad_reads
        .enumerate()
        .map(|(idx, (_read_a, _read_b, read_d))| read_d && !sram_read_req_readies[idx])
        .any(|x| x))
        || (acc_reads
            .enumerate()
            .map(|(idx, (_read_a, _read_b, read_d))| read_d && !sram_read_req_readies[idx + U::from(4)])
            .any(|x| x)));

    ControlSignals {
        perform_single_mul,
        perform_single_preload,

        a_address,
        b_address,
        d_address,

        a_address_rs1,
        b_address_rs2,
        d_address_rs1,
        c_address_rs2,

        a_bank: data_a_bank,
        b_bank: data_b_bank,
        d_bank: data_d_bank,

        a_bank_acc: data_a_bank_acc,
        b_bank_acc: data_b_bank_acc,
        d_bank_acc: data_d_bank_acc,

        a_read_from_acc,
        b_read_from_acc,
        d_read_from_acc,

        a_unpadded_cols: if a_row_is_not_all_zeros { a_cols.unwrap_or(0.into_u()) } else { 0.into_u() },
        b_unpadded_cols: if b_row_is_not_all_zeros { b_cols.unwrap_or(0.into_u()) } else { 0.into_u() },
        d_unpadded_cols: if d_row_is_not_all_zeros { d_cols.unwrap_or(0.into_u()) } else { 0.into_u() },

        a_garbage,
        b_garbage,
        d_garbage,

        spad_reads,
        acc_reads,

        a_fire: a_valid && a_ready,
        b_fire: b_valid && b_ready,
        d_fire: d_valid && d_ready,

        a_should_be_fed_into_transposer,
        b_should_be_fed_into_transposer,

        c_addr: c_address_rs2,
        c_rows: c_rows.unwrap_or(0.into_u()),
        c_cols: c_cols.unwrap_or(0.into_u()),

        accumulate_zeros,
        preload_zeros,

        start_inputting_a,
        start_inputting_b,
        start_inputting_d,

        total_rows,

        rob_id: cmd.cmds[preload_cmd_place].and_then(|cmd| cmd.rob_id),

        dataflow: cfg.current_dataflow,
        shift: cfg.in_shift,
        transpose_a: cfg.transpose_a,
        transpose_bd: cfg.transpose_bd,

        prop: cmd.in_prop,

        first: !counters.a_fire_started && !counters.b_fire_started && !counters.d_fire_started,

        flush: 0.into_u(),
    }
}

/// Compute `last` bit and `s_next` for execute.
fn compute_last_and_s_next<const EX_QUEUE_LENGTH: usize>(
    cmd_w_type: ExeCmdT<EX_QUEUE_LENGTH>,
    cfg: ConfigS,
    signals: ControlSignals,
    s: CounterS,
) -> (bool, CounterS) {
    let firing = signals.start_inputting_a || signals.start_inputting_b || signals.start_inputting_d;

    let (a_fire_counter, a_addr_offset, a_fire_started) = if !firing {
        (0.into_u(), 0.into_u(), s.a_fire_started)
    } else if firing && signals.a_fire {
        let a_fire_counter = wrapping_inc::<4>(s.a_fire_counter, signals.total_rows);
        let a_addr_offset: U<20> = if s.a_fire_counter == (signals.total_rows - 1.into_u()).resize() {
            0.into_u()
        } else {
            (s.a_addr_offset + cfg.a_addr_stride.resize()).resize()
        };

        (a_fire_counter, a_addr_offset, true)
    } else {
        (s.a_fire_counter, s.a_addr_offset, s.a_fire_started)
    };

    let (b_fire_counter, b_fire_started) = if !firing {
        (0.into_u(), s.b_fire_started)
    } else if firing && signals.b_fire {
        (wrapping_inc::<4>(s.b_fire_counter, signals.total_rows), true)
    } else {
        (s.b_fire_counter, s.b_fire_started)
    };

    let (d_fire_counter, d_fire_started) = if !firing {
        (0.into_u(), s.d_fire_started)
    } else if firing && signals.d_fire {
        (wrapping_inc::<4>(s.d_fire_counter, signals.total_rows), true)
    } else {
        (s.d_fire_counter, s.d_fire_started)
    };

    let about_to_fire_all_rows = ((s.a_fire_counter.resize() == (signals.total_rows - 1.into_u()) && signals.a_fire)
        || s.a_fire_counter == 0.into_u())
        && ((s.b_fire_counter.resize() == (signals.total_rows - 1.into_u()) && signals.b_fire)
            || s.b_fire_counter == 0.into_u())
        && ((s.d_fire_counter.resize() == (signals.total_rows - 1.into_u()) && signals.d_fire)
            || s.d_fire_counter == 0.into_u())
        && (s.a_fire_started || s.b_fire_started || s.d_fire_started);

    let s_next = CounterS {
        a_fire_counter,
        b_fire_counter,
        d_fire_counter,
        a_fire_started,
        b_fire_started,
        d_fire_started,
        a_addr_offset,
        ..s
    };

    let s_next = if !about_to_fire_all_rows {
        s_next
    } else {
        CounterS { a_fire_started: false, b_fire_started: false, d_fire_started: false, ..s_next }
    };

    (about_to_fire_all_rows || matches!(cmd_w_type, ExeCmdT::Flush(_)), s_next)
}

fn filter_req<P: Copy, const D: Dep>(p: I<VrH<(bool, P), bool>, { D }>) -> Vr<P, { D }> {
    p.map_resolver(|er| er.ready).filter_map(|(is_valid, req)| if is_valid { Some(req) } else { None })
}

/// Generate scratchpad read requests
fn spad_read_req<const EX_QUEUE_LENGTH: usize>(
    cmd_mesh_spad: I<VrH<MeshControlSignals<EX_QUEUE_LENGTH>, Array<bool, SP_BANKS>>, { Dep::Helpful }>,
) -> [Vr<ScratchpadReadReq, { Dep::Demanding }>; SP_BANKS] {
    let (req0, req1, req2, req3) = cmd_mesh_spad
        .map(|spad| {
            let arr = range::<4>().map(|i| {
                let (read_a, read_b, read_d) = spad.signals.spad_reads[i];
                let d_fire_counter_mulpre = spad.counters.d_fire_counter;

                let addr = if read_a {
                    spad.signals.a_address_rs1.sp_row() + spad.counters.a_fire_counter.resize()
                } else if read_b {
                    spad.signals.b_address_rs2.sp_row() + spad.counters.b_fire_counter.resize()
                } else if read_d {
                    spad.signals.d_address_rs1.sp_row() + (BLOCK_SIZE - 1).into_u() - d_fire_counter_mulpre.resize()
                } else {
                    hpanic!("At least one of read_a, read_b, read_d must be true.")
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
    cmd_mesh_acc: I<VrH<MeshControlSignals<EX_QUEUE_LENGTH>, Array<bool, ACC_BANKS>>, { Dep::Helpful }>,
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
                    hpanic!("At least one of read_a, read_b, read_d must be true.")
                };

                let acc_req = AccumulatorReadReq {
                    scale: acc.cfg.acc_scale,
                    full: false,
                    act: acc.cfg.activation,
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

#[allow(unused)]
fn mesh_with_delays_wrapper(
    a: Vr<A>,
    b: Vr<B>,
    d: Vr<D>,
    req: I<VrH<MeshReq, TagsInProgress>, { Dep::Helpful }>,
) -> Valid<MeshResp> {
    mesh_with_delays_ffi(a, b, d, req)
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
    cmd_raw: Vr<GemminiCmd>,
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
    let (config_cmd, compute_cmd) = cmd_raw.comb(transpose_preload_unroller).comb(cmd_decoder::<EX_QUEUE_LENGTH>); // EX_QUEUE_LENGTH = 3

    // 2. Process the config command.
    //
    // It just return the ROB id of the command. The configuration information was parsed in the decode stage (step 1).
    let config_rob_id = config_cmd.filter_map(|(cmd, _)| {
        if let ExeCmdT::Config(cmd) = cmd {
            cmd.cmds[0].and_then(|cmd| cmd.rob_id)
        } else {
            None
        }
    });

    // 3. Compute all cofiguruations and signals. Also, wait for finishing fire all rows.
    let compute_cmd = compute_cmd.map_resolver_inner::<(U<2>, (Array<MeshTag, 6>, Array<bool, 6>), bool)>(
        |(pop_count, (matmul_in_progress, _), any_pending_robs)| (pop_count, matmul_in_progress, any_pending_robs),
    );
    let compute_cmd = unsafe {
        compute_cmd.fsm::<(), { Dep::Helpful }, VrH<
            (ExeCmdT<EX_QUEUE_LENGTH>, ConfigS, Array<bool, 6>),
            (U<2>, (Array<MeshTag, 6>, Array<bool, 6>), bool),
        >>((), |ip, er, ()| (ip.map(|(exe_cmd, cfg)| (exe_cmd, cfg, er.inner.1 .1)), er, ()))
    };
    let compute_cmd = compute_cmd
        .fsm_egress::<(bool, MeshControlSignals<EX_QUEUE_LENGTH>), CounterS>(
            CounterS::default(),
            true,
            true,
            |(cmd_w_type, cfg, sram_read_req_readies), counters| {
                let signals = compute_control_signals(cmd_w_type, cfg, counters, sram_read_req_readies);
                let (about_to_fire_all_rows, s_next) = compute_last_and_s_next(cmd_w_type, cfg, signals, counters);

                let ep =
                    (about_to_fire_all_rows, MeshControlSignals { cmd_decoded: cmd_w_type, cfg, counters, signals });

                (ep, s_next, about_to_fire_all_rows)
            },
        )
        .fsm_map(false, |(about_to_fire_all_rows, mesh_cntl_signals), s_in_prop_flush| {
            // TODO: Refactor this.
            let in_prop_flush = if !about_to_fire_all_rows {
                s_in_prop_flush
            } else if matches!(mesh_cntl_signals.cfg.current_dataflow, Dataflow::WS) {
                false
            } else {
                let cmd = mesh_cntl_signals.cmd_decoded;
                match cmd {
                    ExeCmdT::Config(_) => hpanic!("Config command is not allowed here."),
                    ExeCmdT::Preload(cmd) => {
                        if matches!(mesh_cntl_signals.cfg.current_dataflow, Dataflow::OS) {
                            !LocalAddr::from(cmd.rs2s[0]).is_garbage()
                        } else {
                            s_in_prop_flush
                        }
                    }
                    ExeCmdT::PreloadAndCompute(cmd) => {
                        if matches!(mesh_cntl_signals.cfg.current_dataflow, Dataflow::OS) {
                            !LocalAddr::from(cmd.rs2s[1]).is_garbage()
                        } else {
                            s_in_prop_flush
                        }
                    }
                    ExeCmdT::Compute(_) | ExeCmdT::Flush(_) => s_in_prop_flush,
                }
            };

            let counter_updated = CounterS { in_prop_flush, ..mesh_cntl_signals.counters };
            let ep = (about_to_fire_all_rows, MeshControlSignals { counters: counter_updated, ..mesh_cntl_signals });

            (ep, in_prop_flush)
        })
        .map_resolver_with_p::<((TagsInProgress, Array<bool, 6>), bool)>(|ip, er| {
            let about_to_fire_all_rows = ip.map_or(false, |p| p.0);

            let pop_count = if about_to_fire_all_rows && er.ready {
                ip.map_or(0.into_u(), |p| match p.1.cmd_decoded {
                    ExeCmdT::Config(_) => hpanic!("Config command is not allowed here.."),
                    ExeCmdT::Preload(_) => 1.into_u(),
                    ExeCmdT::PreloadAndCompute(_) => 2.into_u(),
                    ExeCmdT::Compute(_) => 1.into_u(),
                    ExeCmdT::Flush(_) => 0.into_u(),
                })
            } else {
                0.into_u()
            };

            let any_pending_robs = er.inner.1;

            Ready::new(er.ready, (pop_count, er.inner.0, any_pending_robs))
        });

    let (compute_cmd, pending_completed_rob_ids) = compute_cmd.lfork();

    // 4. Process the pending completed rob ids.
    let pending_completed_rob_ids = pending_completed_rob_ids
        .filter_map(|ip: (bool, MeshControlSignals<EX_QUEUE_LENGTH>)| {
            let (about_to_fire_all_rows, mesh_control_signals) = ip;

            if about_to_fire_all_rows {
                match mesh_control_signals.cmd_decoded {
                    ExeCmdT::Config(_) => None,
                    ExeCmdT::Preload(cmd) => {
                        let pending_completed_rob_ids_0 = cmd.cmds[0].and_then(|cmd0| {
                            if cmd0.rob_id.is_some() && mesh_control_signals.signals.c_address_rs2.is_garbage() {
                                cmd0.rob_id
                            } else {
                                None
                            }
                        });

                        Some(Array::from([pending_completed_rob_ids_0, None]))
                    }
                    ExeCmdT::Compute(cmd) => {
                        let pending_completed_rob_ids_0 = cmd.cmds[0].and_then(|cmd0| cmd0.rob_id);
                        let pending_completed_rob_ids_1 = cmd.cmds[1].and_then(|cmd1| {
                            if about_to_fire_all_rows
                                && cmd1.rob_id.is_some()
                                && mesh_control_signals.signals.c_address_rs2.is_garbage()
                            {
                                cmd1.rob_id
                            } else {
                                None
                            }
                        });

                        Some(Array::from([pending_completed_rob_ids_0, pending_completed_rob_ids_1]))
                    }
                    ExeCmdT::PreloadAndCompute(cmd) => {
                        let pending_completed_rob_ids_0 = cmd.cmds[0].and_then(|cmd0| cmd0.rob_id);

                        Some(Array::from([pending_completed_rob_ids_0, None]))
                    }
                    ExeCmdT::Flush(_) => None,
                }
            } else {
                None
            }
        })
        .fsm_egress::<HOption<U<{ clog2(RS_ENTRIES) }>>, U<2>>(
            0.into_u(),
            true,
            true,
            |pending_rob_ids: Array<HOption<U<{ clog2(RS_ENTRIES) }>>, 2>, ptr: U<2>| {
                let num_elements =
                    (U::from(pending_rob_ids[0].is_some()) + U::from(pending_rob_ids[1].is_some())).resize::<2>();
                let is_last = ptr >= num_elements;

                let ptr_next = wrapping_inc::<2>(ptr, 3.into_u());

                (pending_rob_ids[ptr], ptr_next, is_last)
            },
        )
        .filter_map::<U<{ clog2(RS_ENTRIES) }>>(|p| p)
        .map_resolver_inner_with_p(|ip, _er| ip.is_some());

    // 5. Process the mesh(compute, flush) command.
    //
    // We have to do the following:
    // 1) Return the rob id of the mesh command.
    // 2) Compute with the mesh: read the operands from SRAM -> run the mesh -> write the result back to SRAM.
    let (compute_cmd, write_req_config) = compute_cmd.map(|p| p.1).lfork_uni();
    let (nonflush_compute_cmd, flush_cmd) = compute_cmd.lfork_uni();

    let flush_cmd = flush_cmd
        .filter_map(|p| if let ExeCmdT::Flush(_) = p.cmd_decoded { Some(p) } else { None })
        .discard_into_vr()
        .map(|p| {
            let MeshControlSignals { counters, signals, .. } = p;

            ControlSignals {
                perform_single_mul: false,
                perform_single_preload: false,
                a_bank: 0.into_u(),
                b_bank: 0.into_u(),
                d_bank: 0.into_u(),
                a_bank_acc: 0.into_u(),
                b_bank_acc: 0.into_u(),
                d_bank_acc: 0.into_u(),
                a_read_from_acc: false,
                b_read_from_acc: false,
                d_read_from_acc: false,
                a_garbage: false,
                b_garbage: false,
                d_garbage: false,
                a_address: LocalAddr::garbage(),
                b_address: LocalAddr::garbage(),
                d_address: LocalAddr::garbage(),
                a_address_rs1: LocalAddr::garbage(),
                b_address_rs2: LocalAddr::garbage(),
                d_address_rs1: LocalAddr::garbage(),
                c_address_rs2: LocalAddr::garbage(),
                a_unpadded_cols: 0.into_u(),
                b_unpadded_cols: 0.into_u(),
                d_unpadded_cols: 0.into_u(),
                a_fire: false,
                b_fire: false,
                d_fire: false,
                spad_reads: Array::from([(false, false, false); SP_BANKS]),
                acc_reads: Array::from([(false, false, false); ACC_BANKS]),
                a_should_be_fed_into_transposer: false,
                b_should_be_fed_into_transposer: false,
                accumulate_zeros: false,
                preload_zeros: false,
                start_inputting_a: false,
                start_inputting_b: false,
                start_inputting_d: false,
                c_addr: LocalAddr::from(0.into_u()),
                c_rows: signals.c_rows,
                c_cols: signals.c_cols,
                transpose_a: signals.transpose_a,
                transpose_bd: signals.transpose_bd,
                total_rows: BLOCK_SIZE.into_u(),
                rob_id: signals.rob_id,
                dataflow: signals.dataflow,
                prop: counters.in_prop_flush,
                shift: signals.shift,
                first: false,
                flush: 1.into_u(),
            }
        })
        .reg_fwd(true)
        .map_resolver_inner(|_| ());

    let (cmd_mesh_cntl, cmd_mesh_mem) =
        nonflush_compute_cmd.filter_map(|p| if let ExeCmdT::Flush(_) = p.cmd_decoded { None } else { Some(p) }).lfork();

    // 6. Read
    let (cmd_mesh_spad, cmd_mesh_acc) = cmd_mesh_mem
        .map_resolver_inner::<(Array<bool, SP_BANKS>, Array<bool, ACC_BANKS>)>(|(er_inner1, er_inner2)| {
            U::from([er_inner1[0], er_inner1[1], er_inner1[2], er_inner1[3], er_inner2[0], er_inner2[1]])
        })
        .lfork();
    let spad_resps = cmd_mesh_spad.comb(spad_read_req).comb(spad_readers);
    let acc_resps = cmd_mesh_acc.comb(acc_read_req).comb(acc_readers);

    // 7. Run Mesh
    let cmd_mesh_cntl = cmd_mesh_cntl.map(|cmd: MeshControlSignals<EX_QUEUE_LENGTH>| cmd.signals).fifo::<5>(); // TODO: Use `{ SPAD_READ_DELAY + 1 }` instead of `5`
    let cmd_mesh_cntl: I<VrH<ControlSignals, TagsInProgress>, { Dep::Helpful }> =
        [cmd_mesh_cntl, flush_cmd].merge().reg_fwd(true);
    let (mesh_a, mesh_b, mesh_d, mesh_req) = generate_mesh_inputs::<TR, MR>(cmd_mesh_cntl, spad_resps, acc_resps);

    // TODO: Do not use magic number.
    // let mesh_resp = mwd_inner(mesh_a, mesh_b, mesh_d, mesh_req);
    let mesh_resp = mesh_with_delays_wrapper(mesh_a, mesh_b, mesh_d, mesh_req);

    let (mesh_resp, mesh_resp_rob_id) = mesh_resp.lfork_uni();

    // 8. Process mesh response rob id.
    let mesh_resp_rob_id =
        mesh_resp_rob_id.filter_map(|resp| if resp.last { resp.tag.rob_id } else { None }).discard_into_vr();

    // 9. Write
    let sram_write: Valid<MeshRespExtended> =
        mesh_resp.fsm_map::<MeshRespExtended, U<4>>(0.into_u(), |ip, output_counter| {
            let start_array_outputting = !ip.tag.addr.is_garbage();
            let s_next = wrapping_inc::<4>(output_counter, 16.into_u());

            (MeshRespExtended { mesh_resp: ip, output_counter, start_array_outputting }, s_next)
        });

    let (spad_write, acc_write) = sram_write.lfork();

    let (write_req_config_spad, write_req_config_acc) = unsafe {
        write_req_config
            .fsm::<HOption<(Dataflow, U<3>, U<16>)>, { Dep::Helpful }, ValidH<(Dataflow, U<3>, U<16>), _>>(
                None,
                |ip, _er: (), s| {
                    if let Some(ip) = ip {
                        let dataflow = ip.signals.dataflow;
                        let act = ip.cfg.activation;
                        let c_addr_stride = ip.cfg.c_addr_stride;

                        let ep = Some((dataflow, act, c_addr_stride));
                        (ep, (), ep)
                    } else {
                        (s, (), s)
                    }
                },
            )
            .lfork()
    };

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
pub fn exe(
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
