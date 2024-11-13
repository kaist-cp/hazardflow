module ExecuteControllerBlackBoxAdapter(
  input          clock,
                 reset,
                 io_cmd_valid,
  input  [6:0]   io_cmd_bits_cmd_inst_funct,
  input  [4:0]   io_cmd_bits_cmd_inst_rs2,
                 io_cmd_bits_cmd_inst_rs1,
  input          io_cmd_bits_cmd_inst_xd,
                 io_cmd_bits_cmd_inst_xs1,
                 io_cmd_bits_cmd_inst_xs2,
  input  [4:0]   io_cmd_bits_cmd_inst_rd,
  input  [6:0]   io_cmd_bits_cmd_inst_opcode,
  input  [63:0]  io_cmd_bits_cmd_rs1,
                 io_cmd_bits_cmd_rs2,
  input          io_cmd_bits_cmd_status_debug,
                 io_cmd_bits_cmd_status_cease,
                 io_cmd_bits_cmd_status_wfi,
  input  [31:0]  io_cmd_bits_cmd_status_isa,
  input  [1:0]   io_cmd_bits_cmd_status_dprv,
  input          io_cmd_bits_cmd_status_dv,
  input  [1:0]   io_cmd_bits_cmd_status_prv,
  input          io_cmd_bits_cmd_status_v,
                 io_cmd_bits_cmd_status_sd,
  input  [22:0]  io_cmd_bits_cmd_status_zero2,
  input          io_cmd_bits_cmd_status_mpv,
                 io_cmd_bits_cmd_status_gva,
                 io_cmd_bits_cmd_status_mbe,
                 io_cmd_bits_cmd_status_sbe,
  input  [1:0]   io_cmd_bits_cmd_status_sxl,
                 io_cmd_bits_cmd_status_uxl,
  input          io_cmd_bits_cmd_status_sd_rv32,
  input  [7:0]   io_cmd_bits_cmd_status_zero1,
  input          io_cmd_bits_cmd_status_tsr,
                 io_cmd_bits_cmd_status_tw,
                 io_cmd_bits_cmd_status_tvm,
                 io_cmd_bits_cmd_status_mxr,
                 io_cmd_bits_cmd_status_sum,
                 io_cmd_bits_cmd_status_mprv,
  input  [1:0]   io_cmd_bits_cmd_status_xs,
                 io_cmd_bits_cmd_status_fs,
                 io_cmd_bits_cmd_status_mpp,
                 io_cmd_bits_cmd_status_vs,
  input          io_cmd_bits_cmd_status_spp,
                 io_cmd_bits_cmd_status_mpie,
                 io_cmd_bits_cmd_status_ube,
                 io_cmd_bits_cmd_status_spie,
                 io_cmd_bits_cmd_status_upie,
                 io_cmd_bits_cmd_status_mie,
                 io_cmd_bits_cmd_status_hie,
                 io_cmd_bits_cmd_status_sie,
                 io_cmd_bits_cmd_status_uie,
                 io_cmd_bits_rob_id_valid,
  input  [5:0]   io_cmd_bits_rob_id_bits,
  input          io_cmd_bits_from_matmul_fsm,
                 io_cmd_bits_from_conv_fsm,
                 io_srams_read_0_req_ready,
                 io_srams_read_0_resp_valid,
  input  [127:0] io_srams_read_0_resp_bits_data,
  input          io_srams_read_0_resp_bits_fromDMA,
                 io_srams_read_1_req_ready,
                 io_srams_read_1_resp_valid,
  input  [127:0] io_srams_read_1_resp_bits_data,
  input          io_srams_read_1_resp_bits_fromDMA,
                 io_srams_read_2_req_ready,
                 io_srams_read_2_resp_valid,
  input  [127:0] io_srams_read_2_resp_bits_data,
  input          io_srams_read_2_resp_bits_fromDMA,
                 io_srams_read_3_req_ready,
                 io_srams_read_3_resp_valid,
  input  [127:0] io_srams_read_3_resp_bits_data,
  input          io_srams_read_3_resp_bits_fromDMA,
                 io_acc_read_req_0_ready,
                 io_acc_read_req_1_ready,
                 io_acc_read_resp_0_valid,
  input  [7:0]   io_acc_read_resp_0_bits_full_data_0_0,
                 io_acc_read_resp_0_bits_full_data_1_0,
                 io_acc_read_resp_0_bits_full_data_2_0,
                 io_acc_read_resp_0_bits_full_data_3_0,
                 io_acc_read_resp_0_bits_full_data_4_0,
                 io_acc_read_resp_0_bits_full_data_5_0,
                 io_acc_read_resp_0_bits_full_data_6_0,
                 io_acc_read_resp_0_bits_full_data_7_0,
                 io_acc_read_resp_0_bits_full_data_8_0,
                 io_acc_read_resp_0_bits_full_data_9_0,
                 io_acc_read_resp_0_bits_full_data_10_0,
                 io_acc_read_resp_0_bits_full_data_11_0,
                 io_acc_read_resp_0_bits_full_data_12_0,
                 io_acc_read_resp_0_bits_full_data_13_0,
                 io_acc_read_resp_0_bits_full_data_14_0,
                 io_acc_read_resp_0_bits_full_data_15_0,
  input  [31:0]  io_acc_read_resp_0_bits_data_0_0,
                 io_acc_read_resp_0_bits_data_1_0,
                 io_acc_read_resp_0_bits_data_2_0,
                 io_acc_read_resp_0_bits_data_3_0,
                 io_acc_read_resp_0_bits_data_4_0,
                 io_acc_read_resp_0_bits_data_5_0,
                 io_acc_read_resp_0_bits_data_6_0,
                 io_acc_read_resp_0_bits_data_7_0,
                 io_acc_read_resp_0_bits_data_8_0,
                 io_acc_read_resp_0_bits_data_9_0,
                 io_acc_read_resp_0_bits_data_10_0,
                 io_acc_read_resp_0_bits_data_11_0,
                 io_acc_read_resp_0_bits_data_12_0,
                 io_acc_read_resp_0_bits_data_13_0,
                 io_acc_read_resp_0_bits_data_14_0,
                 io_acc_read_resp_0_bits_data_15_0,
  input  [1:0]   io_acc_read_resp_0_bits_acc_bank_id,
  input          io_acc_read_resp_0_bits_fromDMA,
                 io_acc_read_resp_1_valid,
  input  [7:0]   io_acc_read_resp_1_bits_full_data_0_0,
                 io_acc_read_resp_1_bits_full_data_1_0,
                 io_acc_read_resp_1_bits_full_data_2_0,
                 io_acc_read_resp_1_bits_full_data_3_0,
                 io_acc_read_resp_1_bits_full_data_4_0,
                 io_acc_read_resp_1_bits_full_data_5_0,
                 io_acc_read_resp_1_bits_full_data_6_0,
                 io_acc_read_resp_1_bits_full_data_7_0,
                 io_acc_read_resp_1_bits_full_data_8_0,
                 io_acc_read_resp_1_bits_full_data_9_0,
                 io_acc_read_resp_1_bits_full_data_10_0,
                 io_acc_read_resp_1_bits_full_data_11_0,
                 io_acc_read_resp_1_bits_full_data_12_0,
                 io_acc_read_resp_1_bits_full_data_13_0,
                 io_acc_read_resp_1_bits_full_data_14_0,
                 io_acc_read_resp_1_bits_full_data_15_0,
  input  [31:0]  io_acc_read_resp_1_bits_data_0_0,
                 io_acc_read_resp_1_bits_data_1_0,
                 io_acc_read_resp_1_bits_data_2_0,
                 io_acc_read_resp_1_bits_data_3_0,
                 io_acc_read_resp_1_bits_data_4_0,
                 io_acc_read_resp_1_bits_data_5_0,
                 io_acc_read_resp_1_bits_data_6_0,
                 io_acc_read_resp_1_bits_data_7_0,
                 io_acc_read_resp_1_bits_data_8_0,
                 io_acc_read_resp_1_bits_data_9_0,
                 io_acc_read_resp_1_bits_data_10_0,
                 io_acc_read_resp_1_bits_data_11_0,
                 io_acc_read_resp_1_bits_data_12_0,
                 io_acc_read_resp_1_bits_data_13_0,
                 io_acc_read_resp_1_bits_data_14_0,
                 io_acc_read_resp_1_bits_data_15_0,
  input  [1:0]   io_acc_read_resp_1_bits_acc_bank_id,
  input          io_acc_read_resp_1_bits_fromDMA,
                 io_acc_write_0_ready,
                 io_acc_write_1_ready,
  output         io_cmd_ready,
                 io_srams_read_0_req_valid,
  output [11:0]  io_srams_read_0_req_bits_addr,
  output         io_srams_read_0_req_bits_fromDMA,
                 io_srams_read_0_resp_ready,
                 io_srams_read_1_req_valid,
  output [11:0]  io_srams_read_1_req_bits_addr,
  output         io_srams_read_1_req_bits_fromDMA,
                 io_srams_read_1_resp_ready,
                 io_srams_read_2_req_valid,
  output [11:0]  io_srams_read_2_req_bits_addr,
  output         io_srams_read_2_req_bits_fromDMA,
                 io_srams_read_2_resp_ready,
                 io_srams_read_3_req_valid,
  output [11:0]  io_srams_read_3_req_bits_addr,
  output         io_srams_read_3_req_bits_fromDMA,
                 io_srams_read_3_resp_ready,
                 io_srams_write_0_en,
  output [11:0]  io_srams_write_0_addr,
  output         io_srams_write_0_mask_0,
                 io_srams_write_0_mask_1,
                 io_srams_write_0_mask_2,
                 io_srams_write_0_mask_3,
                 io_srams_write_0_mask_4,
                 io_srams_write_0_mask_5,
                 io_srams_write_0_mask_6,
                 io_srams_write_0_mask_7,
                 io_srams_write_0_mask_8,
                 io_srams_write_0_mask_9,
                 io_srams_write_0_mask_10,
                 io_srams_write_0_mask_11,
                 io_srams_write_0_mask_12,
                 io_srams_write_0_mask_13,
                 io_srams_write_0_mask_14,
                 io_srams_write_0_mask_15,
  output [127:0] io_srams_write_0_data,
  output         io_srams_write_1_en,
  output [11:0]  io_srams_write_1_addr,
  output         io_srams_write_1_mask_0,
                 io_srams_write_1_mask_1,
                 io_srams_write_1_mask_2,
                 io_srams_write_1_mask_3,
                 io_srams_write_1_mask_4,
                 io_srams_write_1_mask_5,
                 io_srams_write_1_mask_6,
                 io_srams_write_1_mask_7,
                 io_srams_write_1_mask_8,
                 io_srams_write_1_mask_9,
                 io_srams_write_1_mask_10,
                 io_srams_write_1_mask_11,
                 io_srams_write_1_mask_12,
                 io_srams_write_1_mask_13,
                 io_srams_write_1_mask_14,
                 io_srams_write_1_mask_15,
  output [127:0] io_srams_write_1_data,
  output         io_srams_write_2_en,
  output [11:0]  io_srams_write_2_addr,
  output         io_srams_write_2_mask_0,
                 io_srams_write_2_mask_1,
                 io_srams_write_2_mask_2,
                 io_srams_write_2_mask_3,
                 io_srams_write_2_mask_4,
                 io_srams_write_2_mask_5,
                 io_srams_write_2_mask_6,
                 io_srams_write_2_mask_7,
                 io_srams_write_2_mask_8,
                 io_srams_write_2_mask_9,
                 io_srams_write_2_mask_10,
                 io_srams_write_2_mask_11,
                 io_srams_write_2_mask_12,
                 io_srams_write_2_mask_13,
                 io_srams_write_2_mask_14,
                 io_srams_write_2_mask_15,
  output [127:0] io_srams_write_2_data,
  output         io_srams_write_3_en,
  output [11:0]  io_srams_write_3_addr,
  output         io_srams_write_3_mask_0,
                 io_srams_write_3_mask_1,
                 io_srams_write_3_mask_2,
                 io_srams_write_3_mask_3,
                 io_srams_write_3_mask_4,
                 io_srams_write_3_mask_5,
                 io_srams_write_3_mask_6,
                 io_srams_write_3_mask_7,
                 io_srams_write_3_mask_8,
                 io_srams_write_3_mask_9,
                 io_srams_write_3_mask_10,
                 io_srams_write_3_mask_11,
                 io_srams_write_3_mask_12,
                 io_srams_write_3_mask_13,
                 io_srams_write_3_mask_14,
                 io_srams_write_3_mask_15,
  output [127:0] io_srams_write_3_data,
  output         io_acc_read_req_0_valid,
  output [31:0]  io_acc_read_req_0_bits_scale_bits,
  output [8:0]   io_acc_read_req_0_bits_addr,
  output [31:0]  io_acc_read_req_0_bits_igelu_qb,
                 io_acc_read_req_0_bits_igelu_qc,
                 io_acc_read_req_0_bits_iexp_qln2,
                 io_acc_read_req_0_bits_iexp_qln2_inv,
  output [2:0]   io_acc_read_req_0_bits_act,
  output         io_acc_read_req_0_bits_full,
                 io_acc_read_req_0_bits_fromDMA,
                 io_acc_read_req_1_valid,
  output [31:0]  io_acc_read_req_1_bits_scale_bits,
  output [8:0]   io_acc_read_req_1_bits_addr,
  output [31:0]  io_acc_read_req_1_bits_igelu_qb,
                 io_acc_read_req_1_bits_igelu_qc,
                 io_acc_read_req_1_bits_iexp_qln2,
                 io_acc_read_req_1_bits_iexp_qln2_inv,
  output [2:0]   io_acc_read_req_1_bits_act,
  output         io_acc_read_req_1_bits_full,
                 io_acc_read_req_1_bits_fromDMA,
                 io_acc_read_resp_0_ready,
                 io_acc_read_resp_1_ready,
                 io_acc_write_0_valid,
  output [8:0]   io_acc_write_0_bits_addr,
  output [31:0]  io_acc_write_0_bits_data_0_0,
                 io_acc_write_0_bits_data_1_0,
                 io_acc_write_0_bits_data_2_0,
                 io_acc_write_0_bits_data_3_0,
                 io_acc_write_0_bits_data_4_0,
                 io_acc_write_0_bits_data_5_0,
                 io_acc_write_0_bits_data_6_0,
                 io_acc_write_0_bits_data_7_0,
                 io_acc_write_0_bits_data_8_0,
                 io_acc_write_0_bits_data_9_0,
                 io_acc_write_0_bits_data_10_0,
                 io_acc_write_0_bits_data_11_0,
                 io_acc_write_0_bits_data_12_0,
                 io_acc_write_0_bits_data_13_0,
                 io_acc_write_0_bits_data_14_0,
                 io_acc_write_0_bits_data_15_0,
  output         io_acc_write_0_bits_acc,
                 io_acc_write_0_bits_mask_0,
                 io_acc_write_0_bits_mask_1,
                 io_acc_write_0_bits_mask_2,
                 io_acc_write_0_bits_mask_3,
                 io_acc_write_0_bits_mask_4,
                 io_acc_write_0_bits_mask_5,
                 io_acc_write_0_bits_mask_6,
                 io_acc_write_0_bits_mask_7,
                 io_acc_write_0_bits_mask_8,
                 io_acc_write_0_bits_mask_9,
                 io_acc_write_0_bits_mask_10,
                 io_acc_write_0_bits_mask_11,
                 io_acc_write_0_bits_mask_12,
                 io_acc_write_0_bits_mask_13,
                 io_acc_write_0_bits_mask_14,
                 io_acc_write_0_bits_mask_15,
                 io_acc_write_0_bits_mask_16,
                 io_acc_write_0_bits_mask_17,
                 io_acc_write_0_bits_mask_18,
                 io_acc_write_0_bits_mask_19,
                 io_acc_write_0_bits_mask_20,
                 io_acc_write_0_bits_mask_21,
                 io_acc_write_0_bits_mask_22,
                 io_acc_write_0_bits_mask_23,
                 io_acc_write_0_bits_mask_24,
                 io_acc_write_0_bits_mask_25,
                 io_acc_write_0_bits_mask_26,
                 io_acc_write_0_bits_mask_27,
                 io_acc_write_0_bits_mask_28,
                 io_acc_write_0_bits_mask_29,
                 io_acc_write_0_bits_mask_30,
                 io_acc_write_0_bits_mask_31,
                 io_acc_write_0_bits_mask_32,
                 io_acc_write_0_bits_mask_33,
                 io_acc_write_0_bits_mask_34,
                 io_acc_write_0_bits_mask_35,
                 io_acc_write_0_bits_mask_36,
                 io_acc_write_0_bits_mask_37,
                 io_acc_write_0_bits_mask_38,
                 io_acc_write_0_bits_mask_39,
                 io_acc_write_0_bits_mask_40,
                 io_acc_write_0_bits_mask_41,
                 io_acc_write_0_bits_mask_42,
                 io_acc_write_0_bits_mask_43,
                 io_acc_write_0_bits_mask_44,
                 io_acc_write_0_bits_mask_45,
                 io_acc_write_0_bits_mask_46,
                 io_acc_write_0_bits_mask_47,
                 io_acc_write_0_bits_mask_48,
                 io_acc_write_0_bits_mask_49,
                 io_acc_write_0_bits_mask_50,
                 io_acc_write_0_bits_mask_51,
                 io_acc_write_0_bits_mask_52,
                 io_acc_write_0_bits_mask_53,
                 io_acc_write_0_bits_mask_54,
                 io_acc_write_0_bits_mask_55,
                 io_acc_write_0_bits_mask_56,
                 io_acc_write_0_bits_mask_57,
                 io_acc_write_0_bits_mask_58,
                 io_acc_write_0_bits_mask_59,
                 io_acc_write_0_bits_mask_60,
                 io_acc_write_0_bits_mask_61,
                 io_acc_write_0_bits_mask_62,
                 io_acc_write_0_bits_mask_63,
                 io_acc_write_1_valid,
  output [8:0]   io_acc_write_1_bits_addr,
  output [31:0]  io_acc_write_1_bits_data_0_0,
                 io_acc_write_1_bits_data_1_0,
                 io_acc_write_1_bits_data_2_0,
                 io_acc_write_1_bits_data_3_0,
                 io_acc_write_1_bits_data_4_0,
                 io_acc_write_1_bits_data_5_0,
                 io_acc_write_1_bits_data_6_0,
                 io_acc_write_1_bits_data_7_0,
                 io_acc_write_1_bits_data_8_0,
                 io_acc_write_1_bits_data_9_0,
                 io_acc_write_1_bits_data_10_0,
                 io_acc_write_1_bits_data_11_0,
                 io_acc_write_1_bits_data_12_0,
                 io_acc_write_1_bits_data_13_0,
                 io_acc_write_1_bits_data_14_0,
                 io_acc_write_1_bits_data_15_0,
  output         io_acc_write_1_bits_acc,
                 io_acc_write_1_bits_mask_0,
                 io_acc_write_1_bits_mask_1,
                 io_acc_write_1_bits_mask_2,
                 io_acc_write_1_bits_mask_3,
                 io_acc_write_1_bits_mask_4,
                 io_acc_write_1_bits_mask_5,
                 io_acc_write_1_bits_mask_6,
                 io_acc_write_1_bits_mask_7,
                 io_acc_write_1_bits_mask_8,
                 io_acc_write_1_bits_mask_9,
                 io_acc_write_1_bits_mask_10,
                 io_acc_write_1_bits_mask_11,
                 io_acc_write_1_bits_mask_12,
                 io_acc_write_1_bits_mask_13,
                 io_acc_write_1_bits_mask_14,
                 io_acc_write_1_bits_mask_15,
                 io_acc_write_1_bits_mask_16,
                 io_acc_write_1_bits_mask_17,
                 io_acc_write_1_bits_mask_18,
                 io_acc_write_1_bits_mask_19,
                 io_acc_write_1_bits_mask_20,
                 io_acc_write_1_bits_mask_21,
                 io_acc_write_1_bits_mask_22,
                 io_acc_write_1_bits_mask_23,
                 io_acc_write_1_bits_mask_24,
                 io_acc_write_1_bits_mask_25,
                 io_acc_write_1_bits_mask_26,
                 io_acc_write_1_bits_mask_27,
                 io_acc_write_1_bits_mask_28,
                 io_acc_write_1_bits_mask_29,
                 io_acc_write_1_bits_mask_30,
                 io_acc_write_1_bits_mask_31,
                 io_acc_write_1_bits_mask_32,
                 io_acc_write_1_bits_mask_33,
                 io_acc_write_1_bits_mask_34,
                 io_acc_write_1_bits_mask_35,
                 io_acc_write_1_bits_mask_36,
                 io_acc_write_1_bits_mask_37,
                 io_acc_write_1_bits_mask_38,
                 io_acc_write_1_bits_mask_39,
                 io_acc_write_1_bits_mask_40,
                 io_acc_write_1_bits_mask_41,
                 io_acc_write_1_bits_mask_42,
                 io_acc_write_1_bits_mask_43,
                 io_acc_write_1_bits_mask_44,
                 io_acc_write_1_bits_mask_45,
                 io_acc_write_1_bits_mask_46,
                 io_acc_write_1_bits_mask_47,
                 io_acc_write_1_bits_mask_48,
                 io_acc_write_1_bits_mask_49,
                 io_acc_write_1_bits_mask_50,
                 io_acc_write_1_bits_mask_51,
                 io_acc_write_1_bits_mask_52,
                 io_acc_write_1_bits_mask_53,
                 io_acc_write_1_bits_mask_54,
                 io_acc_write_1_bits_mask_55,
                 io_acc_write_1_bits_mask_56,
                 io_acc_write_1_bits_mask_57,
                 io_acc_write_1_bits_mask_58,
                 io_acc_write_1_bits_mask_59,
                 io_acc_write_1_bits_mask_60,
                 io_acc_write_1_bits_mask_61,
                 io_acc_write_1_bits_mask_62,
                 io_acc_write_1_bits_mask_63,
                 io_completed_valid,
  output [5:0]   io_completed_bits
//   output         io_busy // TODO
);

    wire in_input_0_payload_discriminant = io_cmd_valid;
    wire [5-1:0] in_input_0_payload_Some_0_cmd_inst_funct_discriminant = io_cmd_bits_cmd_inst_funct;
    wire [5-1:0] in_input_0_payload_Some_0_cmd_inst_rs2 = io_cmd_bits_cmd_inst_rs2;
    wire [5-1:0] in_input_0_payload_Some_0_cmd_inst_rs1 = io_cmd_bits_cmd_inst_rs1;
    wire in_input_0_payload_Some_0_cmd_inst_xd = io_cmd_bits_cmd_inst_xd;
    wire in_input_0_payload_Some_0_cmd_inst_xs1 = io_cmd_bits_cmd_inst_xs1;
    wire in_input_0_payload_Some_0_cmd_inst_xs2 = io_cmd_bits_cmd_inst_xs2;
    wire [5-1:0] in_input_0_payload_Some_0_cmd_inst_rd = io_cmd_bits_cmd_inst_rd;
    wire [7-1:0] in_input_0_payload_Some_0_cmd_inst_opcode = io_cmd_bits_cmd_inst_opcode;
    wire [64-1:0] in_input_0_payload_Some_0_cmd_rs1 = io_cmd_bits_cmd_rs1;
    wire [64-1:0] in_input_0_payload_Some_0_cmd_rs2 = io_cmd_bits_cmd_rs2;
    wire in_input_0_payload_Some_0_cmd_status_debug = io_cmd_bits_cmd_status_debug;
    wire in_input_0_payload_Some_0_cmd_status_cease = io_cmd_bits_cmd_status_cease;
    wire in_input_0_payload_Some_0_cmd_status_wfi = io_cmd_bits_cmd_status_wfi;
    wire [32-1:0] in_input_0_payload_Some_0_cmd_status_isa = io_cmd_bits_cmd_status_isa;
    wire [2-1:0] in_input_0_payload_Some_0_cmd_status_dprv = io_cmd_bits_cmd_status_dprv;
    wire in_input_0_payload_Some_0_cmd_status_dv = io_cmd_bits_cmd_status_dv;
    wire [2-1:0] in_input_0_payload_Some_0_cmd_status_prv = io_cmd_bits_cmd_status_prv;
    wire in_input_0_payload_Some_0_cmd_status_v = io_cmd_bits_cmd_status_v;
    wire in_input_0_payload_Some_0_cmd_status_sd = io_cmd_bits_cmd_status_sd;
    wire [23-1:0] in_input_0_payload_Some_0_cmd_status_zero2 = io_cmd_bits_cmd_status_zero2;
    wire in_input_0_payload_Some_0_cmd_status_mpv = io_cmd_bits_cmd_status_mpv;
    wire in_input_0_payload_Some_0_cmd_status_gva = io_cmd_bits_cmd_status_gva;
    wire in_input_0_payload_Some_0_cmd_status_mbe = io_cmd_bits_cmd_status_mbe;
    wire in_input_0_payload_Some_0_cmd_status_sbe = io_cmd_bits_cmd_status_sbe;
    wire [2-1:0] in_input_0_payload_Some_0_cmd_status_sxl = io_cmd_bits_cmd_status_sxl;
    wire [2-1:0] in_input_0_payload_Some_0_cmd_status_uxl = io_cmd_bits_cmd_status_uxl;
    wire in_input_0_payload_Some_0_cmd_status_sd_rv32 = io_cmd_bits_cmd_status_sd_rv32;
    wire [8-1:0] in_input_0_payload_Some_0_cmd_status_zero1 = io_cmd_bits_cmd_status_zero1;
    wire in_input_0_payload_Some_0_cmd_status_tsr = io_cmd_bits_cmd_status_tsr;
    wire in_input_0_payload_Some_0_cmd_status_tw = io_cmd_bits_cmd_status_tw;
    wire in_input_0_payload_Some_0_cmd_status_tvm = io_cmd_bits_cmd_status_tvm;
    wire in_input_0_payload_Some_0_cmd_status_mxr = io_cmd_bits_cmd_status_mxr;
    wire in_input_0_payload_Some_0_cmd_status_sum = io_cmd_bits_cmd_status_sum;
    wire in_input_0_payload_Some_0_cmd_status_mprv = io_cmd_bits_cmd_status_mprv;
    wire [2-1:0] in_input_0_payload_Some_0_cmd_status_xs = io_cmd_bits_cmd_status_xs;
    wire [2-1:0] in_input_0_payload_Some_0_cmd_status_fs = io_cmd_bits_cmd_status_fs;
    wire [2-1:0] in_input_0_payload_Some_0_cmd_status_mpp = io_cmd_bits_cmd_status_mpp;
    wire [2-1:0] in_input_0_payload_Some_0_cmd_status_vs = io_cmd_bits_cmd_status_vs;
    wire in_input_0_payload_Some_0_cmd_status_spp = io_cmd_bits_cmd_status_spp;
    wire in_input_0_payload_Some_0_cmd_status_mpie = io_cmd_bits_cmd_status_mpie;
    wire in_input_0_payload_Some_0_cmd_status_ube = io_cmd_bits_cmd_status_ube;
    wire in_input_0_payload_Some_0_cmd_status_spie = io_cmd_bits_cmd_status_spie;
    wire in_input_0_payload_Some_0_cmd_status_upie = io_cmd_bits_cmd_status_upie;
    wire in_input_0_payload_Some_0_cmd_status_mie = io_cmd_bits_cmd_status_mie;
    wire in_input_0_payload_Some_0_cmd_status_hie = io_cmd_bits_cmd_status_hie;
    wire in_input_0_payload_Some_0_cmd_status_sie = io_cmd_bits_cmd_status_sie;
    wire in_input_0_payload_Some_0_cmd_status_uie = io_cmd_bits_cmd_status_uie;
    wire in_input_0_payload_Some_0_rob_id_discriminant = io_cmd_bits_rob_id_valid;
    wire [6-1:0] in_input_0_payload_Some_0_rob_id_Some_0 = io_cmd_bits_rob_id_bits;
    wire in_input_0_payload_Some_0_from_matmul_fsm = io_cmd_bits_from_matmul_fsm;
    wire in_input_0_payload_Some_0_from_conv_fsm = io_cmd_bits_from_conv_fsm;
    wire in_input_0_resolver_ready;
    assign io_cmd_ready = in_input_0_resolver_ready;

    // resp of spad_readers
    wire [4-1:0] in_input_1_output_payload_discriminant;
    assign in_input_1_output_payload_discriminant[0] = io_srams_read_0_resp_valid;
    assign in_input_1_output_payload_discriminant[1] = io_srams_read_1_resp_valid;
    assign in_input_1_output_payload_discriminant[2] = io_srams_read_2_resp_valid;
    assign in_input_1_output_payload_discriminant[3] = io_srams_read_3_resp_valid;
    wire [512-1:0] in_input_1_output_payload_Some_0_data;
    assign in_input_1_output_payload_Some_0_data[0*128 +: 128] = io_srams_read_0_resp_bits_data;
    assign in_input_1_output_payload_Some_0_data[1*128 +: 128] = io_srams_read_1_resp_bits_data;
    assign in_input_1_output_payload_Some_0_data[2*128 +: 128] = io_srams_read_2_resp_bits_data;
    assign in_input_1_output_payload_Some_0_data[3*128 +: 128] = io_srams_read_3_resp_bits_data;
    wire [4-1:0] in_input_1_output_payload_Some_0_from_dma;
    assign in_input_1_output_payload_Some_0_from_dma[0] = io_srams_read_0_resp_bits_fromDMA;
    assign in_input_1_output_payload_Some_0_from_dma[1] = io_srams_read_1_resp_bits_fromDMA;
    assign in_input_1_output_payload_Some_0_from_dma[2] = io_srams_read_2_resp_bits_fromDMA;
    assign in_input_1_output_payload_Some_0_from_dma[3] = io_srams_read_3_resp_bits_fromDMA;
    wire [4-1:0] in_input_1_output_resolver_ready;
    assign io_srams_read_0_resp_ready = in_input_1_output_resolver_ready[0];
    assign io_srams_read_1_resp_ready = in_input_1_output_resolver_ready[1];
    assign io_srams_read_2_resp_ready = in_input_1_output_resolver_ready[2];
    assign io_srams_read_3_resp_ready = in_input_1_output_resolver_ready[3];
    
    // req of spad_readers
    wire [4-1:0] out_input_1_input_0_payload_discriminant;
    assign io_srams_read_0_req_valid = out_input_1_input_0_payload_discriminant[0];
    assign io_srams_read_1_req_valid = out_input_1_input_0_payload_discriminant[1];
    assign io_srams_read_2_req_valid = out_input_1_input_0_payload_discriminant[2];
    assign io_srams_read_3_req_valid = out_input_1_input_0_payload_discriminant[3];
    wire [48-1:0] out_input_1_input_0_payload_Some_0_addr;
    assign io_srams_read_0_req_bits_addr = out_input_1_input_0_payload_Some_0_addr[0*12 +: 12];
    assign io_srams_read_1_req_bits_addr = out_input_1_input_0_payload_Some_0_addr[1*12 +: 12];
    assign io_srams_read_2_req_bits_addr = out_input_1_input_0_payload_Some_0_addr[2*12 +: 12];
    assign io_srams_read_3_req_bits_addr = out_input_1_input_0_payload_Some_0_addr[3*12 +: 12];
    wire [4-1:0] out_input_1_input_0_payload_Some_0_from_dma;
    assign io_srams_read_0_req_bits_fromDMA = out_input_1_input_0_payload_Some_0_from_dma[0];
    assign io_srams_read_1_req_bits_fromDMA = out_input_1_input_0_payload_Some_0_from_dma[1];
    assign io_srams_read_2_req_bits_fromDMA = out_input_1_input_0_payload_Some_0_from_dma[2];
    assign io_srams_read_3_req_bits_fromDMA = out_input_1_input_0_payload_Some_0_from_dma[3];
    wire [4-1:0] out_input_1_input_0_resolver_ready;
    assign out_input_1_input_0_resolver_ready[0] = io_srams_read_0_req_ready;
    assign out_input_1_input_0_resolver_ready[1] = io_srams_read_1_req_ready;
    assign out_input_1_input_0_resolver_ready[2] = io_srams_read_2_req_ready;
    assign out_input_1_input_0_resolver_ready[3] = io_srams_read_3_req_ready;

    // req of spad_writers
    wire [4-1:0] out_input_2_input_0_payload_discriminant;
    assign io_srams_write_0_en = out_input_2_input_0_payload_discriminant[0];
    assign io_srams_write_1_en = out_input_2_input_0_payload_discriminant[1];
    assign io_srams_write_2_en = out_input_2_input_0_payload_discriminant[2];
    assign io_srams_write_3_en = out_input_2_input_0_payload_discriminant[3];
    wire [48-1:0] out_input_2_input_0_payload_Some_0_addr;
    assign io_srams_write_0_addr = out_input_2_input_0_payload_Some_0_addr[0*12 +: 12];
    assign io_srams_write_1_addr = out_input_2_input_0_payload_Some_0_addr[1*12 +: 12];
    assign io_srams_write_2_addr = out_input_2_input_0_payload_Some_0_addr[2*12 +: 12];
    assign io_srams_write_3_addr = out_input_2_input_0_payload_Some_0_addr[3*12 +: 12];
    wire [512-1:0] out_input_2_input_0_payload_Some_0_data;
    assign io_srams_write_0_data = out_input_2_input_0_payload_Some_0_data[0*128 +: 128];
    assign io_srams_write_1_data = out_input_2_input_0_payload_Some_0_data[1*128 +: 128];
    assign io_srams_write_2_data = out_input_2_input_0_payload_Some_0_data[2*128 +: 128];
    assign io_srams_write_3_data = out_input_2_input_0_payload_Some_0_data[3*128 +: 128];
    wire [64-1:0] out_input_2_input_0_payload_Some_0_mask;
    assign io_srams_write_0_mask_0 = out_input_2_input_0_payload_Some_0_mask[0*16 + 0];
    assign io_srams_write_0_mask_1 = out_input_2_input_0_payload_Some_0_mask[0*16 + 1];
    assign io_srams_write_0_mask_2 = out_input_2_input_0_payload_Some_0_mask[0*16 + 2];
    assign io_srams_write_0_mask_3 = out_input_2_input_0_payload_Some_0_mask[0*16 + 3];
    assign io_srams_write_0_mask_4 = out_input_2_input_0_payload_Some_0_mask[0*16 + 4];
    assign io_srams_write_0_mask_5 = out_input_2_input_0_payload_Some_0_mask[0*16 + 5];
    assign io_srams_write_0_mask_6 = out_input_2_input_0_payload_Some_0_mask[0*16 + 6];
    assign io_srams_write_0_mask_7 = out_input_2_input_0_payload_Some_0_mask[0*16 + 7];
    assign io_srams_write_0_mask_8 = out_input_2_input_0_payload_Some_0_mask[0*16 + 8];
    assign io_srams_write_0_mask_9 = out_input_2_input_0_payload_Some_0_mask[0*16 + 9];
    assign io_srams_write_0_mask_10 = out_input_2_input_0_payload_Some_0_mask[0*16 + 10];
    assign io_srams_write_0_mask_11 = out_input_2_input_0_payload_Some_0_mask[0*16 + 11];
    assign io_srams_write_0_mask_12 = out_input_2_input_0_payload_Some_0_mask[0*16 + 12];
    assign io_srams_write_0_mask_13 = out_input_2_input_0_payload_Some_0_mask[0*16 + 13];
    assign io_srams_write_0_mask_14 = out_input_2_input_0_payload_Some_0_mask[0*16 + 14];
    assign io_srams_write_0_mask_15 = out_input_2_input_0_payload_Some_0_mask[0*16 + 15];
    assign io_srams_write_1_mask_0 = out_input_2_input_0_payload_Some_0_mask[1*16 + 0];
    assign io_srams_write_1_mask_1 = out_input_2_input_0_payload_Some_0_mask[1*16 + 1];
    assign io_srams_write_1_mask_2 = out_input_2_input_0_payload_Some_0_mask[1*16 + 2];
    assign io_srams_write_1_mask_3 = out_input_2_input_0_payload_Some_0_mask[1*16 + 3];
    assign io_srams_write_1_mask_4 = out_input_2_input_0_payload_Some_0_mask[1*16 + 4];
    assign io_srams_write_1_mask_5 = out_input_2_input_0_payload_Some_0_mask[1*16 + 5];
    assign io_srams_write_1_mask_6 = out_input_2_input_0_payload_Some_0_mask[1*16 + 6];
    assign io_srams_write_1_mask_7 = out_input_2_input_0_payload_Some_0_mask[1*16 + 7];
    assign io_srams_write_1_mask_8 = out_input_2_input_0_payload_Some_0_mask[1*16 + 8];
    assign io_srams_write_1_mask_9 = out_input_2_input_0_payload_Some_0_mask[1*16 + 9];
    assign io_srams_write_1_mask_10 = out_input_2_input_0_payload_Some_0_mask[1*16 + 10];
    assign io_srams_write_1_mask_11 = out_input_2_input_0_payload_Some_0_mask[1*16 + 11];
    assign io_srams_write_1_mask_12 = out_input_2_input_0_payload_Some_0_mask[1*16 + 12];
    assign io_srams_write_1_mask_13 = out_input_2_input_0_payload_Some_0_mask[1*16 + 13];
    assign io_srams_write_1_mask_14 = out_input_2_input_0_payload_Some_0_mask[1*16 + 14];
    assign io_srams_write_1_mask_15 = out_input_2_input_0_payload_Some_0_mask[1*16 + 15];
    assign io_srams_write_2_mask_0 = out_input_2_input_0_payload_Some_0_mask[2*16 + 0];
    assign io_srams_write_2_mask_1 = out_input_2_input_0_payload_Some_0_mask[2*16 + 1];
    assign io_srams_write_2_mask_2 = out_input_2_input_0_payload_Some_0_mask[2*16 + 2];
    assign io_srams_write_2_mask_3 = out_input_2_input_0_payload_Some_0_mask[2*16 + 3];
    assign io_srams_write_2_mask_4 = out_input_2_input_0_payload_Some_0_mask[2*16 + 4];
    assign io_srams_write_2_mask_5 = out_input_2_input_0_payload_Some_0_mask[2*16 + 5];
    assign io_srams_write_2_mask_6 = out_input_2_input_0_payload_Some_0_mask[2*16 + 6];
    assign io_srams_write_2_mask_7 = out_input_2_input_0_payload_Some_0_mask[2*16 + 7];
    assign io_srams_write_2_mask_8 = out_input_2_input_0_payload_Some_0_mask[2*16 + 8];
    assign io_srams_write_2_mask_9 = out_input_2_input_0_payload_Some_0_mask[2*16 + 9];
    assign io_srams_write_2_mask_10 = out_input_2_input_0_payload_Some_0_mask[2*16 + 10];
    assign io_srams_write_2_mask_11 = out_input_2_input_0_payload_Some_0_mask[2*16 + 11];
    assign io_srams_write_2_mask_12 = out_input_2_input_0_payload_Some_0_mask[2*16 + 12];
    assign io_srams_write_2_mask_13 = out_input_2_input_0_payload_Some_0_mask[2*16 + 13];
    assign io_srams_write_2_mask_14 = out_input_2_input_0_payload_Some_0_mask[2*16 + 14];
    assign io_srams_write_2_mask_15 = out_input_2_input_0_payload_Some_0_mask[2*16 + 15];
    assign io_srams_write_3_mask_0 = out_input_2_input_0_payload_Some_0_mask[3*16 + 0];
    assign io_srams_write_3_mask_1 = out_input_2_input_0_payload_Some_0_mask[3*16 + 1];
    assign io_srams_write_3_mask_2 = out_input_2_input_0_payload_Some_0_mask[3*16 + 2];
    assign io_srams_write_3_mask_3 = out_input_2_input_0_payload_Some_0_mask[3*16 + 3];
    assign io_srams_write_3_mask_4 = out_input_2_input_0_payload_Some_0_mask[3*16 + 4];
    assign io_srams_write_3_mask_5 = out_input_2_input_0_payload_Some_0_mask[3*16 + 5];
    assign io_srams_write_3_mask_6 = out_input_2_input_0_payload_Some_0_mask[3*16 + 6];
    assign io_srams_write_3_mask_7 = out_input_2_input_0_payload_Some_0_mask[3*16 + 7];
    assign io_srams_write_3_mask_8 = out_input_2_input_0_payload_Some_0_mask[3*16 + 8];
    assign io_srams_write_3_mask_9 = out_input_2_input_0_payload_Some_0_mask[3*16 + 9];
    assign io_srams_write_3_mask_10 = out_input_2_input_0_payload_Some_0_mask[3*16 + 10];
    assign io_srams_write_3_mask_11 = out_input_2_input_0_payload_Some_0_mask[3*16 + 11];
    assign io_srams_write_3_mask_12 = out_input_2_input_0_payload_Some_0_mask[3*16 + 12];
    assign io_srams_write_3_mask_13 = out_input_2_input_0_payload_Some_0_mask[3*16 + 13];
    assign io_srams_write_3_mask_14 = out_input_2_input_0_payload_Some_0_mask[3*16 + 14];
    assign io_srams_write_3_mask_15 = out_input_2_input_0_payload_Some_0_mask[3*16 + 15];

    
    // req of acc_readers
    wire [2-1:0] out_input_3_input_0_payload_discriminant;
    assign io_acc_read_req_0_valid = out_input_3_input_0_payload_discriminant[0];
    assign io_acc_read_req_1_valid = out_input_3_input_0_payload_discriminant[1];
    wire [64-1:0] out_input_3_input_0_payload_Some_0_scale;
    assign io_acc_read_req_0_bits_scale_bits = out_input_3_input_0_payload_Some_0_scale[0*32 +: 32];
    assign io_acc_read_req_1_bits_scale_bits = out_input_3_input_0_payload_Some_0_scale[1*32 +: 32];
    wire [2-1:0] out_input_3_input_0_payload_Some_0_full;
    assign io_acc_read_req_0_bits_full = out_input_3_input_0_payload_Some_0_full[0];
    assign io_acc_read_req_1_bits_full = out_input_3_input_0_payload_Some_0_full[1];
    wire [6-1:0] out_input_3_input_0_payload_Some_0_act;
    assign io_acc_read_req_0_bits_act = out_input_3_input_0_payload_Some_0_act[0*3 +: 3];
    assign io_acc_read_req_1_bits_act = out_input_3_input_0_payload_Some_0_act[1*3 +: 3];
    wire [2-1:0] out_input_3_input_0_payload_Some_0_from_dma;
    assign io_acc_read_req_0_bits_fromDMA = out_input_3_input_0_payload_Some_0_from_dma[0];
    assign io_acc_read_req_1_bits_fromDMA = out_input_3_input_0_payload_Some_0_from_dma[1];
    wire [64-1:0] out_input_3_input_0_payload_Some_0_addr;
    assign io_acc_read_req_0_bits_addr = out_input_3_input_0_payload_Some_0_addr[0*32 +: 9];
    assign io_acc_read_req_1_bits_addr = out_input_3_input_0_payload_Some_0_addr[1*32 +: 9];
    wire [2-1:0] out_input_3_input_0_resolver_ready;
    assign out_input_3_input_0_resolver_ready[0] = io_acc_read_req_0_ready;
    assign out_input_3_input_0_resolver_ready[1] = io_acc_read_req_1_ready;

    // resp of acc_readers
    wire [2-1:0] in_input_3_output_payload_discriminant;
    assign in_input_3_output_payload_discriminant[0] = io_acc_read_resp_0_valid;
    assign in_input_3_output_payload_discriminant[1] = io_acc_read_resp_1_valid;
    wire [256-1:0] in_input_3_output_payload_Some_0_data;
    assign in_input_3_output_payload_Some_0_data[0*128 +: 128] = {io_acc_read_resp_0_bits_data_3_0, io_acc_read_resp_0_bits_data_2_0, io_acc_read_resp_0_bits_data_1_0, io_acc_read_resp_0_bits_data_0_0};
    assign in_input_3_output_payload_Some_0_data[1*128 +: 128] = {io_acc_read_resp_1_bits_data_3_0, io_acc_read_resp_1_bits_data_2_0, io_acc_read_resp_1_bits_data_1_0, io_acc_read_resp_1_bits_data_0_0};
    wire [2-1:0] in_input_3_output_payload_Some_0_from_dma;
    assign in_input_3_output_payload_Some_0_from_dma[0] = io_acc_read_resp_0_bits_fromDMA;
    assign in_input_3_output_payload_Some_0_from_dma[1] = io_acc_read_resp_1_bits_fromDMA;
    wire [2-1:0] in_input_3_output_resolver_ready;
    assign io_acc_read_resp_0_ready = in_input_3_output_resolver_ready[0];
    assign io_acc_read_resp_1_ready = in_input_3_output_resolver_ready[1];

    // req of acc_writers
    wire [2-1:0] out_input_4_input_0_payload_discriminant;
    assign io_acc_write_0_valid = out_input_4_input_0_payload_discriminant[0];
    assign io_acc_write_1_valid = out_input_4_input_0_payload_discriminant[1];
    wire [18-1:0] out_input_4_input_0_payload_Some_0_addr;
    assign io_acc_write_0_bits_addr = out_input_4_input_0_payload_Some_0_addr[0*9 +: 9];
    assign io_acc_write_1_bits_addr = out_input_4_input_0_payload_Some_0_addr[1*9 +: 9];
    wire [1024-1:0] out_input_4_input_0_payload_Some_0_data;
    assign io_acc_write_0_bits_data_0_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 0*32 +: 32];
    assign io_acc_write_0_bits_data_1_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 1*32 +: 32];
    assign io_acc_write_0_bits_data_2_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 2*32 +: 32];
    assign io_acc_write_0_bits_data_3_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 3*32 +: 32];
    assign io_acc_write_0_bits_data_4_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 4*32 +: 32];
    assign io_acc_write_0_bits_data_5_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 5*32 +: 32];
    assign io_acc_write_0_bits_data_6_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 6*32 +: 32];
    assign io_acc_write_0_bits_data_7_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 7*32 +: 32];
    assign io_acc_write_0_bits_data_8_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 8*32 +: 32];
    assign io_acc_write_0_bits_data_9_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 9*32 +: 32];
    assign io_acc_write_0_bits_data_10_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 10*32 +: 32];
    assign io_acc_write_0_bits_data_11_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 11*32 +: 32];
    assign io_acc_write_0_bits_data_12_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 12*32 +: 32];
    assign io_acc_write_0_bits_data_13_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 13*32 +: 32];
    assign io_acc_write_0_bits_data_14_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 14*32 +: 32];
    assign io_acc_write_0_bits_data_15_0 = out_input_4_input_0_payload_Some_0_data[0*512 + 15*32 +: 32];
    assign io_acc_write_1_bits_data_0_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 0*32 +: 32];
    assign io_acc_write_1_bits_data_1_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 1*32 +: 32];
    assign io_acc_write_1_bits_data_2_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 2*32 +: 32];
    assign io_acc_write_1_bits_data_3_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 3*32 +: 32];
    assign io_acc_write_1_bits_data_4_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 4*32 +: 32];
    assign io_acc_write_1_bits_data_5_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 5*32 +: 32];
    assign io_acc_write_1_bits_data_6_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 6*32 +: 32];
    assign io_acc_write_1_bits_data_7_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 7*32 +: 32];
    assign io_acc_write_1_bits_data_8_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 8*32 +: 32];
    assign io_acc_write_1_bits_data_9_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 9*32 +: 32];
    assign io_acc_write_1_bits_data_10_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 10*32 +: 32];
    assign io_acc_write_1_bits_data_11_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 11*32 +: 32];
    assign io_acc_write_1_bits_data_12_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 12*32 +: 32];
    assign io_acc_write_1_bits_data_13_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 13*32 +: 32];
    assign io_acc_write_1_bits_data_14_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 14*32 +: 32];
    assign io_acc_write_1_bits_data_15_0 = out_input_4_input_0_payload_Some_0_data[1*512 + 15*32 +: 32];
    wire [2-1:0] out_input_4_input_0_payload_Some_0_acc;
    assign io_acc_write_0_bits_acc = out_input_4_input_0_payload_Some_0_acc[0*1 +: 1];
    assign io_acc_write_1_bits_acc = out_input_4_input_0_payload_Some_0_acc[1*1 +: 1];
    wire [128-1:0] out_input_4_input_0_payload_Some_0_mask;
    assign io_acc_write_0_bits_mask_0 = out_input_4_input_0_payload_Some_0_mask[0*64 + 0];
    assign io_acc_write_0_bits_mask_1 = out_input_4_input_0_payload_Some_0_mask[0*64 + 1];
    assign io_acc_write_0_bits_mask_2 = out_input_4_input_0_payload_Some_0_mask[0*64 + 2];
    assign io_acc_write_0_bits_mask_3 = out_input_4_input_0_payload_Some_0_mask[0*64 + 3];
    assign io_acc_write_0_bits_mask_4 = out_input_4_input_0_payload_Some_0_mask[0*64 + 4];
    assign io_acc_write_0_bits_mask_5 = out_input_4_input_0_payload_Some_0_mask[0*64 + 5];
    assign io_acc_write_0_bits_mask_6 = out_input_4_input_0_payload_Some_0_mask[0*64 + 6];
    assign io_acc_write_0_bits_mask_7 = out_input_4_input_0_payload_Some_0_mask[0*64 + 7];
    assign io_acc_write_0_bits_mask_8 = out_input_4_input_0_payload_Some_0_mask[0*64 + 8];
    assign io_acc_write_0_bits_mask_9 = out_input_4_input_0_payload_Some_0_mask[0*64 + 9];
    assign io_acc_write_0_bits_mask_10 = out_input_4_input_0_payload_Some_0_mask[0*64 + 0];
    assign io_acc_write_0_bits_mask_11 = out_input_4_input_0_payload_Some_0_mask[0*64 + 1];
    assign io_acc_write_0_bits_mask_12 = out_input_4_input_0_payload_Some_0_mask[0*64 + 2];
    assign io_acc_write_0_bits_mask_13 = out_input_4_input_0_payload_Some_0_mask[0*64 + 3];
    assign io_acc_write_0_bits_mask_14 = out_input_4_input_0_payload_Some_0_mask[0*64 + 4];
    assign io_acc_write_0_bits_mask_15 = out_input_4_input_0_payload_Some_0_mask[0*64 + 5];
    assign io_acc_write_0_bits_mask_16 = out_input_4_input_0_payload_Some_0_mask[0*64 + 6];
    assign io_acc_write_0_bits_mask_17 = out_input_4_input_0_payload_Some_0_mask[0*64 + 7];
    assign io_acc_write_0_bits_mask_18 = out_input_4_input_0_payload_Some_0_mask[0*64 + 8];
    assign io_acc_write_0_bits_mask_19 = out_input_4_input_0_payload_Some_0_mask[0*64 + 9];
    assign io_acc_write_0_bits_mask_20 = out_input_4_input_0_payload_Some_0_mask[0*64 + 0];
    assign io_acc_write_0_bits_mask_21 = out_input_4_input_0_payload_Some_0_mask[0*64 + 1];
    assign io_acc_write_0_bits_mask_22 = out_input_4_input_0_payload_Some_0_mask[0*64 + 2];
    assign io_acc_write_0_bits_mask_23 = out_input_4_input_0_payload_Some_0_mask[0*64 + 3];
    assign io_acc_write_0_bits_mask_24 = out_input_4_input_0_payload_Some_0_mask[0*64 + 4];
    assign io_acc_write_0_bits_mask_25 = out_input_4_input_0_payload_Some_0_mask[0*64 + 5];
    assign io_acc_write_0_bits_mask_26 = out_input_4_input_0_payload_Some_0_mask[0*64 + 6];
    assign io_acc_write_0_bits_mask_27 = out_input_4_input_0_payload_Some_0_mask[0*64 + 7];
    assign io_acc_write_0_bits_mask_28 = out_input_4_input_0_payload_Some_0_mask[0*64 + 8];
    assign io_acc_write_0_bits_mask_29 = out_input_4_input_0_payload_Some_0_mask[0*64 + 9];
    assign io_acc_write_0_bits_mask_30 = out_input_4_input_0_payload_Some_0_mask[0*64 + 0];
    assign io_acc_write_0_bits_mask_31 = out_input_4_input_0_payload_Some_0_mask[0*64 + 1];
    assign io_acc_write_0_bits_mask_32 = out_input_4_input_0_payload_Some_0_mask[0*64 + 2];
    assign io_acc_write_0_bits_mask_33 = out_input_4_input_0_payload_Some_0_mask[0*64 + 3];
    assign io_acc_write_0_bits_mask_34 = out_input_4_input_0_payload_Some_0_mask[0*64 + 4];
    assign io_acc_write_0_bits_mask_35 = out_input_4_input_0_payload_Some_0_mask[0*64 + 5];
    assign io_acc_write_0_bits_mask_36 = out_input_4_input_0_payload_Some_0_mask[0*64 + 6];
    assign io_acc_write_0_bits_mask_37 = out_input_4_input_0_payload_Some_0_mask[0*64 + 7];
    assign io_acc_write_0_bits_mask_38 = out_input_4_input_0_payload_Some_0_mask[0*64 + 8];
    assign io_acc_write_0_bits_mask_39 = out_input_4_input_0_payload_Some_0_mask[0*64 + 9];
    assign io_acc_write_0_bits_mask_40 = out_input_4_input_0_payload_Some_0_mask[0*64 + 0];
    assign io_acc_write_0_bits_mask_41 = out_input_4_input_0_payload_Some_0_mask[0*64 + 1];
    assign io_acc_write_0_bits_mask_42 = out_input_4_input_0_payload_Some_0_mask[0*64 + 2];
    assign io_acc_write_0_bits_mask_43 = out_input_4_input_0_payload_Some_0_mask[0*64 + 3];
    assign io_acc_write_0_bits_mask_44 = out_input_4_input_0_payload_Some_0_mask[0*64 + 4];
    assign io_acc_write_0_bits_mask_45 = out_input_4_input_0_payload_Some_0_mask[0*64 + 5];
    assign io_acc_write_0_bits_mask_46 = out_input_4_input_0_payload_Some_0_mask[0*64 + 6];
    assign io_acc_write_0_bits_mask_47 = out_input_4_input_0_payload_Some_0_mask[0*64 + 7];
    assign io_acc_write_0_bits_mask_48 = out_input_4_input_0_payload_Some_0_mask[0*64 + 8];
    assign io_acc_write_0_bits_mask_49 = out_input_4_input_0_payload_Some_0_mask[0*64 + 9];
    assign io_acc_write_0_bits_mask_50 = out_input_4_input_0_payload_Some_0_mask[0*64 + 0];
    assign io_acc_write_0_bits_mask_51 = out_input_4_input_0_payload_Some_0_mask[0*64 + 1];
    assign io_acc_write_0_bits_mask_52 = out_input_4_input_0_payload_Some_0_mask[0*64 + 2];
    assign io_acc_write_0_bits_mask_53 = out_input_4_input_0_payload_Some_0_mask[0*64 + 3];
    assign io_acc_write_0_bits_mask_54 = out_input_4_input_0_payload_Some_0_mask[0*64 + 4];
    assign io_acc_write_0_bits_mask_55 = out_input_4_input_0_payload_Some_0_mask[0*64 + 5];
    assign io_acc_write_0_bits_mask_56 = out_input_4_input_0_payload_Some_0_mask[0*64 + 6];
    assign io_acc_write_0_bits_mask_57 = out_input_4_input_0_payload_Some_0_mask[0*64 + 7];
    assign io_acc_write_0_bits_mask_58 = out_input_4_input_0_payload_Some_0_mask[0*64 + 8];
    assign io_acc_write_0_bits_mask_59 = out_input_4_input_0_payload_Some_0_mask[0*64 + 9];
    assign io_acc_write_0_bits_mask_60 = out_input_4_input_0_payload_Some_0_mask[0*64 + 0];
    assign io_acc_write_0_bits_mask_61 = out_input_4_input_0_payload_Some_0_mask[0*64 + 1];
    assign io_acc_write_0_bits_mask_62 = out_input_4_input_0_payload_Some_0_mask[0*64 + 2];
    assign io_acc_write_0_bits_mask_63 = out_input_4_input_0_payload_Some_0_mask[0*64 + 3];
    assign io_acc_write_1_bits_mask_0 = out_input_4_input_0_payload_Some_0_mask[1*64 + 0];
    assign io_acc_write_1_bits_mask_1 = out_input_4_input_0_payload_Some_0_mask[1*64 + 1];
    assign io_acc_write_1_bits_mask_2 = out_input_4_input_0_payload_Some_0_mask[1*64 + 2];
    assign io_acc_write_1_bits_mask_3 = out_input_4_input_0_payload_Some_0_mask[1*64 + 3];
    assign io_acc_write_1_bits_mask_4 = out_input_4_input_0_payload_Some_0_mask[1*64 + 4];
    assign io_acc_write_1_bits_mask_5 = out_input_4_input_0_payload_Some_0_mask[1*64 + 5];
    assign io_acc_write_1_bits_mask_6 = out_input_4_input_0_payload_Some_0_mask[1*64 + 6];
    assign io_acc_write_1_bits_mask_7 = out_input_4_input_0_payload_Some_0_mask[1*64 + 7];
    assign io_acc_write_1_bits_mask_8 = out_input_4_input_0_payload_Some_0_mask[1*64 + 8];
    assign io_acc_write_1_bits_mask_9 = out_input_4_input_0_payload_Some_0_mask[1*64 + 9];
    assign io_acc_write_1_bits_mask_10 = out_input_4_input_0_payload_Some_0_mask[1*64 + 0];
    assign io_acc_write_1_bits_mask_11 = out_input_4_input_0_payload_Some_0_mask[1*64 + 1];
    assign io_acc_write_1_bits_mask_12 = out_input_4_input_0_payload_Some_0_mask[1*64 + 2];
    assign io_acc_write_1_bits_mask_13 = out_input_4_input_0_payload_Some_0_mask[1*64 + 3];
    assign io_acc_write_1_bits_mask_14 = out_input_4_input_0_payload_Some_0_mask[1*64 + 4];
    assign io_acc_write_1_bits_mask_15 = out_input_4_input_0_payload_Some_0_mask[1*64 + 5];
    assign io_acc_write_1_bits_mask_16 = out_input_4_input_0_payload_Some_0_mask[1*64 + 6];
    assign io_acc_write_1_bits_mask_17 = out_input_4_input_0_payload_Some_0_mask[1*64 + 7];
    assign io_acc_write_1_bits_mask_18 = out_input_4_input_0_payload_Some_0_mask[1*64 + 8];
    assign io_acc_write_1_bits_mask_19 = out_input_4_input_0_payload_Some_0_mask[1*64 + 9];
    assign io_acc_write_1_bits_mask_20 = out_input_4_input_0_payload_Some_0_mask[1*64 + 0];
    assign io_acc_write_1_bits_mask_21 = out_input_4_input_0_payload_Some_0_mask[1*64 + 1];
    assign io_acc_write_1_bits_mask_22 = out_input_4_input_0_payload_Some_0_mask[1*64 + 2];
    assign io_acc_write_1_bits_mask_23 = out_input_4_input_0_payload_Some_0_mask[1*64 + 3];
    assign io_acc_write_1_bits_mask_24 = out_input_4_input_0_payload_Some_0_mask[1*64 + 4];
    assign io_acc_write_1_bits_mask_25 = out_input_4_input_0_payload_Some_0_mask[1*64 + 5];
    assign io_acc_write_1_bits_mask_26 = out_input_4_input_0_payload_Some_0_mask[1*64 + 6];
    assign io_acc_write_1_bits_mask_27 = out_input_4_input_0_payload_Some_0_mask[1*64 + 7];
    assign io_acc_write_1_bits_mask_28 = out_input_4_input_0_payload_Some_0_mask[1*64 + 8];
    assign io_acc_write_1_bits_mask_29 = out_input_4_input_0_payload_Some_0_mask[1*64 + 9];
    assign io_acc_write_1_bits_mask_30 = out_input_4_input_0_payload_Some_0_mask[1*64 + 0];
    assign io_acc_write_1_bits_mask_31 = out_input_4_input_0_payload_Some_0_mask[1*64 + 1];
    assign io_acc_write_1_bits_mask_32 = out_input_4_input_0_payload_Some_0_mask[1*64 + 2];
    assign io_acc_write_1_bits_mask_33 = out_input_4_input_0_payload_Some_0_mask[1*64 + 3];
    assign io_acc_write_1_bits_mask_34 = out_input_4_input_0_payload_Some_0_mask[1*64 + 4];
    assign io_acc_write_1_bits_mask_35 = out_input_4_input_0_payload_Some_0_mask[1*64 + 5];
    assign io_acc_write_1_bits_mask_36 = out_input_4_input_0_payload_Some_0_mask[1*64 + 6];
    assign io_acc_write_1_bits_mask_37 = out_input_4_input_0_payload_Some_0_mask[1*64 + 7];
    assign io_acc_write_1_bits_mask_38 = out_input_4_input_0_payload_Some_0_mask[1*64 + 8];
    assign io_acc_write_1_bits_mask_39 = out_input_4_input_0_payload_Some_0_mask[1*64 + 9];
    assign io_acc_write_1_bits_mask_40 = out_input_4_input_0_payload_Some_0_mask[1*64 + 0];
    assign io_acc_write_1_bits_mask_41 = out_input_4_input_0_payload_Some_0_mask[1*64 + 1];
    assign io_acc_write_1_bits_mask_42 = out_input_4_input_0_payload_Some_0_mask[1*64 + 2];
    assign io_acc_write_1_bits_mask_43 = out_input_4_input_0_payload_Some_0_mask[1*64 + 3];
    assign io_acc_write_1_bits_mask_44 = out_input_4_input_0_payload_Some_0_mask[1*64 + 4];
    assign io_acc_write_1_bits_mask_45 = out_input_4_input_0_payload_Some_0_mask[1*64 + 5];
    assign io_acc_write_1_bits_mask_46 = out_input_4_input_0_payload_Some_0_mask[1*64 + 6];
    assign io_acc_write_1_bits_mask_47 = out_input_4_input_0_payload_Some_0_mask[1*64 + 7];
    assign io_acc_write_1_bits_mask_48 = out_input_4_input_0_payload_Some_0_mask[1*64 + 8];
    assign io_acc_write_1_bits_mask_49 = out_input_4_input_0_payload_Some_0_mask[1*64 + 9];
    assign io_acc_write_1_bits_mask_50 = out_input_4_input_0_payload_Some_0_mask[1*64 + 0];
    assign io_acc_write_1_bits_mask_51 = out_input_4_input_0_payload_Some_0_mask[1*64 + 1];
    assign io_acc_write_1_bits_mask_52 = out_input_4_input_0_payload_Some_0_mask[1*64 + 2];
    assign io_acc_write_1_bits_mask_53 = out_input_4_input_0_payload_Some_0_mask[1*64 + 3];
    assign io_acc_write_1_bits_mask_54 = out_input_4_input_0_payload_Some_0_mask[1*64 + 4];
    assign io_acc_write_1_bits_mask_55 = out_input_4_input_0_payload_Some_0_mask[1*64 + 5];
    assign io_acc_write_1_bits_mask_56 = out_input_4_input_0_payload_Some_0_mask[1*64 + 6];
    assign io_acc_write_1_bits_mask_57 = out_input_4_input_0_payload_Some_0_mask[1*64 + 7];
    assign io_acc_write_1_bits_mask_58 = out_input_4_input_0_payload_Some_0_mask[1*64 + 8];
    assign io_acc_write_1_bits_mask_59 = out_input_4_input_0_payload_Some_0_mask[1*64 + 9];
    assign io_acc_write_1_bits_mask_60 = out_input_4_input_0_payload_Some_0_mask[1*64 + 0];
    assign io_acc_write_1_bits_mask_61 = out_input_4_input_0_payload_Some_0_mask[1*64 + 1];
    assign io_acc_write_1_bits_mask_62 = out_input_4_input_0_payload_Some_0_mask[1*64 + 2];
    assign io_acc_write_1_bits_mask_63 = out_input_4_input_0_payload_Some_0_mask[1*64 + 3];
    
    wire  out_output_payload_discriminant;
    assign io_completed_valid = out_output_payload_discriminant;
    wire [6-1:0] out_output_payload_Some_0;
    assign io_completed_bits = out_output_payload_Some_0;

execute_default_top execute_hf
(
    .clk(clock),
    .rst(reset),
    .in_input_0_payload_discriminant(in_input_0_payload_discriminant),
    .in_input_0_payload_Some_0_cmd_inst_funct_discriminant(in_input_0_payload_Some_0_cmd_inst_funct_discriminant),
    .in_input_0_payload_Some_0_cmd_inst_rs2(in_input_0_payload_Some_0_cmd_inst_rs2),
    .in_input_0_payload_Some_0_cmd_inst_rs1(in_input_0_payload_Some_0_cmd_inst_rs1),
    .in_input_0_payload_Some_0_cmd_inst_xd(in_input_0_payload_Some_0_cmd_inst_xd),
    .in_input_0_payload_Some_0_cmd_inst_xs1(in_input_0_payload_Some_0_cmd_inst_xs1),
    .in_input_0_payload_Some_0_cmd_inst_xs2(in_input_0_payload_Some_0_cmd_inst_xs2),
    .in_input_0_payload_Some_0_cmd_inst_rd(in_input_0_payload_Some_0_cmd_inst_rd),
    .in_input_0_payload_Some_0_cmd_inst_opcode(in_input_0_payload_Some_0_cmd_inst_opcode),
    .in_input_0_payload_Some_0_cmd_rs1(in_input_0_payload_Some_0_cmd_rs1),
    .in_input_0_payload_Some_0_cmd_rs2(in_input_0_payload_Some_0_cmd_rs2),
    .in_input_0_payload_Some_0_cmd_status_debug(in_input_0_payload_Some_0_cmd_status_debug),
    .in_input_0_payload_Some_0_cmd_status_cease(in_input_0_payload_Some_0_cmd_status_cease),
    .in_input_0_payload_Some_0_cmd_status_wfi(in_input_0_payload_Some_0_cmd_status_wfi),
    .in_input_0_payload_Some_0_cmd_status_isa(in_input_0_payload_Some_0_cmd_status_isa),
    .in_input_0_payload_Some_0_cmd_status_dprv(in_input_0_payload_Some_0_cmd_status_dprv),
    .in_input_0_payload_Some_0_cmd_status_dv(in_input_0_payload_Some_0_cmd_status_dv),
    .in_input_0_payload_Some_0_cmd_status_prv(in_input_0_payload_Some_0_cmd_status_prv),
    .in_input_0_payload_Some_0_cmd_status_v(in_input_0_payload_Some_0_cmd_status_v),
    .in_input_0_payload_Some_0_cmd_status_sd(in_input_0_payload_Some_0_cmd_status_sd),
    .in_input_0_payload_Some_0_cmd_status_zero2(in_input_0_payload_Some_0_cmd_status_zero2),
    .in_input_0_payload_Some_0_cmd_status_mpv(in_input_0_payload_Some_0_cmd_status_mpv),
    .in_input_0_payload_Some_0_cmd_status_gva(in_input_0_payload_Some_0_cmd_status_gva),
    .in_input_0_payload_Some_0_cmd_status_mbe(in_input_0_payload_Some_0_cmd_status_mbe),
    .in_input_0_payload_Some_0_cmd_status_sbe(in_input_0_payload_Some_0_cmd_status_sbe),
    .in_input_0_payload_Some_0_cmd_status_sxl(in_input_0_payload_Some_0_cmd_status_sxl),
    .in_input_0_payload_Some_0_cmd_status_uxl(in_input_0_payload_Some_0_cmd_status_uxl),
    .in_input_0_payload_Some_0_cmd_status_sd_rv32(in_input_0_payload_Some_0_cmd_status_sd_rv32),
    .in_input_0_payload_Some_0_cmd_status_zero1(in_input_0_payload_Some_0_cmd_status_zero1),
    .in_input_0_payload_Some_0_cmd_status_tsr(in_input_0_payload_Some_0_cmd_status_tsr),
    .in_input_0_payload_Some_0_cmd_status_tw(in_input_0_payload_Some_0_cmd_status_tw),
    .in_input_0_payload_Some_0_cmd_status_tvm(in_input_0_payload_Some_0_cmd_status_tvm),
    .in_input_0_payload_Some_0_cmd_status_mxr(in_input_0_payload_Some_0_cmd_status_mxr),
    .in_input_0_payload_Some_0_cmd_status_sum(in_input_0_payload_Some_0_cmd_status_sum),
    .in_input_0_payload_Some_0_cmd_status_mprv(in_input_0_payload_Some_0_cmd_status_mprv),
    .in_input_0_payload_Some_0_cmd_status_xs(in_input_0_payload_Some_0_cmd_status_xs),
    .in_input_0_payload_Some_0_cmd_status_fs(in_input_0_payload_Some_0_cmd_status_fs),
    .in_input_0_payload_Some_0_cmd_status_mpp(in_input_0_payload_Some_0_cmd_status_mpp),
    .in_input_0_payload_Some_0_cmd_status_vs(in_input_0_payload_Some_0_cmd_status_vs),
    .in_input_0_payload_Some_0_cmd_status_spp(in_input_0_payload_Some_0_cmd_status_spp),
    .in_input_0_payload_Some_0_cmd_status_mpie(in_input_0_payload_Some_0_cmd_status_mpie),
    .in_input_0_payload_Some_0_cmd_status_ube(in_input_0_payload_Some_0_cmd_status_ube),
    .in_input_0_payload_Some_0_cmd_status_spie(in_input_0_payload_Some_0_cmd_status_spie),
    .in_input_0_payload_Some_0_cmd_status_upie(in_input_0_payload_Some_0_cmd_status_upie),
    .in_input_0_payload_Some_0_cmd_status_mie(in_input_0_payload_Some_0_cmd_status_mie),
    .in_input_0_payload_Some_0_cmd_status_hie(in_input_0_payload_Some_0_cmd_status_hie),
    .in_input_0_payload_Some_0_cmd_status_sie(in_input_0_payload_Some_0_cmd_status_sie),
    .in_input_0_payload_Some_0_cmd_status_uie(in_input_0_payload_Some_0_cmd_status_uie),
    .in_input_0_payload_Some_0_rob_id_discriminant(in_input_0_payload_Some_0_rob_id_discriminant),
    .in_input_0_payload_Some_0_rob_id_Some_0(in_input_0_payload_Some_0_rob_id_Some_0),
    .in_input_0_payload_Some_0_from_matmul_fsm(in_input_0_payload_Some_0_from_matmul_fsm),
    .in_input_0_payload_Some_0_from_conv_fsm(in_input_0_payload_Some_0_from_conv_fsm),
    .in_input_0_resolver_ready(in_input_0_resolver_ready),
    .in_input_1_output_payload_discriminant(in_input_1_output_payload_discriminant),
    .in_input_1_output_payload_Some_0_data(in_input_1_output_payload_Some_0_data),
    .in_input_1_output_payload_Some_0_from_dma(in_input_1_output_payload_Some_0_from_dma),
    .in_input_1_output_resolver_ready(in_input_1_output_resolver_ready),
    .in_input_3_output_payload_discriminant(in_input_3_output_payload_discriminant),
    .in_input_3_output_payload_Some_0_data(in_input_3_output_payload_Some_0_data),
    .in_input_3_output_payload_Some_0_from_dma(in_input_3_output_payload_Some_0_from_dma),
    .in_input_3_output_resolver_ready(in_input_3_output_resolver_ready),
    .out_input_1_input_0_payload_discriminant(out_input_1_input_0_payload_discriminant),
    .out_input_1_input_0_payload_Some_0_addr(out_input_1_input_0_payload_Some_0_addr),
    .out_input_1_input_0_payload_Some_0_from_dma(out_input_1_input_0_payload_Some_0_from_dma),
    .out_input_1_input_0_resolver_ready(out_input_1_input_0_resolver_ready),
    .out_input_2_input_0_payload_discriminant(out_input_2_input_0_payload_discriminant),
    .out_input_2_input_0_payload_Some_0_addr(out_input_2_input_0_payload_Some_0_addr),
    .out_input_2_input_0_payload_Some_0_data(out_input_2_input_0_payload_Some_0_data),
    .out_input_2_input_0_payload_Some_0_mask(out_input_2_input_0_payload_Some_0_mask),
    .out_input_3_input_0_payload_discriminant(out_input_3_input_0_payload_discriminant),
    .out_input_3_input_0_payload_Some_0_scale(out_input_3_input_0_payload_Some_0_scale),
    .out_input_3_input_0_payload_Some_0_full(out_input_3_input_0_payload_Some_0_full),
    .out_input_3_input_0_payload_Some_0_act(out_input_3_input_0_payload_Some_0_act),
    .out_input_3_input_0_payload_Some_0_from_dma(out_input_3_input_0_payload_Some_0_from_dma),
    .out_input_3_input_0_payload_Some_0_addr(out_input_3_input_0_payload_Some_0_addr),
    .out_input_3_input_0_resolver_ready(out_input_3_input_0_resolver_ready),
    .out_input_4_input_0_payload_discriminant(out_input_4_input_0_payload_discriminant),
    .out_input_4_input_0_payload_Some_0_addr(out_input_4_input_0_payload_Some_0_addr),
    .out_input_4_input_0_payload_Some_0_data(out_input_4_input_0_payload_Some_0_data),
    .out_input_4_input_0_payload_Some_0_acc(out_input_4_input_0_payload_Some_0_acc),
    .out_input_4_input_0_payload_Some_0_mask(out_input_4_input_0_payload_Some_0_mask),
    .out_output_payload_discriminant(out_output_payload_discriminant),
    .out_output_payload_Some_0(out_output_payload_Some_0)
);

endmodule