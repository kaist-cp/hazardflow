//! Store controller.

use crate::gemmini::dma::dma_command_tracker::*;
use crate::gemmini::isa::*;
use crate::gemmini::scratchpad::*;
use crate::gemmini::sram::dma::*;
use crate::gemmini::*;

const BLOCK_ROWS: usize = MESH_ROWS * TILE_ROWS;
const BLOCK_COLS: usize = MESH_COLS * TILE_COLS;

const CL_BLOCK_COLS: usize = clog2(BLOCK_COLS);

#[derive(Debug, Default, Clone, Copy)]
struct PoolConfig {
    stride: U<CONFIG_MVOUT_RS1_MAX_POOLING_STRIDE_WIDTH>,
    size: U<CONFIG_MVOUT_RS1_MAX_POOLING_WINDOW_SIZE_WIDTH>,
    out_dim: U<CONFIG_MVOUT_RS1_POOL_OUT_DIM_WIDTH>,
    porows: U<CONFIG_MVOUT_RS1_POOL_OUT_ROWS_WIDTH>,
    pocols: U<CONFIG_MVOUT_RS1_POOL_OUT_COLS_WIDTH>,
    orows: U<CONFIG_MVOUT_RS1_OUT_ROWS_WIDTH>,
    ocols: U<CONFIG_MVOUT_RS1_OUT_COLS_WIDTH>,
    upad: U<CONFIG_MVOUT_RS1_UPPER_ZERO_PADDING_WIDTH>,
    lpad: U<CONFIG_MVOUT_RS1_LEFT_ZERO_PADDING_WIDTH>,
}

#[derive(Debug, Default, Clone, Copy)]
struct Config {
    stride: U<CORE_MAX_ADDR_BITS>,

    activation: U<3>, // TODO: magic number
    igelu_qb: U<32>,
    igelu_qc: U<32>,
    iexp_qln2: U<32>,
    iexp_qln2_inv: U<32>,
    norm_stats_id: U<8>, // TODO: magic number
    acc_scale: U<ACC_SCALE_BITS>,

    pool: PoolConfig,
}

#[derive(Debug, Default, Clone, Copy)]
struct Counter {
    row: U<12>,  // TODO: magic number
    block: U<8>, // TODO: magic number

    porow: U<CONFIG_MVOUT_RS1_POOL_OUT_ROWS_WIDTH>,
    pocol: U<CONFIG_MVOUT_RS1_POOL_OUT_COLS_WIDTH>,
    wrow: U<CONFIG_MVOUT_RS1_MAX_POOLING_WINDOW_SIZE_WIDTH>,
    wcol: U<CONFIG_MVOUT_RS1_MAX_POOLING_WINDOW_SIZE_WIDTH>,
}

#[derive(Debug, Clone, Copy)]
struct CmdDecoded {
    cmd: GemminiCmd,
    vaddr: U<64>,
    config: Config,

    pooling_is_enabled: bool,
    mvout_1d_enabled: bool,

    mvout_1d_rows: U<16>,   // TODO: Change 16 to correct value
    pool_total_rows: U<20>, // TODO: Change 20 to correct value

    mvout_rs2: MvoutRs2<MVOUT_ROWS_BITS, MVOUT_COLS_BITS>,
    blocks: U<MVOUT_COLS_BITS>,

    config_mvout_rs1: ConfigMvoutRs1,
    config_mvout_rs2: ConfigMvoutRs2<ACC_SCALE_BITS, 32>,

    config_norm_rs1: ConfigNormRs1<32>,
    config_norm_rs2: ConfigNormRs2<32>,
}

fn decode_cmd(cmd: GemminiCmd, config: Config) -> CmdDecoded {
    let pooling_is_enabled = config.pool.stride != 0.into_u(); // TODO: Add `&& has_max_pool.B`
    let mvout_1d_enabled = config.pool.size != 0.into_u() && !pooling_is_enabled;

    let mvout_1d_rows = config.pool.orows * config.pool.ocols;
    let pool_total_rows = config.pool.porows * config.pool.pocols * config.pool.size * config.pool.size;

    let mvout_rs2 = MvoutRs2::<MVOUT_ROWS_BITS, MVOUT_COLS_BITS>::from(cmd.cmd.rs2);
    let blocks = (mvout_rs2.num_cols >> CL_BLOCK_COLS)
        .trunk_add(((mvout_rs2.num_cols & (BLOCK_COLS - 1).into_u()) != 0.into_u()).into_u());

    let config_mvout_rs1 = ConfigMvoutRs1::from(cmd.cmd.rs1);
    let config_mvout_rs2 = ConfigMvoutRs2::<ACC_SCALE_BITS, 32>::from(cmd.cmd.rs2);

    let config_norm_rs1 = ConfigNormRs1::<32>::from(cmd.cmd.rs1);
    let config_norm_rs2 = ConfigNormRs2::<32>::from(cmd.cmd.rs2);

    CmdDecoded {
        cmd,
        vaddr: cmd.cmd.rs1,
        config,
        pooling_is_enabled,
        mvout_1d_enabled,
        mvout_1d_rows,
        pool_total_rows,
        mvout_rs2,
        blocks,
        config_mvout_rs1,
        config_mvout_rs2,
        config_norm_rs1,
        config_norm_rs2,
    }
}

fn update_config(cmd_decoded: CmdDecoded, config: Config) -> Config {
    let do_config = matches!(cmd_decoded.cmd.cmd.inst.funct, Funct::ConfigCmd)
        && matches!(ConfigCmd::from(cmd_decoded.config_mvout_rs1.cmd_type), ConfigCmd::Store);
    let do_config_norm = matches!(cmd_decoded.cmd.cmd.inst.funct, Funct::ConfigCmd)
        && matches!(ConfigCmd::from(cmd_decoded.config_mvout_rs1.cmd_type), ConfigCmd::Norm);

    // If command is not changing config, return early.
    if !do_config && !do_config_norm {
        return config;
    }

    if do_config {
        let pool = if cmd_decoded.config_mvout_rs1.pool_stride != 0.into_u() {
            PoolConfig {
                size: cmd_decoded.config_mvout_rs1.pool_size,
                stride: cmd_decoded.config_mvout_rs1.pool_stride,
                out_dim: cmd_decoded.config_mvout_rs1.pool_out_dim,
                porows: cmd_decoded.config_mvout_rs1.porows,
                pocols: cmd_decoded.config_mvout_rs1.pocols,
                orows: cmd_decoded.config_mvout_rs1.orows,
                ocols: cmd_decoded.config_mvout_rs1.ocols,
                upad: cmd_decoded.config_mvout_rs1.upad,
                lpad: cmd_decoded.config_mvout_rs1.lpad,
            }
        } else {
            PoolConfig {
                orows: cmd_decoded.config_mvout_rs1.orows,
                ocols: cmd_decoded.config_mvout_rs1.ocols,
                out_dim: cmd_decoded.config_mvout_rs1.pool_out_dim,
                ..config.pool
            }
        };

        Config {
            stride: cmd_decoded.config_mvout_rs2.stride.resize(),
            activation: cmd_decoded.config_mvout_rs1.activation.resize(),
            acc_scale: if !cmd_decoded.config_mvout_rs2.acc_scale.all(|b| b) {
                cmd_decoded.config_mvout_rs2.acc_scale
            } else {
                config.acc_scale
            },
            pool,
            ..config
        }
    } else if cmd_decoded.config_norm_rs1.set_stats_id_only != 0.into_u() {
        Config {
            igelu_qb: cmd_decoded.config_norm_rs2.qb,
            igelu_qc: cmd_decoded.config_norm_rs2.qc,
            iexp_qln2: if cmd_decoded.config_norm_rs1.q_const_type == 0.into_u() {
                cmd_decoded.config_norm_rs1.q_const
            } else {
                config.iexp_qln2
            },
            iexp_qln2_inv: if cmd_decoded.config_norm_rs1.q_const_type == 1.into_u() {
                cmd_decoded.config_norm_rs1.q_const
            } else {
                config.iexp_qln2_inv
            },
            activation: config.activation.set_range(2, cmd_decoded.config_norm_rs1.act_msb),
            norm_stats_id: cmd_decoded.config_norm_rs1.norm_stats_id,
            ..config
        }
    } else {
        Config { norm_stats_id: cmd_decoded.config_norm_rs1.norm_stats_id, ..config }
    }
}

fn compute_alloc_req<const MAX_BYTES: usize>(cmd_decoded: CmdDecoded) -> AllocReq<U<{ clog2(RS_ENTRIES) }>, MAX_BYTES>
where [(); clog2(MAX_BYTES + 1)]: {
    AllocReq {
        bytes_to_read: if !cmd_decoded.pooling_is_enabled {
            if cmd_decoded.mvout_1d_enabled {
                cmd_decoded.mvout_1d_rows.resize()
            } else {
                (cmd_decoded.mvout_rs2.num_rows * cmd_decoded.blocks).resize()
            }
        } else {
            cmd_decoded.pool_total_rows.resize()
        },
        // `unwrap()` always success because the ROB ID is inserted in the controller between reservation station and store controller.
        tag: cmd_decoded.cmd.rob_id.unwrap(),
    }
}

fn compute_dma_req<const NCMDS: usize>(
    cmd_id: U<{ clog2(NCMDS) }>,
    cmd_decoded: CmdDecoded,
    counter: Counter,
) -> ScratchpadMemWriteReq<32, ACC_SCALE_BITS> {
    let config = cmd_decoded.config;

    let vaddr = cmd_decoded.vaddr;
    let stride = config.stride;
    let pool = config.pool;
    let localaddr = cmd_decoded.mvout_rs2.local_addr;

    let pooling_is_enabled = cmd_decoded.pooling_is_enabled;
    let mvout_1d_enabled = cmd_decoded.mvout_1d_enabled;

    let orow = counter.porow * pool.stride + counter.wrow.resize() - pool.upad.resize();

    let ocol = counter.pocol * pool.stride + counter.wcol.resize() - pool.lpad.resize();

    let current_vaddr = vaddr + (counter.row * stride).resize();
    let current_localaddr = localaddr + (counter.block * BLOCK_ROWS.into_u::<5>() + counter.row.resize());

    let pool_row_addr = localaddr + (orow * pool.ocols + ocol.resize()).resize();
    // TODO: Add below logic
    // when (orow_is_negative || ocol_is_negative || orow >= pool_orows || ocol >= pool_ocols) {
    //     pool_row_addr.make_this_garbage()
    //   }
    let pool_vaddr = vaddr + ((counter.porow * pool.out_dim + counter.pocol.resize()) * stride).resize();

    ScratchpadMemWriteReq {
        vaddr: if pooling_is_enabled || mvout_1d_enabled { pool_vaddr.resize() } else { current_vaddr.resize() },
        laddr: if pooling_is_enabled { pool_row_addr } else { current_localaddr }, // TODO: Change `norm_cmd` field
        acc_act: config.activation,
        acc_igelu_qb: config.igelu_qb,
        acc_igelu_qc: config.igelu_qc,
        acc_iexp_qln2: config.iexp_qln2,
        acc_iexp_qln2_inv: config.iexp_qln2_inv,
        acc_norm_stats_id: config.norm_stats_id,
        acc_scale: config.acc_scale,
        len: if counter.block == (cmd_decoded.blocks - 1.into_u()).resize() {
            (((cmd_decoded.mvout_rs2.num_cols - 1.into_u()) & (BLOCK_COLS - 1).into_u()) + 1.into_u()).resize()
        } else {
            BLOCK_COLS.into_u()
        },
        block: counter.block,
        cmd_id: cmd_id.resize(),
        status: cmd_decoded.cmd.cmd.status,
        pool_en: pooling_is_enabled && (counter.wrow != 0.into_u() || counter.wcol != 0.into_u()),
        store_en: if pooling_is_enabled {
            counter.wrow == pool.size - 1.into_u() && counter.wcol == pool.size - 1.into_u()
        } else {
            counter.block == (cmd_decoded.blocks - 1.into_u()).resize()
        },
    }
}

/// Store controller.
///
/// This module is responsible for all instructions that move data from Gemmini's private SRAMs into main memory.
/// This module is also responsible for "max-pooling" instructions, because Gemmini performs pooling when moving unpooled data from the private SRAMs into main memory.
/// Ingresses cmd from `ReservationStation` and egresses RobId to `ReservationStation`
///
/// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/StoreController.scala>
pub fn store<const NCMDS: usize, const MAX_BYTES: usize>(
    cmd: Vr<GemminiCmd>,
    dma_accessor: impl FnOnce(Vr<ScratchpadMemWriteReq<32, ACC_SCALE_BITS>>) -> Valid<ScratchpadMemWriteResp>,
) -> Vr<U<{ clog2(RS_ENTRIES) }>>
where
    [(); clog2(NCMDS)]:,
    [(); clog2(MAX_BYTES + 1)]:,
{
    let (alloc_m, complete_m) = module_split(dma_command_tracker::<U<{ clog2(RS_ENTRIES) }>, NCMDS, MAX_BYTES>);

    // TODO: Use `ST_QUEUE_LENGTH` instead of `2`.
    let cmd = cmd.fifo::<2>().fsm_map::<CmdDecoded, Config>(Config::default(), |ip, s| {
        let cmd_decoded = decode_cmd(ip, s);
        let s_next = update_config(cmd_decoded, s);

        (cmd_decoded, s_next)
    });

    let (cmd_config, cmd_store) = cmd
        .map::<(CmdDecoded, BoundedU<2>)>(|cmd_decoded| {
            let sel = if matches!(cmd_decoded.cmd.cmd.inst.funct, Funct::ConfigCmd) { 0.into_u() } else { 1.into_u() };
            (cmd_decoded, BoundedU::new(sel))
        })
        .map_resolver_inner::<((), ())>(|_| ())
        .branch();

    cmd_config.sink_fsm_map((), |_, s| (Ready::valid(()), s));

    let alloc_resp = cmd_store
        .map(|cmd_decoded| (compute_alloc_req(cmd_decoded), cmd_decoded))
        .comb(attach_payload(attach_ready(alloc_m)));

    let dma_resp = alloc_resp
        .fsm_egress::<ScratchpadMemWriteReq<32, ACC_SCALE_BITS>, Counter>(
            Counter::default(),
            true,
            true,
            |(alloc_resp, cmd_decoded), counter| {
                let ep = compute_dma_req(alloc_resp.cmd_id, cmd_decoded, counter);

                // The const parameters for `wrapping_add`s are literals insteand of constants because of a Rust bug.
                // (https://github.com/rust-lang/rust/issues/89421)
                let pool = cmd_decoded.config.pool;
                let counter_next = if !cmd_decoded.pooling_is_enabled {
                    if cmd_decoded.mvout_1d_enabled {
                        Counter {
                            pocol: wrapping_add::<8>(counter.pocol, 1.into_u(), pool.ocols.resize()),
                            porow: if counter.pocol == pool.ocols - 1.into_u() {
                                wrapping_add::<8>(counter.pocol, 1.into_u(), pool.orows.resize())
                            } else {
                                counter.pocol
                            },
                            row: wrapping_add::<12>(counter.row, 1.into_u(), cmd_decoded.mvout_1d_rows.resize()),
                            block: wrapping_add::<8>(counter.block, 1.into_u(), cmd_decoded.blocks.resize()),
                            ..counter
                        }
                    } else {
                        Counter {
                            row: if counter.block == cmd_decoded.blocks.resize() - 1.into_u() {
                                wrapping_add::<12>(counter.row, 1.into_u(), cmd_decoded.mvout_rs2.num_rows.resize())
                            } else {
                                counter.row
                            },
                            block: wrapping_add::<8>(counter.block, 1.into_u(), cmd_decoded.blocks.resize()),
                            ..counter
                        }
                    }
                } else {
                    Counter {
                        wcol: wrapping_add::<2>(counter.wcol, 1.into_u(), pool.size.resize()),
                        wrow: if counter.wcol == pool.size - 1.into_u() {
                            wrapping_add::<2>(counter.wrow, 1.into_u(), pool.size.resize())
                        } else {
                            counter.wrow
                        },
                        pocol: if counter.wrow == pool.size - 1.into_u() && counter.wcol == pool.size - 1.into_u() {
                            wrapping_add::<8>(counter.pocol, 1.into_u(), pool.pocols.resize())
                        } else {
                            counter.pocol
                        },
                        porow: if counter.pocol == pool.pocols - 1.into_u()
                            && counter.wrow == pool.size - 1.into_u()
                            && counter.wcol == pool.size - 1.into_u()
                        {
                            wrapping_add::<8>(counter.porow, 1.into_u(), pool.porows.resize())
                        } else {
                            counter.porow
                        },
                        ..counter
                    }
                };

                let is_last = if cmd_decoded.pooling_is_enabled {
                    counter.porow == cmd_decoded.config.pool.porows - 1.into_u()
                        && counter.pocol == cmd_decoded.config.pool.pocols - 1.into_u()
                        && counter.wrow == cmd_decoded.config.pool.size - 1.into_u()
                        && counter.wcol == cmd_decoded.config.pool.size - 1.into_u()
                } else {
                    let last_block = counter.block == (cmd_decoded.blocks - 1.into_u()).resize();
                    let last_row = counter.row
                        == (if cmd_decoded.mvout_1d_enabled {
                            cmd_decoded.mvout_1d_rows - 1.into_u()
                        } else {
                            cmd_decoded.mvout_rs2.num_rows.resize() - 1.into_u()
                        })
                        .resize();

                    last_block && last_row
                };

                (ep, counter_next, is_last)
            },
        )
        .comb(dma_accessor);

    dma_resp
        .map(|p| RequestReturned { bytes_read: 1.into_u(), cmd_id: p.cmd_id.resize() })
        .comb(complete_m)
        .map(|p| p.tag)
}

/// Debug
#[synthesize]
pub fn store_default(
    cmd: Vr<GemminiCmd>,
    dma_accessor: impl FnOnce(Vr<ScratchpadMemWriteReq<32, ACC_SCALE_BITS>>) -> Valid<ScratchpadMemWriteResp>,
) -> Vr<U<{ clog2(RS_ENTRIES) }>> {
    store::<2, 16384>(cmd, dma_accessor)
}
