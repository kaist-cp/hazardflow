module LoadControllerBlackBoxAdapter #(parameter LOG_2_UP_RESERVATION_STATION_ENTRIES = 6, MVIN_SCALE_T_BITS = 32) (
    input clock,
    input reset,

    output io_cmd_ready,
    input io_cmd_valid,
    input [7-1:0] io_cmd_bits_cmd_inst_funct,
    input [5-1:0] io_cmd_bits_cmd_inst_rs2,
    input [5-1:0] io_cmd_bits_cmd_inst_rs1,
    input io_cmd_bits_cmd_inst_xd,
    input io_cmd_bits_cmd_inst_xs1,
    input io_cmd_bits_cmd_inst_xs2,
    input [5-1:0] io_cmd_bits_cmd_inst_rd,
    input [7-1:0] io_cmd_bits_cmd_inst_opcode,
    input [64-1:0] io_cmd_bits_cmd_rs1,
    input [64-1:0] io_cmd_bits_cmd_rs2,
    input io_cmd_bits_cmd_status_debug,
    input io_cmd_bits_cmd_status_cease,
    input io_cmd_bits_cmd_status_wfi,
    input [32-1:0] io_cmd_bits_cmd_status_isa,
    input [2-1:0] io_cmd_bits_cmd_status_dprv,
    input io_cmd_bits_cmd_status_dv,
    input [2-1:0] io_cmd_bits_cmd_status_prv,
    input io_cmd_bits_cmd_status_v,
    input io_cmd_bits_cmd_status_sd,
    input [23-1:0] io_cmd_bits_cmd_status_zero2,
    input io_cmd_bits_cmd_status_mpv,
    input io_cmd_bits_cmd_status_gva,
    input io_cmd_bits_cmd_status_mbe,
    input io_cmd_bits_cmd_status_sbe,
    input [2-1:0] io_cmd_bits_cmd_status_sxl,
    input [2-1:0] io_cmd_bits_cmd_status_uxl,
    input io_cmd_bits_cmd_status_sd_rv32,
    input [8-1:0] io_cmd_bits_cmd_status_zero1,
    input io_cmd_bits_cmd_status_tsr,
    input io_cmd_bits_cmd_status_tw,
    input io_cmd_bits_cmd_status_tvm,
    input io_cmd_bits_cmd_status_mxr,
    input io_cmd_bits_cmd_status_sum,
    input io_cmd_bits_cmd_status_mprv,
    input [2-1:0] io_cmd_bits_cmd_status_xs,
    input [2-1:0] io_cmd_bits_cmd_status_fs,
    input [2-1:0] io_cmd_bits_cmd_status_mpp,
    input [2-1:0] io_cmd_bits_cmd_status_vs,
    input io_cmd_bits_cmd_status_spp,
    input io_cmd_bits_cmd_status_mpie,
    input io_cmd_bits_cmd_status_ube,
    input io_cmd_bits_cmd_status_spie,
    input io_cmd_bits_cmd_status_upie,
    input io_cmd_bits_cmd_status_mie,
    input io_cmd_bits_cmd_status_hie,
    input io_cmd_bits_cmd_status_sie,
    input io_cmd_bits_cmd_status_uie,
    input io_cmd_bits_rob_id_valid,
    input [LOG_2_UP_RESERVATION_STATION_ENTRIES-1:0] io_cmd_bits_rob_id_bits,
    input io_cmd_bits_from_matmul_fsm,
    input io_cmd_bits_from_conv_fsm,

    input io_dma_req_ready,
    output io_dma_req_valid,
    output [40-1:0] io_dma_req_bits_vaddr,
    output io_dma_req_bits_laddr_is_acc_addr,
    output io_dma_req_bits_laddr_accumulate,
    output io_dma_req_bits_laddr_read_full_acc_row,
    output io_dma_req_bits_laddr_norm_cmd,
    output [11-1:0] io_dma_req_bits_laddr_garbage,
    output io_dma_req_bits_laddr_garbage_bit,
    output [14-1:0] io_dma_req_bits_laddr_data,
    output [16-1:0] io_dma_req_bits_cols,
    output [16-1:0] io_dma_req_bits_repeats,
    output [MVIN_SCALE_T_BITS-1:0] io_dma_req_bits_scale,
    output io_dma_req_bits_has_acc_bitwidth,
    output io_dma_req_bits_all_zeros,
    output [16-1:0] io_dma_req_bits_block_stride,
    output [8-1:0] io_dma_req_bits_pixel_repeats,
    output [8-1:0] io_dma_req_bits_cmd_id,
    output io_dma_req_bits_status_debug,
    output io_dma_req_bits_status_cease,
    output io_dma_req_bits_status_wfi,
    output [32-1:0] io_dma_req_bits_status_isa,
    output [2-1:0] io_dma_req_bits_status_dprv,
    output io_dma_req_bits_status_dv,
    output [2-1:0] io_dma_req_bits_status_prv,
    output io_dma_req_bits_status_v,
    output io_dma_req_bits_status_sd,
    output [23-1:0] io_dma_req_bits_status_zero2,
    output io_dma_req_bits_status_mpv,
    output io_dma_req_bits_status_gva,
    output io_dma_req_bits_status_mbe,
    output io_dma_req_bits_status_sbe,
    output [2-1:0] io_dma_req_bits_status_sxl,
    output [2-1:0] io_dma_req_bits_status_uxl,
    output io_dma_req_bits_status_sd_rv32,
    output [8-1:0] io_dma_req_bits_status_zero1,
    output io_dma_req_bits_status_tsr,
    output io_dma_req_bits_status_tw,
    output io_dma_req_bits_status_tvm,
    output io_dma_req_bits_status_mxr,
    output io_dma_req_bits_status_sum,
    output io_dma_req_bits_status_mprv,
    output [2-1:0] io_dma_req_bits_status_xs,
    output [2-1:0] io_dma_req_bits_status_fs,
    output [2-1:0] io_dma_req_bits_status_mpp,
    output [2-1:0] io_dma_req_bits_status_vs,
    output io_dma_req_bits_status_spp,
    output io_dma_req_bits_status_mpie,
    output io_dma_req_bits_status_ube,
    output io_dma_req_bits_status_spie,
    output io_dma_req_bits_status_upie,
    output io_dma_req_bits_status_mie,
    output io_dma_req_bits_status_hie,
    output io_dma_req_bits_status_sie,
    output io_dma_req_bits_status_uie,
    input io_dma_resp_valid,
    input [16-1:0] io_dma_resp_bits_bytesRead,
    input [8-1:0] io_dma_resp_bits_cmd_id,

    input io_completed_ready,
    output io_completed_valid,
    output [LOG_2_UP_RESERVATION_STATION_ENTRIES-1:0] io_completed_bits
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

    wire in_input_1_output_payload_discriminant = io_dma_resp_valid;
    wire [16-1:0] in_input_1_output_payload_Some_0_bytes_read = io_dma_resp_bits_bytesRead;
    wire [8-1:0] in_input_1_output_payload_Some_0_cmd_id = io_dma_resp_bits_cmd_id;
    wire out_input_1_input_0_payload_discriminant;
    wire [40-1:0] out_input_1_input_0_payload_Some_0_vaddr;
    wire out_input_1_input_0_payload_Some_0_laddr_is_acc_addr;
    wire out_input_1_input_0_payload_Some_0_laddr_accumulate;
    wire out_input_1_input_0_payload_Some_0_laddr_read_full_acc_row;
    wire [3-1:0] out_input_1_input_0_payload_Some_0_laddr_norm_cmd;
    wire [11-1:0] out_input_1_input_0_payload_Some_0_laddr_garbage;
    wire out_input_1_input_0_payload_Some_0_laddr_is_garbage;
    wire [14-1:0] out_input_1_input_0_payload_Some_0_laddr_data;
    wire [16-1:0] out_input_1_input_0_payload_Some_0_cols;
    wire [16-1:0] out_input_1_input_0_payload_Some_0_repeats;
    wire [32-1:0] out_input_1_input_0_payload_Some_0_scale;
    wire out_input_1_input_0_payload_Some_0_has_acc_bitwidth;
    wire out_input_1_input_0_payload_Some_0_all_zeros;
    wire [16-1:0] out_input_1_input_0_payload_Some_0_block_stride;
    wire [8-1:0] out_input_1_input_0_payload_Some_0_pixel_repeats;
    wire [8-1:0] out_input_1_input_0_payload_Some_0_cmd_id;
    wire out_input_1_input_0_payload_Some_0_status_debug;
    wire out_input_1_input_0_payload_Some_0_status_cease;
    wire out_input_1_input_0_payload_Some_0_status_wfi;
    wire [32-1:0] out_input_1_input_0_payload_Some_0_status_isa;
    wire [2-1:0] out_input_1_input_0_payload_Some_0_status_dprv;
    wire out_input_1_input_0_payload_Some_0_status_dv;
    wire [2-1:0] out_input_1_input_0_payload_Some_0_status_prv;
    wire out_input_1_input_0_payload_Some_0_status_v;
    wire out_input_1_input_0_payload_Some_0_status_sd;
    wire [23-1:0] out_input_1_input_0_payload_Some_0_status_zero2;
    wire out_input_1_input_0_payload_Some_0_status_mpv;
    wire out_input_1_input_0_payload_Some_0_status_gva;
    wire out_input_1_input_0_payload_Some_0_status_mbe;
    wire out_input_1_input_0_payload_Some_0_status_sbe;
    wire [2-1:0] out_input_1_input_0_payload_Some_0_status_sxl;
    wire [2-1:0] out_input_1_input_0_payload_Some_0_status_uxl;
    wire out_input_1_input_0_payload_Some_0_status_sd_rv32;
    wire [8-1:0] out_input_1_input_0_payload_Some_0_status_zero1;
    wire out_input_1_input_0_payload_Some_0_status_tsr;
    wire out_input_1_input_0_payload_Some_0_status_tw;
    wire out_input_1_input_0_payload_Some_0_status_tvm;
    wire out_input_1_input_0_payload_Some_0_status_mxr;
    wire out_input_1_input_0_payload_Some_0_status_sum;
    wire out_input_1_input_0_payload_Some_0_status_mprv;
    wire [2-1:0] out_input_1_input_0_payload_Some_0_status_xs;
    wire [2-1:0] out_input_1_input_0_payload_Some_0_status_fs;
    wire [2-1:0] out_input_1_input_0_payload_Some_0_status_mpp;
    wire [2-1:0] out_input_1_input_0_payload_Some_0_status_vs;
    wire out_input_1_input_0_payload_Some_0_status_spp;
    wire out_input_1_input_0_payload_Some_0_status_mpie;
    wire out_input_1_input_0_payload_Some_0_status_ube;
    wire out_input_1_input_0_payload_Some_0_status_spie;
    wire out_input_1_input_0_payload_Some_0_status_upie;
    wire out_input_1_input_0_payload_Some_0_status_mie;
    wire out_input_1_input_0_payload_Some_0_status_hie;
    wire out_input_1_input_0_payload_Some_0_status_sie;
    wire out_input_1_input_0_payload_Some_0_status_uie;
    wire out_input_1_input_0_resolver_ready = io_dma_req_ready;

    wire out_output_payload_discriminant;
    wire [6-1:0] out_output_payload_Some_0;
    wire out_output_resolver_ready = io_completed_ready;

    load_default_top load_default(
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
        .in_input_1_output_payload_Some_0_bytes_read(in_input_1_output_payload_Some_0_bytes_read),
        .in_input_1_output_payload_Some_0_cmd_id(in_input_1_output_payload_Some_0_cmd_id),
        .out_input_1_input_0_payload_discriminant(out_input_1_input_0_payload_discriminant),
        .out_input_1_input_0_payload_Some_0_vaddr(out_input_1_input_0_payload_Some_0_vaddr),
        .out_input_1_input_0_payload_Some_0_laddr_is_acc_addr(out_input_1_input_0_payload_Some_0_laddr_is_acc_addr),
        .out_input_1_input_0_payload_Some_0_laddr_accumulate(out_input_1_input_0_payload_Some_0_laddr_accumulate),
        .out_input_1_input_0_payload_Some_0_laddr_read_full_acc_row(out_input_1_input_0_payload_Some_0_laddr_read_full_acc_row),
        .out_input_1_input_0_payload_Some_0_laddr_norm_cmd(out_input_1_input_0_payload_Some_0_laddr_norm_cmd),
        .out_input_1_input_0_payload_Some_0_laddr_garbage(out_input_1_input_0_payload_Some_0_laddr_garbage),
        .out_input_1_input_0_payload_Some_0_laddr_is_garbage(out_input_1_input_0_payload_Some_0_laddr_is_garbage),
        .out_input_1_input_0_payload_Some_0_laddr_data(out_input_1_input_0_payload_Some_0_laddr_data),
        .out_input_1_input_0_payload_Some_0_cols(out_input_1_input_0_payload_Some_0_cols),
        .out_input_1_input_0_payload_Some_0_repeats(out_input_1_input_0_payload_Some_0_repeats),
        .out_input_1_input_0_payload_Some_0_scale(out_input_1_input_0_payload_Some_0_scale),
        .out_input_1_input_0_payload_Some_0_has_acc_bitwidth(out_input_1_input_0_payload_Some_0_has_acc_bitwidth),
        .out_input_1_input_0_payload_Some_0_all_zeros(out_input_1_input_0_payload_Some_0_all_zeros),
        .out_input_1_input_0_payload_Some_0_block_stride(out_input_1_input_0_payload_Some_0_block_stride),
        .out_input_1_input_0_payload_Some_0_pixel_repeats(out_input_1_input_0_payload_Some_0_pixel_repeats),
        .out_input_1_input_0_payload_Some_0_cmd_id(out_input_1_input_0_payload_Some_0_cmd_id),
        .out_input_1_input_0_payload_Some_0_status_debug(out_input_1_input_0_payload_Some_0_status_debug),
        .out_input_1_input_0_payload_Some_0_status_cease(out_input_1_input_0_payload_Some_0_status_cease),
        .out_input_1_input_0_payload_Some_0_status_wfi(out_input_1_input_0_payload_Some_0_status_wfi),
        .out_input_1_input_0_payload_Some_0_status_isa(out_input_1_input_0_payload_Some_0_status_isa),
        .out_input_1_input_0_payload_Some_0_status_dprv(out_input_1_input_0_payload_Some_0_status_dprv),
        .out_input_1_input_0_payload_Some_0_status_dv(out_input_1_input_0_payload_Some_0_status_dv),
        .out_input_1_input_0_payload_Some_0_status_prv(out_input_1_input_0_payload_Some_0_status_prv),
        .out_input_1_input_0_payload_Some_0_status_v(out_input_1_input_0_payload_Some_0_status_v),
        .out_input_1_input_0_payload_Some_0_status_sd(out_input_1_input_0_payload_Some_0_status_sd),
        .out_input_1_input_0_payload_Some_0_status_zero2(out_input_1_input_0_payload_Some_0_status_zero2),
        .out_input_1_input_0_payload_Some_0_status_mpv(out_input_1_input_0_payload_Some_0_status_mpv),
        .out_input_1_input_0_payload_Some_0_status_gva(out_input_1_input_0_payload_Some_0_status_gva),
        .out_input_1_input_0_payload_Some_0_status_mbe(out_input_1_input_0_payload_Some_0_status_mbe),
        .out_input_1_input_0_payload_Some_0_status_sbe(out_input_1_input_0_payload_Some_0_status_sbe),
        .out_input_1_input_0_payload_Some_0_status_sxl(out_input_1_input_0_payload_Some_0_status_sxl),
        .out_input_1_input_0_payload_Some_0_status_uxl(out_input_1_input_0_payload_Some_0_status_uxl),
        .out_input_1_input_0_payload_Some_0_status_sd_rv32(out_input_1_input_0_payload_Some_0_status_sd_rv32),
        .out_input_1_input_0_payload_Some_0_status_zero1(out_input_1_input_0_payload_Some_0_status_zero1),
        .out_input_1_input_0_payload_Some_0_status_tsr(out_input_1_input_0_payload_Some_0_status_tsr),
        .out_input_1_input_0_payload_Some_0_status_tw(out_input_1_input_0_payload_Some_0_status_tw),
        .out_input_1_input_0_payload_Some_0_status_tvm(out_input_1_input_0_payload_Some_0_status_tvm),
        .out_input_1_input_0_payload_Some_0_status_mxr(out_input_1_input_0_payload_Some_0_status_mxr),
        .out_input_1_input_0_payload_Some_0_status_sum(out_input_1_input_0_payload_Some_0_status_sum),
        .out_input_1_input_0_payload_Some_0_status_mprv(out_input_1_input_0_payload_Some_0_status_mprv),
        .out_input_1_input_0_payload_Some_0_status_xs(out_input_1_input_0_payload_Some_0_status_xs),
        .out_input_1_input_0_payload_Some_0_status_fs(out_input_1_input_0_payload_Some_0_status_fs),
        .out_input_1_input_0_payload_Some_0_status_mpp(out_input_1_input_0_payload_Some_0_status_mpp),
        .out_input_1_input_0_payload_Some_0_status_vs(out_input_1_input_0_payload_Some_0_status_vs),
        .out_input_1_input_0_payload_Some_0_status_spp(out_input_1_input_0_payload_Some_0_status_spp),
        .out_input_1_input_0_payload_Some_0_status_mpie(out_input_1_input_0_payload_Some_0_status_mpie),
        .out_input_1_input_0_payload_Some_0_status_ube(out_input_1_input_0_payload_Some_0_status_ube),
        .out_input_1_input_0_payload_Some_0_status_spie(out_input_1_input_0_payload_Some_0_status_spie),
        .out_input_1_input_0_payload_Some_0_status_upie(out_input_1_input_0_payload_Some_0_status_upie),
        .out_input_1_input_0_payload_Some_0_status_mie(out_input_1_input_0_payload_Some_0_status_mie),
        .out_input_1_input_0_payload_Some_0_status_hie(out_input_1_input_0_payload_Some_0_status_hie),
        .out_input_1_input_0_payload_Some_0_status_sie(out_input_1_input_0_payload_Some_0_status_sie),
        .out_input_1_input_0_payload_Some_0_status_uie(out_input_1_input_0_payload_Some_0_status_uie),
        .out_input_1_input_0_resolver_ready(out_input_1_input_0_resolver_ready),

        .out_output_payload_discriminant(out_output_payload_discriminant),
        .out_output_payload_Some_0(out_output_payload_Some_0),
        .out_output_resolver_ready(out_output_resolver_ready)
    );

    assign io_cmd_ready = in_input_0_resolver_ready;

    assign io_dma_req_valid = out_input_1_input_0_payload_discriminant;                   
    assign io_dma_req_bits_vaddr = out_input_1_input_0_payload_Some_0_vaddr;                   
    assign io_dma_req_bits_laddr_is_acc_addr = out_input_1_input_0_payload_Some_0_laddr_is_acc_addr;       
    assign io_dma_req_bits_laddr_accumulate = out_input_1_input_0_payload_Some_0_laddr_accumulate;        
    assign io_dma_req_bits_laddr_read_full_acc_row = out_input_1_input_0_payload_Some_0_laddr_read_full_acc_row; 
    assign io_dma_req_bits_laddr_norm_cmd = out_input_1_input_0_payload_Some_0_laddr_norm_cmd;          
    assign io_dma_req_bits_laddr_garbage = out_input_1_input_0_payload_Some_0_laddr_garbage;           
    assign io_dma_req_bits_laddr_garbage_bit = out_input_1_input_0_payload_Some_0_laddr_is_garbage;        
    assign io_dma_req_bits_laddr_data = out_input_1_input_0_payload_Some_0_laddr_data;              
    assign io_dma_req_bits_cols = out_input_1_input_0_payload_Some_0_cols;                    
    assign io_dma_req_bits_repeats = out_input_1_input_0_payload_Some_0_repeats;                 
    assign io_dma_req_bits_scale = out_input_1_input_0_payload_Some_0_scale;                   
    assign io_dma_req_bits_has_acc_bitwidth = out_input_1_input_0_payload_Some_0_has_acc_bitwidth;        
    assign io_dma_req_bits_all_zeros = out_input_1_input_0_payload_Some_0_all_zeros;               
    assign io_dma_req_bits_block_stride = out_input_1_input_0_payload_Some_0_block_stride;            
    assign io_dma_req_bits_pixel_repeats = out_input_1_input_0_payload_Some_0_pixel_repeats;           
    assign io_dma_req_bits_cmd_id = out_input_1_input_0_payload_Some_0_cmd_id;                  
    assign io_dma_req_bits_status_debug = out_input_1_input_0_payload_Some_0_status_debug;            
    assign io_dma_req_bits_status_cease = out_input_1_input_0_payload_Some_0_status_cease;            
    assign io_dma_req_bits_status_wfi = out_input_1_input_0_payload_Some_0_status_wfi;              
    assign io_dma_req_bits_status_isa = out_input_1_input_0_payload_Some_0_status_isa;              
    assign io_dma_req_bits_status_dprv = out_input_1_input_0_payload_Some_0_status_dprv;             
    assign io_dma_req_bits_status_dv = out_input_1_input_0_payload_Some_0_status_dv;               
    assign io_dma_req_bits_status_prv = out_input_1_input_0_payload_Some_0_status_prv;              
    assign io_dma_req_bits_status_v = out_input_1_input_0_payload_Some_0_status_v;                
    assign io_dma_req_bits_status_sd = out_input_1_input_0_payload_Some_0_status_sd;               
    assign io_dma_req_bits_status_zero2 = out_input_1_input_0_payload_Some_0_status_zero2;            
    assign io_dma_req_bits_status_mpv = out_input_1_input_0_payload_Some_0_status_mpv;              
    assign io_dma_req_bits_status_gva = out_input_1_input_0_payload_Some_0_status_gva;              
    assign io_dma_req_bits_status_mbe = out_input_1_input_0_payload_Some_0_status_mbe;              
    assign io_dma_req_bits_status_sbe = out_input_1_input_0_payload_Some_0_status_sbe;              
    assign io_dma_req_bits_status_sxl = out_input_1_input_0_payload_Some_0_status_sxl;              
    assign io_dma_req_bits_status_uxl = out_input_1_input_0_payload_Some_0_status_uxl;              
    assign io_dma_req_bits_status_sd_rv32 = out_input_1_input_0_payload_Some_0_status_sd_rv32;          
    assign io_dma_req_bits_status_zero1 = out_input_1_input_0_payload_Some_0_status_zero1;            
    assign io_dma_req_bits_status_tsr = out_input_1_input_0_payload_Some_0_status_tsr;              
    assign io_dma_req_bits_status_tw = out_input_1_input_0_payload_Some_0_status_tw;               
    assign io_dma_req_bits_status_tvm = out_input_1_input_0_payload_Some_0_status_tvm;              
    assign io_dma_req_bits_status_mxr = out_input_1_input_0_payload_Some_0_status_mxr;              
    assign io_dma_req_bits_status_sum = out_input_1_input_0_payload_Some_0_status_sum;              
    assign io_dma_req_bits_status_mprv = out_input_1_input_0_payload_Some_0_status_mprv;             
    assign io_dma_req_bits_status_xs = out_input_1_input_0_payload_Some_0_status_xs;               
    assign io_dma_req_bits_status_fs = out_input_1_input_0_payload_Some_0_status_fs;               
    assign io_dma_req_bits_status_mpp = out_input_1_input_0_payload_Some_0_status_mpp;              
    assign io_dma_req_bits_status_vs = out_input_1_input_0_payload_Some_0_status_vs;               
    assign io_dma_req_bits_status_spp = out_input_1_input_0_payload_Some_0_status_spp;              
    assign io_dma_req_bits_status_mpie = out_input_1_input_0_payload_Some_0_status_mpie;             
    assign io_dma_req_bits_status_ube = out_input_1_input_0_payload_Some_0_status_ube;              
    assign io_dma_req_bits_status_spie = out_input_1_input_0_payload_Some_0_status_spie;             
    assign io_dma_req_bits_status_upie = out_input_1_input_0_payload_Some_0_status_upie;             
    assign io_dma_req_bits_status_mie = out_input_1_input_0_payload_Some_0_status_mie;              
    assign io_dma_req_bits_status_hie = out_input_1_input_0_payload_Some_0_status_hie;              
    assign io_dma_req_bits_status_sie = out_input_1_input_0_payload_Some_0_status_sie;              
    assign io_dma_req_bits_status_uie = out_input_1_input_0_payload_Some_0_status_uie;              

    assign io_completed_valid = out_output_payload_discriminant;
    assign io_completed_bits = out_output_payload_Some_0;
endmodule
