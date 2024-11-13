//! Gemmini ISA
//!
//! TODO: Hardcoded values

#![allow(missing_docs)] // TODO: Remove this.

use rocc::*;

use crate::gemmini::execute::systolic_array::pe::*;
use crate::gemmini::local_addr::*;
use crate::gemmini::*;

pub mod rocc;

/// Funct values.
#[derive(Debug, Clone, Copy, HEq)]
pub enum Funct {
    ConfigCmd,
    Load2Cmd,
    LoadCmd,
    StoreCmd,
    ComputeAndFlipCmd,
    ComputeAndStayCmd,
    PreloadCmd,
    FlushCmd,

    LoopWs,
    LoopWsConfigBounds,
    LoopWsConfigAddrsAB,
    LoopWsConfigAddrsDC,
    LoopWsConfigStridesAB,
    LoopWsConfigStridesDC,

    Load3Cmd,

    LoopConvWs, // no_bias, wrot180, trans_output_1203, trans_weight_1203, trans_input_3120, dw, max_pixels_per_row | no_pool, downsample, input_dilated, act
    LoopConvWsConfig1, // batch_size, in_dim, in_channels, out_channels | out_dim, pool_out_dim, stride, padding
    LoopConvWsConfig2, // kernel_dim, pool_size, pool_stride, pool_padding | batches, porows, pocols, pochs
    LoopConvWsConfig3, // krows, kcols, kchs, lpad | rpad, upad, dpad, plpad
    LoopConvWsConfig4, // prad, pupad, pdpad, orows | ocols, kernel_dilation
    LoopConvWsConfig5, // *weights | *output
    LoopConvWsConfig6, // *bias, *input

    ClkGateEn,
}

/// Config command type. It is generated from `rs1[2:0]`.
#[derive(Debug, Clone, Copy, HEq)]
pub enum ConfigCmd {
    Ex,
    Load,
    Store,
    Norm,
}

impl From<U<2>> for ConfigCmd {
    fn from(value: U<2>) -> ConfigCmd {
        if value == 0.into_u() {
            ConfigCmd::Ex
        } else if value == 1.into_u() {
            ConfigCmd::Load
        } else if value == 2.into_u() {
            ConfigCmd::Store
        } else {
            // value is 3
            ConfigCmd::Norm
        }
    }
}

/* Configurations for excuting rs1, rs2 */

// rs1
pub const CONFIG_EX_RS1_CMD_TYPE_WIDTH: usize = 2;
pub const CONFIG_EX_RS1_DATAFLOW_WIDTH: usize = 1;
pub const CONFIG_EX_RS1_ACTIVATION_WIDTH: usize = 2;
pub const CONFIG_EX_RS1_SPACERO_WIDTH: usize = 7 - 2 - 1 - 2;
pub const CONFIG_EX_RS1_SET_ONLY_STRIDES_WIDTH: usize = 1;
pub const CONFIG_EX_RS1_TRANSPOSE_A_WIDTH: usize = 1;
pub const CONFIG_EX_RS1_TRANSPOSE_BD_WIDTH: usize = 1;
pub const CONFIG_EX_RS1_SPACER1_WIDTH: usize = 16 - 10;
pub const CONFIG_EX_RS1_A_STRIDE_WIDTH: usize = 16;
pub const CONFIG_EX_RS1_ACC_SCALE_WIDTH: usize = 32;
// rs2
pub const CONFIG_EX_RS2_IN_SHIFT_WIDTH: usize = 32;
pub const CONFIG_EX_RS2_RELU6_SHIFT_WIDTH: usize = 16;
pub const CONFIG_EX_RS2_C_STRIDE_WIDTH: usize = 16;

#[derive(Debug, Clone, Copy)]
pub struct RoCCInstruction {
    pub funct: U<7>,
    pub rs2: U<5>,
    pub rs1: U<5>,
    pub xd: U<1>,
    pub xs1: U<1>,
    pub xs2: U<1>,
    pub rd: U<5>,
}

#[derive(Debug, Clone, Copy)]
pub struct GemminiCmd {
    pub cmd: RoCCCommand<64>,
    pub rob_id: HOption<U<{ clog2(ROB_ENTRIES) }>>,
    pub from_matmul_fsm: bool,
    pub from_conv_fsm: bool,
}

impl GemminiCmd {
    pub fn is_same(self, other: GemminiCmd) -> bool {
        self.cmd.rs1 == other.cmd.rs1 && self.cmd.rs2 == other.cmd.rs2 && self.rob_id == other.rob_id
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigExRs1<const ACC_SCALE_BITS: usize> {
    // pub spacer2;
    pub acc_scale: U<ACC_SCALE_BITS>, // default: fp32
    pub a_stride: U<CONFIG_EX_RS1_A_STRIDE_WIDTH>,
    // pub spacer1;
    pub transpose_bd: bool,
    pub transpose_a: bool,
    pub set_only_strides: bool,
    // pub spacer0;
    pub activation: U<CONFIG_EX_RS1_ACTIVATION_WIDTH>,
    pub dataflow: Dataflow, // 1 bit signal
    pub cmd_type: ConfigCmd,
}

impl<const ACC_SCALE_BITS: usize> ConfigExRs1<ACC_SCALE_BITS> {
    pub fn new(rs1s: HOption<U<64>>) -> Self {
        let rs1s = rs1s.unwrap();

        let dataflow_offset = CONFIG_EX_RS1_CMD_TYPE_WIDTH;
        let activation_offset = CONFIG_EX_RS1_DATAFLOW_WIDTH + dataflow_offset;
        let set_only_strides_offset = CONFIG_EX_RS1_SPACERO_WIDTH + CONFIG_EX_RS1_ACTIVATION_WIDTH + activation_offset;
        let transpose_a_offset = CONFIG_EX_RS1_SET_ONLY_STRIDES_WIDTH + set_only_strides_offset;
        let transpose_bd_offset = CONFIG_EX_RS1_TRANSPOSE_A_WIDTH + transpose_a_offset;
        let a_stride_offset = CONFIG_EX_RS1_SPACER1_WIDTH + CONFIG_EX_RS1_TRANSPOSE_BD_WIDTH + transpose_bd_offset;
        let acc_scale_offset = CONFIG_EX_RS1_A_STRIDE_WIDTH + a_stride_offset;

        ConfigExRs1 {
            acc_scale: rs1s.clip_const::<ACC_SCALE_BITS>(acc_scale_offset),
            a_stride: rs1s.clip_const::<CONFIG_EX_RS1_A_STRIDE_WIDTH>(a_stride_offset),
            transpose_bd: rs1s.clip_const::<CONFIG_EX_RS1_TRANSPOSE_BD_WIDTH>(transpose_bd_offset)[0],
            transpose_a: rs1s.clip_const::<CONFIG_EX_RS1_TRANSPOSE_A_WIDTH>(transpose_a_offset)[0],
            set_only_strides: rs1s.clip_const::<CONFIG_EX_RS1_SET_ONLY_STRIDES_WIDTH>(set_only_strides_offset)[0],
            activation: rs1s.clip_const::<CONFIG_EX_RS1_ACTIVATION_WIDTH>(activation_offset),
            dataflow: if rs1s[dataflow_offset] { Dataflow::WS } else { Dataflow::OS },
            cmd_type: ConfigCmd::from(rs1s.clip_const::<CONFIG_EX_RS1_CMD_TYPE_WIDTH>(0)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigExRs2 {
    pub c_stride: U<CONFIG_EX_RS2_C_STRIDE_WIDTH>,
    // pub relu6_shift;
    pub in_shift: U<CONFIG_EX_RS2_IN_SHIFT_WIDTH>,
}

impl ConfigExRs2 {
    pub fn new(rs2s: HOption<U<64>>) -> Self {
        let rs2s = rs2s.unwrap();
        ConfigExRs2 {
            c_stride: rs2s.clip_const::<CONFIG_EX_RS2_C_STRIDE_WIDTH>(
                CONFIG_EX_RS2_RELU6_SHIFT_WIDTH + CONFIG_EX_RS2_IN_SHIFT_WIDTH,
            ),
            in_shift: rs2s.clip_const::<CONFIG_EX_RS2_IN_SHIFT_WIDTH>(0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MvinRs2<const ROWS_BITS: usize, const COLS_BITS: usize> {
    pub num_rows: U<ROWS_BITS>,
    pub num_cols: U<COLS_BITS>,
    pub local_addr: LocalAddr,
}

impl<const ROWS_BITS: usize, const COLS_BITS: usize> From<U<64>> for MvinRs2<ROWS_BITS, COLS_BITS> {
    fn from(value: U<64>) -> Self {
        let num_rows = value.clip_const::<16>(48);
        let num_cols = value.clip_const::<16>(32);
        let local_addr = LocalAddr::from(value);
        Self { num_rows: num_rows.resize(), num_cols: num_cols.resize(), local_addr }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MvoutRs2<const ROWS_BITS: usize, const COLS_BITS: usize> {
    pub num_rows: U<ROWS_BITS>,
    pub num_cols: U<COLS_BITS>,
    pub local_addr: LocalAddr,
}

impl<const ROWS_BITS: usize, const COLS_BITS: usize> From<U<64>> for MvoutRs2<ROWS_BITS, COLS_BITS> {
    fn from(value: U<64>) -> Self {
        let num_rows = value.clip_const::<16>(48);
        let num_cols = value.clip_const::<16>(32);
        let local_addr = LocalAddr::from(value);
        Self { num_rows: num_rows.resize(), num_cols: num_cols.resize(), local_addr }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigMvinRs1<const SCALE_BITS: usize, const STRIDE_BITS: usize, const PIXEL_REPEAT_BITS: usize> {
    pub scale: U<SCALE_BITS>,
    pub stride: U<STRIDE_BITS>,
    pub pixel_repeats: U<PIXEL_REPEAT_BITS>,
    pub state_id: U<2>, // TODO: Change bitwidth
    pub shrink: bool,   // TODO: Change bitwidth
}

impl<const SCALE_BITS: usize, const STRIDE_BITS: usize, const PIXEL_REPEAT_BITS: usize> From<U<64>>
    for ConfigMvinRs1<SCALE_BITS, STRIDE_BITS, PIXEL_REPEAT_BITS>
{
    fn from(value: U<64>) -> Self {
        let scale = value.clip_const::<32>(32);
        let stride = value.clip_const::<16>(16);
        let pixel_repeats = value.clip_const::<8>(8);
        let state_id = value.clip_const::<2>(3);
        let shrink = value[2];
        Self { scale: scale.resize(), stride: stride.resize(), pixel_repeats: pixel_repeats.resize(), state_id, shrink }
    }
}

pub const CONFIG_MVOUT_RS1_CMD_TYPE_WIDTH: usize = 2;
pub const CONFIG_MVOUT_RS1_ACTIVATION_WIDTH: usize = 2;
pub const CONFIG_MVOUT_RS1_MAX_POOLING_STRIDE_WIDTH: usize = 2;
pub const CONFIG_MVOUT_RS1_MAX_POOLING_WINDOW_SIZE_WIDTH: usize = 2;
pub const CONFIG_MVOUT_RS1_UPPER_ZERO_PADDING_WIDTH: usize = 2;
pub const CONFIG_MVOUT_RS1_LEFT_ZERO_PADDING_WIDTH: usize = 2;
pub const CONFIG_MVOUT_RS1_SPACER_WIDTH: usize = 24 - 2 * 6;
pub const CONFIG_MVOUT_RS1_POOL_OUT_DIM_WIDTH: usize = 8;
pub const CONFIG_MVOUT_RS1_POOL_OUT_ROWS_WIDTH: usize = 8;
pub const CONFIG_MVOUT_RS1_POOL_OUT_COLS_WIDTH: usize = 8;
pub const CONFIG_MVOUT_RS1_OUT_ROWS_WIDTH: usize = 8;
pub const CONFIG_MVOUT_RS1_OUT_COLS_WIDTH: usize = 8;

#[derive(Debug, Clone, Copy)]
pub struct ConfigMvoutRs1 {
    pub ocols: U<CONFIG_MVOUT_RS1_OUT_COLS_WIDTH>,
    pub orows: U<CONFIG_MVOUT_RS1_OUT_ROWS_WIDTH>,
    pub pocols: U<CONFIG_MVOUT_RS1_POOL_OUT_COLS_WIDTH>,
    pub porows: U<CONFIG_MVOUT_RS1_POOL_OUT_ROWS_WIDTH>,
    pub pool_out_dim: U<CONFIG_MVOUT_RS1_POOL_OUT_DIM_WIDTH>,
    pub _spacer: U<CONFIG_MVOUT_RS1_SPACER_WIDTH>,
    pub lpad: U<CONFIG_MVOUT_RS1_LEFT_ZERO_PADDING_WIDTH>,
    pub upad: U<CONFIG_MVOUT_RS1_UPPER_ZERO_PADDING_WIDTH>,
    pub pool_size: U<CONFIG_MVOUT_RS1_MAX_POOLING_WINDOW_SIZE_WIDTH>,
    pub pool_stride: U<CONFIG_MVOUT_RS1_MAX_POOLING_STRIDE_WIDTH>,
    pub activation: U<CONFIG_MVOUT_RS1_ACTIVATION_WIDTH>,
    pub cmd_type: U<CONFIG_MVOUT_RS1_CMD_TYPE_WIDTH>,
}

impl From<U<64>> for ConfigMvoutRs1 {
    fn from(value: U<64>) -> Self {
        let index = 64 - CONFIG_MVOUT_RS1_OUT_COLS_WIDTH;
        let ocols = value.clip_const::<CONFIG_MVOUT_RS1_OUT_COLS_WIDTH>(index);
        let index = index - CONFIG_MVOUT_RS1_OUT_ROWS_WIDTH;
        let orows = value.clip_const::<CONFIG_MVOUT_RS1_OUT_ROWS_WIDTH>(index);
        let index = index - CONFIG_MVOUT_RS1_POOL_OUT_COLS_WIDTH;
        let pocols = value.clip_const::<CONFIG_MVOUT_RS1_POOL_OUT_COLS_WIDTH>(index);
        let index = index - CONFIG_MVOUT_RS1_POOL_OUT_ROWS_WIDTH;
        let porows = value.clip_const::<CONFIG_MVOUT_RS1_POOL_OUT_ROWS_WIDTH>(index);
        let index = index - CONFIG_MVOUT_RS1_POOL_OUT_DIM_WIDTH;
        let pool_out_dim = value.clip_const::<CONFIG_MVOUT_RS1_POOL_OUT_DIM_WIDTH>(index);
        let index = index - CONFIG_MVOUT_RS1_SPACER_WIDTH;
        let _spacer = value.clip_const::<CONFIG_MVOUT_RS1_SPACER_WIDTH>(index);
        let index = index - CONFIG_MVOUT_RS1_LEFT_ZERO_PADDING_WIDTH;
        let lpad = value.clip_const::<CONFIG_MVOUT_RS1_LEFT_ZERO_PADDING_WIDTH>(index);
        let index = index - CONFIG_MVOUT_RS1_UPPER_ZERO_PADDING_WIDTH;
        let upad = value.clip_const::<CONFIG_MVOUT_RS1_UPPER_ZERO_PADDING_WIDTH>(index);
        let index = index - CONFIG_MVOUT_RS1_MAX_POOLING_WINDOW_SIZE_WIDTH;
        let pool_size = value.clip_const::<CONFIG_MVOUT_RS1_MAX_POOLING_WINDOW_SIZE_WIDTH>(index);
        let index = index - CONFIG_MVOUT_RS1_MAX_POOLING_STRIDE_WIDTH;
        let pool_stride = value.clip_const::<CONFIG_MVOUT_RS1_MAX_POOLING_STRIDE_WIDTH>(index);
        let index = index - CONFIG_MVOUT_RS1_ACTIVATION_WIDTH;
        let activation = value.clip_const::<CONFIG_MVOUT_RS1_ACTIVATION_WIDTH>(index);
        let index = index - CONFIG_MVOUT_RS1_CMD_TYPE_WIDTH;
        let cmd_type = value.clip_const::<CONFIG_MVOUT_RS1_CMD_TYPE_WIDTH>(index);
        Self {
            ocols,
            orows,
            pocols,
            porows,
            pool_out_dim,
            _spacer,
            lpad,
            upad,
            pool_size,
            pool_stride,
            activation,
            cmd_type,
        }
    }
}

pub const CONFIG_MVOUT_RS2_ACC_SCALE_WIDTH: usize = 32;
pub const CONFIG_MVOUT_RS2_STRIDE_WIDTH: usize = 32;

#[derive(Debug, Clone, Copy)]
pub struct ConfigMvoutRs2<const ACC_SCALE_BITS: usize, const STRIDE_BITS: usize> {
    pub acc_scale: U<ACC_SCALE_BITS>,
    pub stride: U<STRIDE_BITS>,
}

impl<const ACC_SCALE_BITS: usize, const STRIDE_BITS: usize> From<U<64>>
    for ConfigMvoutRs2<ACC_SCALE_BITS, STRIDE_BITS>
{
    fn from(value: U<64>) -> Self {
        let index = 64 - CONFIG_MVOUT_RS2_ACC_SCALE_WIDTH;
        let acc_scale = value.clip_const::<CONFIG_MVOUT_RS2_ACC_SCALE_WIDTH>(index);
        let index = index - CONFIG_MVOUT_RS2_STRIDE_WIDTH;
        let stride = value.clip_const::<CONFIG_MVOUT_RS2_STRIDE_WIDTH>(index);
        Self { acc_scale: acc_scale.resize(), stride: stride.resize() }
    }
}

pub const CONFIG_NORM_RS1_Q_CONST_WIDTH: usize = 32;
pub const CONFIG_NORM_RS1_SPACER1_WIDTH: usize = 13;
pub const CONFIG_NORM_RS1_Q_CONST_TYPE_WIDTH: usize = 1;
pub const CONFIG_NORM_RS1_SET_STATS_ID_ONLY_WIDTH: usize = 1;
pub const CONFIG_NORM_RS1_ACT_MSB_WIDTH: usize = 1;
pub const CONFIG_NORM_RS1_NORM_STATS_ID_WIDTH: usize = 8;
pub const CONFIG_NORM_RS1_SPACER0_WIDTH: usize = 6;
pub const CONFIG_NORM_RS1_CMD_TYPE_WIDTH: usize = 2;

#[derive(Debug, Clone, Copy)]
pub struct ConfigNormRs1<const ACC_BITS: usize> {
    pub q_const: U<ACC_BITS>,
    pub q_const_type: U<CONFIG_NORM_RS1_Q_CONST_TYPE_WIDTH>,
    pub set_stats_id_only: U<CONFIG_NORM_RS1_SET_STATS_ID_ONLY_WIDTH>,
    pub act_msb: U<CONFIG_NORM_RS1_ACT_MSB_WIDTH>,
    pub norm_stats_id: U<CONFIG_NORM_RS1_NORM_STATS_ID_WIDTH>,
    pub cmd_type: U<CONFIG_NORM_RS1_CMD_TYPE_WIDTH>,
}

impl<const ACC_BITS: usize> From<U<64>> for ConfigNormRs1<ACC_BITS> {
    fn from(value: U<64>) -> Self {
        let index = 64 - CONFIG_NORM_RS1_Q_CONST_WIDTH;
        let q_const = value.clip_const::<CONFIG_NORM_RS1_Q_CONST_WIDTH>(index);
        let index = index - CONFIG_NORM_RS1_SPACER1_WIDTH;
        let _spacer1 = value.clip_const::<CONFIG_NORM_RS1_SPACER1_WIDTH>(index);
        let index = index - CONFIG_NORM_RS1_Q_CONST_TYPE_WIDTH;
        let q_const_type = value.clip_const::<CONFIG_NORM_RS1_Q_CONST_TYPE_WIDTH>(index);
        let index = index - CONFIG_NORM_RS1_SET_STATS_ID_ONLY_WIDTH;
        let set_stats_id_only = value.clip_const::<CONFIG_NORM_RS1_SET_STATS_ID_ONLY_WIDTH>(index);
        let index = index - CONFIG_NORM_RS1_ACT_MSB_WIDTH;
        let act_msb = value.clip_const::<CONFIG_NORM_RS1_ACT_MSB_WIDTH>(index);
        let index = index - CONFIG_NORM_RS1_NORM_STATS_ID_WIDTH;
        let norm_stats_id = value.clip_const::<CONFIG_NORM_RS1_NORM_STATS_ID_WIDTH>(index);
        let index = index - CONFIG_NORM_RS1_SPACER0_WIDTH;
        let _spacer0 = value.clip_const::<CONFIG_NORM_RS1_SPACER0_WIDTH>(index);
        let index = index - CONFIG_NORM_RS1_CMD_TYPE_WIDTH;
        let cmd_type = value.clip_const::<CONFIG_NORM_RS1_CMD_TYPE_WIDTH>(index);
        Self { q_const: q_const.resize(), q_const_type, set_stats_id_only, act_msb, norm_stats_id, cmd_type }
    }
}

pub const CONFIG_NORM_RS2_QC_WIDTH: usize = 32;
pub const CONFIG_NORM_RS2_QB_WIDTH: usize = 32;

#[derive(Debug, Clone, Copy)]
pub struct ConfigNormRs2<const ACC_BITS: usize> {
    pub qc: U<ACC_BITS>,
    pub qb: U<ACC_BITS>,
}

impl<const ACC_BITS: usize> From<U<64>> for ConfigNormRs2<ACC_BITS> {
    fn from(value: U<64>) -> Self {
        let index = 64 - CONFIG_NORM_RS2_QC_WIDTH;
        let qc = value.clip_const::<CONFIG_NORM_RS2_QC_WIDTH>(index);
        let index = index - CONFIG_NORM_RS2_QB_WIDTH;
        let qb = value.clip_const::<CONFIG_NORM_RS2_QB_WIDTH>(index);
        Self { qc: qc.resize(), qb: qb.resize() }
    }
}
