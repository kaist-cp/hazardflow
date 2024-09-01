import re

"""
Convert Chisel `io` to Verilog.
Used for LoadControllerBlackBox.v and StoreControllerBlackBox.v, but may not be generally applicable.
"""

chisel = r"""
    val clock = Input(Clock())
    val reset = Input(Bool())

    val io_cmd_ready = Output(Bool())
    val io_cmd_valid = Input(Bool())
    val io_cmd_bits_cmd_inst_funct = Input(Bits(7.W))
    val io_cmd_bits_cmd_inst_rs2 = Input(Bits(5.W))
    val io_cmd_bits_cmd_inst_rs1 = Input(Bits(5.W))
    val io_cmd_bits_cmd_inst_xd = Input(Bool())
    val io_cmd_bits_cmd_inst_xs1 = Input(Bool())
    val io_cmd_bits_cmd_inst_xs2 = Input(Bool())
    val io_cmd_bits_cmd_inst_rd = Input(Bits(5.W))
    val io_cmd_bits_cmd_inst_opcode = Input(Bits(7.W))
    val io_cmd_bits_cmd_rs1 = Input(Bits(64.W)) // xLen = 64
    val io_cmd_bits_cmd_rs2 = Input(Bits(64.W)) // xLen = 64
    val io_cmd_bits_cmd_status_debug = Input(Bool())
    val io_cmd_bits_cmd_status_cease = Input(Bool())
    val io_cmd_bits_cmd_status_wfi = Input(Bool())
    val io_cmd_bits_cmd_status_isa = Input(UInt(32.W))
    val io_cmd_bits_cmd_status_dprv = Input(UInt(2.W)) // PRV.SZ = 2
    val io_cmd_bits_cmd_status_dv = Input(Bool())
    val io_cmd_bits_cmd_status_prv = Input(UInt(2.W)) // PRV.SZ = 2
    val io_cmd_bits_cmd_status_v = Input(Bool())
    val io_cmd_bits_cmd_status_sd = Input(Bool())
    val io_cmd_bits_cmd_status_zero2 = Input(UInt(23.W))
    val io_cmd_bits_cmd_status_mpv = Input(Bool())
    val io_cmd_bits_cmd_status_gva = Input(Bool())
    val io_cmd_bits_cmd_status_mbe = Input(Bool())
    val io_cmd_bits_cmd_status_sbe = Input(Bool())
    val io_cmd_bits_cmd_status_sxl = Input(UInt(2.W))
    val io_cmd_bits_cmd_status_uxl = Input(UInt(2.W))
    val io_cmd_bits_cmd_status_sd_rv32 = Input(Bool())
    val io_cmd_bits_cmd_status_zero1 = Input(UInt(8.W))
    val io_cmd_bits_cmd_status_tsr = Input(Bool())
    val io_cmd_bits_cmd_status_tw = Input(Bool())
    val io_cmd_bits_cmd_status_tvm = Input(Bool())
    val io_cmd_bits_cmd_status_mxr = Input(Bool())
    val io_cmd_bits_cmd_status_sum = Input(Bool())
    val io_cmd_bits_cmd_status_mprv = Input(Bool())
    val io_cmd_bits_cmd_status_xs = Input(UInt(2.W))
    val io_cmd_bits_cmd_status_fs = Input(UInt(2.W))
    val io_cmd_bits_cmd_status_mpp = Input(UInt(2.W))
    val io_cmd_bits_cmd_status_vs = Input(UInt(2.W))
    val io_cmd_bits_cmd_status_spp = Input(UInt(1.W))
    val io_cmd_bits_cmd_status_mpie = Input(Bool())
    val io_cmd_bits_cmd_status_ube = Input(Bool())
    val io_cmd_bits_cmd_status_spie = Input(Bool())
    val io_cmd_bits_cmd_status_upie = Input(Bool())
    val io_cmd_bits_cmd_status_mie = Input(Bool())
    val io_cmd_bits_cmd_status_hie = Input(Bool())
    val io_cmd_bits_cmd_status_sie = Input(Bool())
    val io_cmd_bits_cmd_status_uie = Input(Bool())
    val io_cmd_bits_rob_id_valid = Input(Bool())
    val io_cmd_bits_rob_id_bits = Input(UInt(log2Up(reservation_station_entries).W))
    val io_cmd_bits_from_matmul_fsm = Input(Bool())
    val io_cmd_bits_from_conv_fsm = Input(Bool())

    val io_dma_req_ready = Input(Bool())
    val io_dma_req_valid = Output(Bool())
    val io_dma_req_bits_vaddr = Output(UInt(40.W)) // coreMaxAddrBits = 40
    val io_dma_req_bits_laddr_is_acc_addr = Output(Bool())
    val io_dma_req_bits_laddr_accumulate = Output(Bool())
    val io_dma_req_bits_laddr_read_full_acc_row = Output(Bool())
    val io_dma_req_bits_laddr_norm_cmd = Output(NormCmd())
    val io_dma_req_bits_laddr_garbage = Output(UInt(11.W)) // (localAddrBits - maxAddrBits - metadata_w - 1) max 0 = 11
    val io_dma_req_bits_laddr_garbage_bit = Output(UInt(1.W)) // localAddrBits - maxAddrBits >= metadata_w + 1
    val io_dma_req_bits_laddr_data = Output(UInt(14.W)) // maxAddrBits = 14
    val io_dma_req_bits_acc_act = Output(UInt(3.W)) // Activation.bitwidth = 3
    val io_dma_req_bits_acc_scale = Output(UInt(acc_scale_t_bits.W))
    val io_dma_req_bits_acc_igelu_qb = Output(UInt(accType.getWidth.W))
    val io_dma_req_bits_acc_igelu_qc = Output(UInt(accType.getWidth.W))
    val io_dma_req_bits_acc_iexp_qln2 = Output(UInt(accType.getWidth.W))
    val io_dma_req_bits_acc_iexp_qln2_inv = Output(UInt(accType.getWidth.W))
    val io_dma_req_bits_acc_norm_stats_id = Output(UInt(8.W))
    val io_dma_req_bits_len = Output(UInt(16.W))
    val io_dma_req_bits_block = Output(UInt(8.W))
    val io_dma_req_bits_cmd_id = Output(UInt(8.W))
    val io_dma_req_bits_status_debug = Output(Bool())
    val io_dma_req_bits_status_cease = Output(Bool())
    val io_dma_req_bits_status_wfi = Output(Bool())
    val io_dma_req_bits_status_isa = Output(UInt(32.W))
    val io_dma_req_bits_status_dprv = Output(UInt(2.W)) // PRV.SZ = 2
    val io_dma_req_bits_status_dv = Output(Bool())
    val io_dma_req_bits_status_prv = Output(UInt(2.W)) // PRV.SZ = 2
    val io_dma_req_bits_status_v = Output(Bool())
    val io_dma_req_bits_status_sd = Output(Bool())
    val io_dma_req_bits_status_zero2 = Output(UInt(23.W))
    val io_dma_req_bits_status_mpv = Output(Bool())
    val io_dma_req_bits_status_gva = Output(Bool())
    val io_dma_req_bits_status_mbe = Output(Bool())
    val io_dma_req_bits_status_sbe = Output(Bool())
    val io_dma_req_bits_status_sxl = Output(UInt(2.W))
    val io_dma_req_bits_status_uxl = Output(UInt(2.W))
    val io_dma_req_bits_status_sd_rv32 = Output(Bool())
    val io_dma_req_bits_status_zero1 = Output(UInt(8.W))
    val io_dma_req_bits_status_tsr = Output(Bool())
    val io_dma_req_bits_status_tw = Output(Bool())
    val io_dma_req_bits_status_tvm = Output(Bool())
    val io_dma_req_bits_status_mxr = Output(Bool())
    val io_dma_req_bits_status_sum = Output(Bool())
    val io_dma_req_bits_status_mprv = Output(Bool())
    val io_dma_req_bits_status_xs = Output(UInt(2.W))
    val io_dma_req_bits_status_fs = Output(UInt(2.W))
    val io_dma_req_bits_status_mpp = Output(UInt(2.W))
    val io_dma_req_bits_status_vs = Output(UInt(2.W))
    val io_dma_req_bits_status_spp = Output(UInt(1.W))
    val io_dma_req_bits_status_mpie = Output(Bool())
    val io_dma_req_bits_status_ube = Output(Bool())
    val io_dma_req_bits_status_spie = Output(Bool())
    val io_dma_req_bits_status_upie = Output(Bool())
    val io_dma_req_bits_status_mie = Output(Bool())
    val io_dma_req_bits_status_hie = Output(Bool())
    val io_dma_req_bits_status_sie = Output(Bool())
    val io_dma_req_bits_status_uie = Output(Bool())
    val io_dma_req_bits_pool_en = Output(Bool())
    val io_dma_req_bits_store_en = Output(Bool())
    val io_dma_resp_valid = Input(Bool())
    val io_dma_resp_bits_cmd_id = Input(UInt(8.W))

    val io_completed_ready = Input(Bool())
    val io_completed_valid = Output(Bool())
    val io_completed_bits = Output(UInt(log2Up(reservation_station_entries).W))
"""

chisel_line_re = re.compile(r"val (\w+) = (\w+)\(\w+\(((.+)\.W)?\)\)")
verilog = ""
for chisel_line in chisel.strip().splitlines():
    chisel_line = chisel_line.strip()
    if chisel_line == "":
        verilog += "\n"
        continue
    # print(chisel_line)
    match = chisel_line_re.match(chisel_line)
    name, inout, _, bits = match.group(1, 2, 3, 4)
    # print((name, inout, bits))

    verilog_line = "    "
    if inout == "Input":
        verilog_line += "input "
    elif inout == "Output":
        verilog_line += "output "
    if not (bits == None or bits == "1"):
        verilog_line += f"[{bits}-1:0] "
    verilog_line += name
    verilog_line += ","
    # print(verilog_line)

    verilog += verilog_line + "\n"

print()
print(verilog)
