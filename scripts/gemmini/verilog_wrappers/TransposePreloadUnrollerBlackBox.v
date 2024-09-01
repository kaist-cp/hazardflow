module TransposePreloadUnrollerBlackBoxAdapter(
  input         clock,
                reset,
                io_in_valid,
  input  [6:0]  io_in_cmd_bits_cmd_inst_funct,
  input  [4:0]  io_in_cmd_bits_cmd_inst_rs2,
                io_in_cmd_bits_cmd_inst_rs1,
  input         io_in_cmd_bits_cmd_inst_xd,
                io_in_cmd_bits_cmd_inst_xs1,
                io_in_cmd_bits_cmd_inst_xs2,
  input  [4:0]  io_in_cmd_bits_cmd_inst_rd,
  input  [6:0]  io_in_cmd_bits_cmd_inst_opcode,
  input  [63:0] io_in_cmd_bits_cmd_rs1,
                io_in_cmd_bits_cmd_rs2,
  input         io_in_cmd_bits_cmd_status_debug,
                io_in_cmd_bits_cmd_status_cease,
                io_in_cmd_bits_cmd_status_wfi,
  input  [31:0] io_in_cmd_bits_cmd_status_isa,
  input  [1:0]  io_in_cmd_bits_cmd_status_dprv,
  input         io_in_cmd_bits_cmd_status_dv,
  input  [1:0]  io_in_cmd_bits_cmd_status_prv,
  input         io_in_cmd_bits_cmd_status_v,
                io_in_cmd_bits_cmd_status_sd,
  input  [22:0] io_in_cmd_bits_cmd_status_zero2,
  input         io_in_cmd_bits_cmd_status_mpv,
                io_in_cmd_bits_cmd_status_gva,
                io_in_cmd_bits_cmd_status_mbe,
                io_in_cmd_bits_cmd_status_sbe,
  input  [1:0]  io_in_cmd_bits_cmd_status_sxl,
                io_in_cmd_bits_cmd_status_uxl,
  input         io_in_cmd_bits_cmd_status_sd_rv32,
  input  [7:0]  io_in_cmd_bits_cmd_status_zero1,
  input         io_in_cmd_bits_cmd_status_tsr,
                io_in_cmd_bits_cmd_status_tw,
                io_in_cmd_bits_cmd_status_tvm,
                io_in_cmd_bits_cmd_status_mxr,
                io_in_cmd_bits_cmd_status_sum,
                io_in_cmd_bits_cmd_status_mprv,
  input  [1:0]  io_in_cmd_bits_cmd_status_xs,
                io_in_cmd_bits_cmd_status_fs,
                io_in_cmd_bits_cmd_status_mpp,
                io_in_cmd_bits_cmd_status_vs,
  input         io_in_cmd_bits_cmd_status_spp,
                io_in_cmd_bits_cmd_status_mpie,
                io_in_cmd_bits_cmd_status_ube,
                io_in_cmd_bits_cmd_status_spie,
                io_in_cmd_bits_cmd_status_upie,
                io_in_cmd_bits_cmd_status_mie,
                io_in_cmd_bits_cmd_status_hie,
                io_in_cmd_bits_cmd_status_sie,
                io_in_cmd_bits_cmd_status_uie,
                io_in_cmd_bits_rob_id_valid,
  input  [5:0]  io_in_cmd_bits_rob_id_bits,
  input         io_in_cmd_bits_from_matmul_fsm,
                io_in_cmd_bits_from_conv_fsm,
                io_out_ready,
  output        io_in_ready,
                io_out_valid,
  output [6:0]  io_out_cmd_bits_cmd_inst_funct,
  output [4:0]  io_out_cmd_bits_cmd_inst_rs2,
                io_out_cmd_bits_cmd_inst_rs1,
  output        io_out_cmd_bits_cmd_inst_xd,
                io_out_cmd_bits_cmd_inst_xs1,
                io_out_cmd_bits_cmd_inst_xs2,
  output [4:0]  io_out_cmd_bits_cmd_inst_rd,
  output [6:0]  io_out_cmd_bits_cmd_inst_opcode,
  output [63:0] io_out_cmd_bits_cmd_rs1,
                io_out_cmd_bits_cmd_rs2,
  output        io_out_cmd_bits_cmd_status_debug,
                io_out_cmd_bits_cmd_status_cease,
                io_out_cmd_bits_cmd_status_wfi,
  output [31:0] io_out_cmd_bits_cmd_status_isa,
  output [1:0]  io_out_cmd_bits_cmd_status_dprv,
  output        io_out_cmd_bits_cmd_status_dv,
  output [1:0]  io_out_cmd_bits_cmd_status_prv,
  output        io_out_cmd_bits_cmd_status_v,
                io_out_cmd_bits_cmd_status_sd,
  output [22:0] io_out_cmd_bits_cmd_status_zero2,
  output        io_out_cmd_bits_cmd_status_mpv,
                io_out_cmd_bits_cmd_status_gva,
                io_out_cmd_bits_cmd_status_mbe,
                io_out_cmd_bits_cmd_status_sbe,
  output [1:0]  io_out_cmd_bits_cmd_status_sxl,
                io_out_cmd_bits_cmd_status_uxl,
  output        io_out_cmd_bits_cmd_status_sd_rv32,
  output [7:0]  io_out_cmd_bits_cmd_status_zero1,
  output        io_out_cmd_bits_cmd_status_tsr,
                io_out_cmd_bits_cmd_status_tw,
                io_out_cmd_bits_cmd_status_tvm,
                io_out_cmd_bits_cmd_status_mxr,
                io_out_cmd_bits_cmd_status_sum,
                io_out_cmd_bits_cmd_status_mprv,
  output [1:0]  io_out_cmd_bits_cmd_status_xs,
                io_out_cmd_bits_cmd_status_fs,
                io_out_cmd_bits_cmd_status_mpp,
                io_out_cmd_bits_cmd_status_vs,
  output        io_out_cmd_bits_cmd_status_spp,
                io_out_cmd_bits_cmd_status_mpie,
                io_out_cmd_bits_cmd_status_ube,
                io_out_cmd_bits_cmd_status_spie,
                io_out_cmd_bits_cmd_status_upie,
                io_out_cmd_bits_cmd_status_mie,
                io_out_cmd_bits_cmd_status_hie,
                io_out_cmd_bits_cmd_status_sie,
                io_out_cmd_bits_cmd_status_uie,
                io_out_cmd_bits_rob_id_valid,
  output [5:0]  io_out_cmd_bits_rob_id_bits,
  output        io_out_cmd_bits_from_matmul_fsm,
                io_out_cmd_bits_from_conv_fsm
);

  // Input to TransposePreloadUnroller module
  wire in_input_0_payload_discriminant = io_in_valid;
  wire [5-1:0] in_input_0_payload_Some_0_cmd_inst_funct_discriminant = io_in_cmd_bits_cmd_inst_funct;
  wire [5-1:0] in_input_0_payload_Some_0_cmd_inst_rs2 = io_in_cmd_bits_cmd_inst_rs2;
  wire [5-1:0] in_input_0_payload_Some_0_cmd_inst_rs1 = io_in_cmd_bits_cmd_inst_rs1;
  wire in_input_0_payload_Some_0_cmd_inst_xd = io_in_cmd_bits_cmd_inst_xd;
  wire in_input_0_payload_Some_0_cmd_inst_xs1 = io_in_cmd_bits_cmd_inst_xs1;
  wire in_input_0_payload_Some_0_cmd_inst_xs2 = io_in_cmd_bits_cmd_inst_xs2;
  wire [5-1:0] in_input_0_payload_Some_0_cmd_inst_rd = io_in_cmd_bits_cmd_inst_rd;
  wire [7-1:0] in_input_0_payload_Some_0_cmd_inst_opcode = io_in_cmd_bits_cmd_inst_opcode;
  wire [64-1:0] in_input_0_payload_Some_0_cmd_rs1 = io_in_cmd_bits_cmd_rs1;
  wire [64-1:0] in_input_0_payload_Some_0_cmd_rs2 = io_in_cmd_bits_cmd_rs2;
  wire in_input_0_payload_Some_0_cmd_status_debug = io_in_cmd_bits_cmd_status_debug;
  wire in_input_0_payload_Some_0_cmd_status_cease = io_in_cmd_bits_cmd_status_cease;
  wire in_input_0_payload_Some_0_cmd_status_wfi = io_in_cmd_bits_cmd_status_wfi;
  wire [32-1:0] in_input_0_payload_Some_0_cmd_status_isa = io_in_cmd_bits_cmd_status_isa;
  wire [2-1:0] in_input_0_payload_Some_0_cmd_status_dprv = io_in_cmd_bits_cmd_status_dprv;
  wire in_input_0_payload_Some_0_cmd_status_dv = io_in_cmd_bits_cmd_status_dv;
  wire [2-1:0] in_input_0_payload_Some_0_cmd_status_prv = io_in_cmd_bits_cmd_status_prv;
  wire in_input_0_payload_Some_0_cmd_status_v = io_in_cmd_bits_cmd_status_v;
  wire in_input_0_payload_Some_0_cmd_status_sd = io_in_cmd_bits_cmd_status_sd;
  wire [23-1:0] in_input_0_payload_Some_0_cmd_status_zero2 = io_in_cmd_bits_cmd_status_zero2;
  wire in_input_0_payload_Some_0_cmd_status_mpv = io_in_cmd_bits_cmd_status_mpv;
  wire in_input_0_payload_Some_0_cmd_status_gva = io_in_cmd_bits_cmd_status_gva;
  wire in_input_0_payload_Some_0_cmd_status_mbe = io_in_cmd_bits_cmd_status_mbe;
  wire in_input_0_payload_Some_0_cmd_status_sbe = io_in_cmd_bits_cmd_status_sbe;
  wire [2-1:0] in_input_0_payload_Some_0_cmd_status_sxl = io_in_cmd_bits_cmd_status_sxl;
  wire [2-1:0] in_input_0_payload_Some_0_cmd_status_uxl = io_in_cmd_bits_cmd_status_uxl;
  wire in_input_0_payload_Some_0_cmd_status_sd_rv32 = io_in_cmd_bits_cmd_status_sd_rv32;
  wire [8-1:0] in_input_0_payload_Some_0_cmd_status_zero1 = io_in_cmd_bits_cmd_status_zero1;
  wire in_input_0_payload_Some_0_cmd_status_tsr = io_in_cmd_bits_cmd_status_tsr;
  wire in_input_0_payload_Some_0_cmd_status_tw = io_in_cmd_bits_cmd_status_tw;
  wire in_input_0_payload_Some_0_cmd_status_tvm = io_in_cmd_bits_cmd_status_tvm;
  wire in_input_0_payload_Some_0_cmd_status_mxr = io_in_cmd_bits_cmd_status_mxr;
  wire in_input_0_payload_Some_0_cmd_status_sum = io_in_cmd_bits_cmd_status_sum;
  wire in_input_0_payload_Some_0_cmd_status_mprv = io_in_cmd_bits_cmd_status_mprv;
  wire [2-1:0] in_input_0_payload_Some_0_cmd_status_xs = io_in_cmd_bits_cmd_status_xs;
  wire [2-1:0] in_input_0_payload_Some_0_cmd_status_fs = io_in_cmd_bits_cmd_status_fs;
  wire [2-1:0] in_input_0_payload_Some_0_cmd_status_mpp = io_in_cmd_bits_cmd_status_mpp;
  wire [2-1:0] in_input_0_payload_Some_0_cmd_status_vs = io_in_cmd_bits_cmd_status_vs;
  wire in_input_0_payload_Some_0_cmd_status_spp = io_in_cmd_bits_cmd_status_spp;
  wire in_input_0_payload_Some_0_cmd_status_mpie = io_in_cmd_bits_cmd_status_mpie;
  wire in_input_0_payload_Some_0_cmd_status_ube = io_in_cmd_bits_cmd_status_ube;
  wire in_input_0_payload_Some_0_cmd_status_spie = io_in_cmd_bits_cmd_status_spie;
  wire in_input_0_payload_Some_0_cmd_status_upie = io_in_cmd_bits_cmd_status_upie;
  wire in_input_0_payload_Some_0_cmd_status_mie = io_in_cmd_bits_cmd_status_mie;
  wire in_input_0_payload_Some_0_cmd_status_hie = io_in_cmd_bits_cmd_status_hie;
  wire in_input_0_payload_Some_0_cmd_status_sie = io_in_cmd_bits_cmd_status_sie;
  wire in_input_0_payload_Some_0_cmd_status_uie = io_in_cmd_bits_cmd_status_uie;
  wire in_input_0_payload_Some_0_rob_id_discriminant = io_in_cmd_bits_rob_id_valid;
  wire [6-1:0] in_input_0_payload_Some_0_rob_id_Some_0 = io_in_cmd_bits_rob_id_bits;
  wire in_input_0_payload_Some_0_from_matmul_fsm = io_in_cmd_bits_from_matmul_fsm;
  wire in_input_0_payload_Some_0_from_conv_fsm = io_in_cmd_bits_from_conv_fsm;
  wire out_output_resolver_ready = io_out_ready;

  // Output to TransposePreloadUnroller module
  wire in_input_0_resolver_ready;
  wire out_output_payload_discriminant;
  wire [5-1:0] out_output_payload_Some_0_cmd_inst_funct_discriminant;
  wire [5-1:0] out_output_payload_Some_0_cmd_inst_rs2;
  wire [5-1:0] out_output_payload_Some_0_cmd_inst_rs1;
  wire out_output_payload_Some_0_cmd_inst_xd;
  wire out_output_payload_Some_0_cmd_inst_xs1;
  wire out_output_payload_Some_0_cmd_inst_xs2;
  wire [5-1:0] out_output_payload_Some_0_cmd_inst_rd;
  wire [7-1:0] out_output_payload_Some_0_cmd_inst_opcode;
  wire [64-1:0] out_output_payload_Some_0_cmd_rs1;
  wire [64-1:0] out_output_payload_Some_0_cmd_rs2;
  wire out_output_payload_Some_0_cmd_status_debug;
  wire out_output_payload_Some_0_cmd_status_cease;
  wire out_output_payload_Some_0_cmd_status_wfi;
  wire [32-1:0] out_output_payload_Some_0_cmd_status_isa;
  wire [2-1:0] out_output_payload_Some_0_cmd_status_dprv;
  wire out_output_payload_Some_0_cmd_status_dv;
  wire [2-1:0] out_output_payload_Some_0_cmd_status_prv;
  wire out_output_payload_Some_0_cmd_status_v;
  wire out_output_payload_Some_0_cmd_status_sd;
  wire [23-1:0] out_output_payload_Some_0_cmd_status_zero2;
  wire out_output_payload_Some_0_cmd_status_mpv;
  wire out_output_payload_Some_0_cmd_status_gva;
  wire out_output_payload_Some_0_cmd_status_mbe;
  wire out_output_payload_Some_0_cmd_status_sbe;
  wire [2-1:0] out_output_payload_Some_0_cmd_status_sxl;
  wire [2-1:0] out_output_payload_Some_0_cmd_status_uxl;
  wire out_output_payload_Some_0_cmd_status_sd_rv32;
  wire [8-1:0] out_output_payload_Some_0_cmd_status_zero1;
  wire out_output_payload_Some_0_cmd_status_tsr;
  wire out_output_payload_Some_0_cmd_status_tw;
  wire out_output_payload_Some_0_cmd_status_tvm;
  wire out_output_payload_Some_0_cmd_status_mxr;
  wire out_output_payload_Some_0_cmd_status_sum;
  wire out_output_payload_Some_0_cmd_status_mprv;
  wire [2-1:0] out_output_payload_Some_0_cmd_status_xs;
  wire [2-1:0] out_output_payload_Some_0_cmd_status_fs;
  wire [2-1:0] out_output_payload_Some_0_cmd_status_mpp;
  wire [2-1:0] out_output_payload_Some_0_cmd_status_vs;
  wire out_output_payload_Some_0_cmd_status_spp;
  wire out_output_payload_Some_0_cmd_status_mpie;
  wire out_output_payload_Some_0_cmd_status_ube;
  wire out_output_payload_Some_0_cmd_status_spie;
  wire out_output_payload_Some_0_cmd_status_upie;
  wire out_output_payload_Some_0_cmd_status_mie;
  wire out_output_payload_Some_0_cmd_status_hie;
  wire out_output_payload_Some_0_cmd_status_sie;
  wire out_output_payload_Some_0_cmd_status_uie;
  wire out_output_payload_Some_0_rob_id_discriminant;
  wire [6-1:0] out_output_payload_Some_0_rob_id_Some_0;
  wire out_output_payload_Some_0_from_matmul_fsm;
  wire out_output_payload_Some_0_from_conv_fsm;

transpose_preload_unroller_top transpose_preload_unroller_inst(
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
    .out_output_resolver_ready(out_output_resolver_ready),

    .in_input_0_resolver_ready(in_input_0_resolver_ready),
    .out_output_payload_discriminant(out_output_payload_discriminant),
    .out_output_payload_Some_0_cmd_inst_funct_discriminant(out_output_payload_Some_0_cmd_inst_funct_discriminant),
    .out_output_payload_Some_0_cmd_inst_rs2(out_output_payload_Some_0_cmd_inst_rs2),
    .out_output_payload_Some_0_cmd_inst_rs1(out_output_payload_Some_0_cmd_inst_rs1),
    .out_output_payload_Some_0_cmd_inst_xd(out_output_payload_Some_0_cmd_inst_xd),
    .out_output_payload_Some_0_cmd_inst_xs1(out_output_payload_Some_0_cmd_inst_xs1),
    .out_output_payload_Some_0_cmd_inst_xs2(out_output_payload_Some_0_cmd_inst_xs2),
    .out_output_payload_Some_0_cmd_inst_rd(out_output_payload_Some_0_cmd_inst_rd),
    .out_output_payload_Some_0_cmd_inst_opcode(out_output_payload_Some_0_cmd_inst_opcode),
    .out_output_payload_Some_0_cmd_rs1(out_output_payload_Some_0_cmd_rs1),
    .out_output_payload_Some_0_cmd_rs2(out_output_payload_Some_0_cmd_rs2),
    .out_output_payload_Some_0_cmd_status_debug(out_output_payload_Some_0_cmd_status_debug),
    .out_output_payload_Some_0_cmd_status_cease(out_output_payload_Some_0_cmd_status_cease),
    .out_output_payload_Some_0_cmd_status_wfi(out_output_payload_Some_0_cmd_status_wfi),
    .out_output_payload_Some_0_cmd_status_isa(out_output_payload_Some_0_cmd_status_isa),
    .out_output_payload_Some_0_cmd_status_dprv(out_output_payload_Some_0_cmd_status_dprv),
    .out_output_payload_Some_0_cmd_status_dv(out_output_payload_Some_0_cmd_status_dv),
    .out_output_payload_Some_0_cmd_status_prv(out_output_payload_Some_0_cmd_status_prv),
    .out_output_payload_Some_0_cmd_status_v(out_output_payload_Some_0_cmd_status_v),
    .out_output_payload_Some_0_cmd_status_sd(out_output_payload_Some_0_cmd_status_sd),
    .out_output_payload_Some_0_cmd_status_zero2(out_output_payload_Some_0_cmd_status_zero2),
    .out_output_payload_Some_0_cmd_status_mpv(out_output_payload_Some_0_cmd_status_mpv),
    .out_output_payload_Some_0_cmd_status_gva(out_output_payload_Some_0_cmd_status_gva),
    .out_output_payload_Some_0_cmd_status_mbe(out_output_payload_Some_0_cmd_status_mbe),
    .out_output_payload_Some_0_cmd_status_sbe(out_output_payload_Some_0_cmd_status_sbe),
    .out_output_payload_Some_0_cmd_status_sxl(out_output_payload_Some_0_cmd_status_sxl),
    .out_output_payload_Some_0_cmd_status_uxl(out_output_payload_Some_0_cmd_status_uxl),
    .out_output_payload_Some_0_cmd_status_sd_rv32(out_output_payload_Some_0_cmd_status_sd_rv32),
    .out_output_payload_Some_0_cmd_status_zero1(out_output_payload_Some_0_cmd_status_zero1),
    .out_output_payload_Some_0_cmd_status_tsr(out_output_payload_Some_0_cmd_status_tsr),
    .out_output_payload_Some_0_cmd_status_tw(out_output_payload_Some_0_cmd_status_tw),
    .out_output_payload_Some_0_cmd_status_tvm(out_output_payload_Some_0_cmd_status_tvm),
    .out_output_payload_Some_0_cmd_status_mxr(out_output_payload_Some_0_cmd_status_mxr),
    .out_output_payload_Some_0_cmd_status_sum(out_output_payload_Some_0_cmd_status_sum),
    .out_output_payload_Some_0_cmd_status_mprv(out_output_payload_Some_0_cmd_status_mprv),
    .out_output_payload_Some_0_cmd_status_xs(out_output_payload_Some_0_cmd_status_xs),
    .out_output_payload_Some_0_cmd_status_fs(out_output_payload_Some_0_cmd_status_fs),
    .out_output_payload_Some_0_cmd_status_mpp(out_output_payload_Some_0_cmd_status_mpp),
    .out_output_payload_Some_0_cmd_status_vs(out_output_payload_Some_0_cmd_status_vs),
    .out_output_payload_Some_0_cmd_status_spp(out_output_payload_Some_0_cmd_status_spp),
    .out_output_payload_Some_0_cmd_status_mpie(out_output_payload_Some_0_cmd_status_mpie),
    .out_output_payload_Some_0_cmd_status_ube(out_output_payload_Some_0_cmd_status_ube),
    .out_output_payload_Some_0_cmd_status_spie(out_output_payload_Some_0_cmd_status_spie),
    .out_output_payload_Some_0_cmd_status_upie(out_output_payload_Some_0_cmd_status_upie),
    .out_output_payload_Some_0_cmd_status_mie(out_output_payload_Some_0_cmd_status_mie),
    .out_output_payload_Some_0_cmd_status_hie(out_output_payload_Some_0_cmd_status_hie),
    .out_output_payload_Some_0_cmd_status_sie(out_output_payload_Some_0_cmd_status_sie),
    .out_output_payload_Some_0_cmd_status_uie(out_output_payload_Some_0_cmd_status_uie),
    .out_output_payload_Some_0_rob_id_discriminant(out_output_payload_Some_0_rob_id_discriminant),
    .out_output_payload_Some_0_rob_id_Some_0(out_output_payload_Some_0_rob_id_Some_0),
    .out_output_payload_Some_0_from_matmul_fsm(out_output_payload_Some_0_from_matmul_fsm),
    .out_output_payload_Some_0_from_conv_fsm(out_output_payload_Some_0_from_conv_fsm)
);

    assign io_in_ready = in_input_0_resolver_ready;
    assign io_out_valid = out_output_payload_discriminant;
    assign io_out_cmd_bits_cmd_inst_funct = out_output_payload_Some_0_cmd_inst_funct_discriminant;
    assign io_out_cmd_bits_cmd_inst_rs2 = out_output_payload_Some_0_cmd_inst_rs2;
    assign io_out_cmd_bits_cmd_inst_rs1 = out_output_payload_Some_0_cmd_inst_rs1;
    assign io_out_cmd_bits_cmd_inst_xd = out_output_payload_Some_0_cmd_inst_xd;
    assign io_out_cmd_bits_cmd_inst_xs1 = out_output_payload_Some_0_cmd_inst_xs1;
    assign io_out_cmd_bits_cmd_inst_xs2 = out_output_payload_Some_0_cmd_inst_xs2;
    assign io_out_cmd_bits_cmd_inst_rd = out_output_payload_Some_0_cmd_inst_rd;
    assign io_out_cmd_bits_cmd_inst_opcode = out_output_payload_Some_0_cmd_inst_opcode;
    assign io_out_cmd_bits_cmd_rs1 = out_output_payload_Some_0_cmd_rs1;
    assign io_out_cmd_bits_cmd_rs2 = out_output_payload_Some_0_cmd_rs2;
    assign io_out_cmd_bits_cmd_status_debug = out_output_payload_Some_0_cmd_status_debug;
    assign io_out_cmd_bits_cmd_status_cease = out_output_payload_Some_0_cmd_status_cease;
    assign io_out_cmd_bits_cmd_status_wfi = out_output_payload_Some_0_cmd_status_wfi;
    assign io_out_cmd_bits_cmd_status_isa = out_output_payload_Some_0_cmd_status_isa;
    assign io_out_cmd_bits_cmd_status_dprv = out_output_payload_Some_0_cmd_status_dprv;
    assign io_out_cmd_bits_cmd_status_dv = out_output_payload_Some_0_cmd_status_dv;
    assign io_out_cmd_bits_cmd_status_prv = out_output_payload_Some_0_cmd_status_prv;
    assign io_out_cmd_bits_cmd_status_v = out_output_payload_Some_0_cmd_status_v;
    assign io_out_cmd_bits_cmd_status_sd = out_output_payload_Some_0_cmd_status_sd;
    assign io_out_cmd_bits_cmd_status_zero2 = out_output_payload_Some_0_cmd_status_zero2;
    assign io_out_cmd_bits_cmd_status_mpv = out_output_payload_Some_0_cmd_status_mpv;
    assign io_out_cmd_bits_cmd_status_gva = out_output_payload_Some_0_cmd_status_gva;
    assign io_out_cmd_bits_cmd_status_mbe = out_output_payload_Some_0_cmd_status_mbe;
    assign io_out_cmd_bits_cmd_status_sbe = out_output_payload_Some_0_cmd_status_sbe;
    assign io_out_cmd_bits_cmd_status_sxl = out_output_payload_Some_0_cmd_status_sxl;
    assign io_out_cmd_bits_cmd_status_uxl = out_output_payload_Some_0_cmd_status_uxl;
    assign io_out_cmd_bits_cmd_status_sd_rv32 = out_output_payload_Some_0_cmd_status_sd_rv32;
    assign io_out_cmd_bits_cmd_status_zero1 = out_output_payload_Some_0_cmd_status_zero1;
    assign io_out_cmd_bits_cmd_status_tsr = out_output_payload_Some_0_cmd_status_tsr;
    assign io_out_cmd_bits_cmd_status_tw = out_output_payload_Some_0_cmd_status_tw;
    assign io_out_cmd_bits_cmd_status_tvm = out_output_payload_Some_0_cmd_status_tvm;
    assign io_out_cmd_bits_cmd_status_mxr = out_output_payload_Some_0_cmd_status_mxr;
    assign io_out_cmd_bits_cmd_status_sum = out_output_payload_Some_0_cmd_status_sum;
    assign io_out_cmd_bits_cmd_status_mprv = out_output_payload_Some_0_cmd_status_mprv;
    assign io_out_cmd_bits_cmd_status_xs = out_output_payload_Some_0_cmd_status_xs;
    assign io_out_cmd_bits_cmd_status_fs = out_output_payload_Some_0_cmd_status_fs;
    assign io_out_cmd_bits_cmd_status_mpp = out_output_payload_Some_0_cmd_status_mpp;
    assign io_out_cmd_bits_cmd_status_vs = out_output_payload_Some_0_cmd_status_vs;
    assign io_out_cmd_bits_cmd_status_spp = out_output_payload_Some_0_cmd_status_spp;
    assign io_out_cmd_bits_cmd_status_mpie = out_output_payload_Some_0_cmd_status_mpie;
    assign io_out_cmd_bits_cmd_status_ube = out_output_payload_Some_0_cmd_status_ube;
    assign io_out_cmd_bits_cmd_status_spie = out_output_payload_Some_0_cmd_status_spie;
    assign io_out_cmd_bits_cmd_status_upie = out_output_payload_Some_0_cmd_status_upie;
    assign io_out_cmd_bits_cmd_status_mie = out_output_payload_Some_0_cmd_status_mie;
    assign io_out_cmd_bits_cmd_status_hie = out_output_payload_Some_0_cmd_status_hie;
    assign io_out_cmd_bits_cmd_status_sie = out_output_payload_Some_0_cmd_status_sie;
    assign io_out_cmd_bits_cmd_status_uie = out_output_payload_Some_0_cmd_status_uie;
    assign io_out_cmd_bits_rob_id_valid = out_output_payload_Some_0_rob_id_discriminant;
    assign io_out_cmd_bits_rob_id_bits = out_output_payload_Some_0_rob_id_Some_0;
    assign io_out_cmd_bits_from_matmul_fsm = out_output_payload_Some_0_from_matmul_fsm;
    assign io_out_cmd_bits_from_conv_fsm = out_output_payload_Some_0_from_conv_fsm;

endmodule