
package gemmini

import chisel3._
import chisel3.util._
import GemminiISA._
import Util._
import org.chipsalliance.cde.config.Parameters
import midas.targetutils.PerfCounter

class LoadControllerBlackBoxAdapter[T <: Data, U <: Data, V <: Data](config: GemminiArrayConfig[T, U, V], coreMaxAddrBits: Int,
                                                                     local_addr_t: LocalAddr)
  extends BlackBox(Map("LOG_2_UP_RESERVATION_STATION_ENTRIES" -> log2Up(config.reservation_station_entries),
                       "MVIN_SCALE_T_BITS" -> config.mvin_scale_t_bits))
    with HasBlackBoxResource {
  import config._

  val io = IO(new Bundle {
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
    val io_dma_req_bits_cols = Output(UInt(16.W))
    val io_dma_req_bits_repeats = Output(UInt(16.W))
    val io_dma_req_bits_scale = Output(UInt(mvin_scale_t_bits.W))
    val io_dma_req_bits_has_acc_bitwidth = Output(Bool())
    val io_dma_req_bits_all_zeros = Output(Bool())
    val io_dma_req_bits_block_stride = Output(UInt(16.W))
    val io_dma_req_bits_pixel_repeats = Output(UInt(8.W))
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
    val io_dma_resp_valid = Input(Bool())
    val io_dma_resp_bits_bytesRead = Input(UInt(16.W))
    val io_dma_resp_bits_cmd_id = Input(UInt(8.W))

    val io_completed_ready = Input(Bool())
    val io_completed_valid = Output(Bool())
    val io_completed_bits = Output(UInt(log2Up(reservation_station_entries).W))
  })
  addResource("/vsrc/LoadControllerBlackBox.v")
}

class LoadController[T <: Data, U <: Data, V <: Data](config: GemminiArrayConfig[T, U, V], coreMaxAddrBits: Int,
                                                      local_addr_t: LocalAddr)
                               (implicit p: Parameters) extends Module {
  import config._

  val io = IO(new Bundle {
    val cmd = Flipped(Decoupled(new GemminiCmd(reservation_station_entries)))

    val dma = new ScratchpadReadMemIO(local_addr_t, mvin_scale_t_bits)

    val completed = Decoupled(UInt(log2Up(reservation_station_entries).W))

    // val busy = Output(Bool())

    // val counter = new CounterEventIO()
  })

  val custom_load_controller = Module(new LoadControllerBlackBoxAdapter(config, coreMaxAddrBits, local_addr_t))

  custom_load_controller.io.clock := clock
  custom_load_controller.io.reset := reset

  io.cmd.ready := custom_load_controller.io.io_cmd_ready
  custom_load_controller.io.io_cmd_valid := io.cmd.valid
  custom_load_controller.io.io_cmd_bits_cmd_inst_funct := io.cmd.bits.cmd.inst.funct
  custom_load_controller.io.io_cmd_bits_cmd_inst_rs2 := io.cmd.bits.cmd.inst.rs2
  custom_load_controller.io.io_cmd_bits_cmd_inst_rs1 := io.cmd.bits.cmd.inst.rs1
  custom_load_controller.io.io_cmd_bits_cmd_inst_xd := io.cmd.bits.cmd.inst.xd
  custom_load_controller.io.io_cmd_bits_cmd_inst_xs1 := io.cmd.bits.cmd.inst.xs1
  custom_load_controller.io.io_cmd_bits_cmd_inst_xs2 := io.cmd.bits.cmd.inst.xs2
  custom_load_controller.io.io_cmd_bits_cmd_inst_rd := io.cmd.bits.cmd.inst.rd
  custom_load_controller.io.io_cmd_bits_cmd_inst_opcode := io.cmd.bits.cmd.inst.opcode
  custom_load_controller.io.io_cmd_bits_cmd_rs1 := io.cmd.bits.cmd.rs1
  custom_load_controller.io.io_cmd_bits_cmd_rs2 := io.cmd.bits.cmd.rs2
  custom_load_controller.io.io_cmd_bits_cmd_status_debug := io.cmd.bits.cmd.status.debug
  custom_load_controller.io.io_cmd_bits_cmd_status_cease := io.cmd.bits.cmd.status.cease
  custom_load_controller.io.io_cmd_bits_cmd_status_wfi := io.cmd.bits.cmd.status.wfi
  custom_load_controller.io.io_cmd_bits_cmd_status_isa := io.cmd.bits.cmd.status.isa
  custom_load_controller.io.io_cmd_bits_cmd_status_dprv := io.cmd.bits.cmd.status.dprv
  custom_load_controller.io.io_cmd_bits_cmd_status_dv := io.cmd.bits.cmd.status.dv
  custom_load_controller.io.io_cmd_bits_cmd_status_prv := io.cmd.bits.cmd.status.prv
  custom_load_controller.io.io_cmd_bits_cmd_status_v := io.cmd.bits.cmd.status.v
  custom_load_controller.io.io_cmd_bits_cmd_status_sd := io.cmd.bits.cmd.status.sd
  custom_load_controller.io.io_cmd_bits_cmd_status_zero2 := io.cmd.bits.cmd.status.zero2
  custom_load_controller.io.io_cmd_bits_cmd_status_mpv := io.cmd.bits.cmd.status.mpv
  custom_load_controller.io.io_cmd_bits_cmd_status_gva := io.cmd.bits.cmd.status.gva
  custom_load_controller.io.io_cmd_bits_cmd_status_mbe := io.cmd.bits.cmd.status.mbe
  custom_load_controller.io.io_cmd_bits_cmd_status_sbe := io.cmd.bits.cmd.status.sbe
  custom_load_controller.io.io_cmd_bits_cmd_status_sxl := io.cmd.bits.cmd.status.sxl
  custom_load_controller.io.io_cmd_bits_cmd_status_uxl := io.cmd.bits.cmd.status.uxl
  custom_load_controller.io.io_cmd_bits_cmd_status_sd_rv32 := io.cmd.bits.cmd.status.sd_rv32
  custom_load_controller.io.io_cmd_bits_cmd_status_zero1 := io.cmd.bits.cmd.status.zero1
  custom_load_controller.io.io_cmd_bits_cmd_status_tsr := io.cmd.bits.cmd.status.tsr
  custom_load_controller.io.io_cmd_bits_cmd_status_tw := io.cmd.bits.cmd.status.tw
  custom_load_controller.io.io_cmd_bits_cmd_status_tvm := io.cmd.bits.cmd.status.tvm
  custom_load_controller.io.io_cmd_bits_cmd_status_mxr := io.cmd.bits.cmd.status.mxr
  custom_load_controller.io.io_cmd_bits_cmd_status_sum := io.cmd.bits.cmd.status.sum
  custom_load_controller.io.io_cmd_bits_cmd_status_mprv := io.cmd.bits.cmd.status.mprv
  custom_load_controller.io.io_cmd_bits_cmd_status_xs := io.cmd.bits.cmd.status.xs
  custom_load_controller.io.io_cmd_bits_cmd_status_fs := io.cmd.bits.cmd.status.fs
  custom_load_controller.io.io_cmd_bits_cmd_status_mpp := io.cmd.bits.cmd.status.mpp
  custom_load_controller.io.io_cmd_bits_cmd_status_vs := io.cmd.bits.cmd.status.vs
  custom_load_controller.io.io_cmd_bits_cmd_status_spp := io.cmd.bits.cmd.status.spp
  custom_load_controller.io.io_cmd_bits_cmd_status_mpie := io.cmd.bits.cmd.status.mpie
  custom_load_controller.io.io_cmd_bits_cmd_status_ube := io.cmd.bits.cmd.status.ube
  custom_load_controller.io.io_cmd_bits_cmd_status_spie := io.cmd.bits.cmd.status.spie
  custom_load_controller.io.io_cmd_bits_cmd_status_upie := io.cmd.bits.cmd.status.upie
  custom_load_controller.io.io_cmd_bits_cmd_status_mie := io.cmd.bits.cmd.status.mie
  custom_load_controller.io.io_cmd_bits_cmd_status_hie := io.cmd.bits.cmd.status.hie
  custom_load_controller.io.io_cmd_bits_cmd_status_sie := io.cmd.bits.cmd.status.sie
  custom_load_controller.io.io_cmd_bits_cmd_status_uie := io.cmd.bits.cmd.status.uie
  custom_load_controller.io.io_cmd_bits_rob_id_valid := io.cmd.bits.rob_id.valid
  custom_load_controller.io.io_cmd_bits_rob_id_bits := io.cmd.bits.rob_id.bits
  custom_load_controller.io.io_cmd_bits_from_matmul_fsm := io.cmd.bits.from_matmul_fsm
  custom_load_controller.io.io_cmd_bits_from_conv_fsm := io.cmd.bits.from_conv_fsm

  custom_load_controller.io.io_dma_req_ready := io.dma.req.ready
  io.dma.req.valid := custom_load_controller.io.io_dma_req_valid
  io.dma.req.bits.vaddr := custom_load_controller.io.io_dma_req_bits_vaddr
  io.dma.req.bits.laddr.is_acc_addr := custom_load_controller.io.io_dma_req_bits_laddr_is_acc_addr
  io.dma.req.bits.laddr.accumulate := custom_load_controller.io.io_dma_req_bits_laddr_accumulate
  io.dma.req.bits.laddr.read_full_acc_row := custom_load_controller.io.io_dma_req_bits_laddr_read_full_acc_row
  io.dma.req.bits.laddr.norm_cmd := custom_load_controller.io.io_dma_req_bits_laddr_norm_cmd
  io.dma.req.bits.laddr.garbage := custom_load_controller.io.io_dma_req_bits_laddr_garbage
  io.dma.req.bits.laddr.garbage_bit := custom_load_controller.io.io_dma_req_bits_laddr_garbage_bit
  io.dma.req.bits.laddr.data := custom_load_controller.io.io_dma_req_bits_laddr_data
  io.dma.req.bits.cols := custom_load_controller.io.io_dma_req_bits_cols
  io.dma.req.bits.repeats := custom_load_controller.io.io_dma_req_bits_repeats
  io.dma.req.bits.scale := custom_load_controller.io.io_dma_req_bits_scale
  io.dma.req.bits.has_acc_bitwidth := custom_load_controller.io.io_dma_req_bits_has_acc_bitwidth
  io.dma.req.bits.all_zeros := custom_load_controller.io.io_dma_req_bits_all_zeros
  io.dma.req.bits.block_stride := custom_load_controller.io.io_dma_req_bits_block_stride
  io.dma.req.bits.pixel_repeats := custom_load_controller.io.io_dma_req_bits_pixel_repeats
  io.dma.req.bits.cmd_id := custom_load_controller.io.io_dma_req_bits_cmd_id
  io.dma.req.bits.status.debug := custom_load_controller.io.io_dma_req_bits_status_debug
  io.dma.req.bits.status.cease := custom_load_controller.io.io_dma_req_bits_status_cease
  io.dma.req.bits.status.wfi := custom_load_controller.io.io_dma_req_bits_status_wfi
  io.dma.req.bits.status.isa := custom_load_controller.io.io_dma_req_bits_status_isa
  io.dma.req.bits.status.dprv := custom_load_controller.io.io_dma_req_bits_status_dprv
  io.dma.req.bits.status.dv := custom_load_controller.io.io_dma_req_bits_status_dv
  io.dma.req.bits.status.prv := custom_load_controller.io.io_dma_req_bits_status_prv
  io.dma.req.bits.status.v := custom_load_controller.io.io_dma_req_bits_status_v
  io.dma.req.bits.status.sd := custom_load_controller.io.io_dma_req_bits_status_sd
  io.dma.req.bits.status.zero2 := custom_load_controller.io.io_dma_req_bits_status_zero2
  io.dma.req.bits.status.mpv := custom_load_controller.io.io_dma_req_bits_status_mpv
  io.dma.req.bits.status.gva := custom_load_controller.io.io_dma_req_bits_status_gva
  io.dma.req.bits.status.mbe := custom_load_controller.io.io_dma_req_bits_status_mbe
  io.dma.req.bits.status.sbe := custom_load_controller.io.io_dma_req_bits_status_sbe
  io.dma.req.bits.status.sxl := custom_load_controller.io.io_dma_req_bits_status_sxl
  io.dma.req.bits.status.uxl := custom_load_controller.io.io_dma_req_bits_status_uxl
  io.dma.req.bits.status.sd_rv32 := custom_load_controller.io.io_dma_req_bits_status_sd_rv32
  io.dma.req.bits.status.zero1 := custom_load_controller.io.io_dma_req_bits_status_zero1
  io.dma.req.bits.status.tsr := custom_load_controller.io.io_dma_req_bits_status_tsr
  io.dma.req.bits.status.tw := custom_load_controller.io.io_dma_req_bits_status_tw
  io.dma.req.bits.status.tvm := custom_load_controller.io.io_dma_req_bits_status_tvm
  io.dma.req.bits.status.mxr := custom_load_controller.io.io_dma_req_bits_status_mxr
  io.dma.req.bits.status.sum := custom_load_controller.io.io_dma_req_bits_status_sum
  io.dma.req.bits.status.mprv := custom_load_controller.io.io_dma_req_bits_status_mprv
  io.dma.req.bits.status.xs := custom_load_controller.io.io_dma_req_bits_status_xs
  io.dma.req.bits.status.fs := custom_load_controller.io.io_dma_req_bits_status_fs
  io.dma.req.bits.status.mpp := custom_load_controller.io.io_dma_req_bits_status_mpp
  io.dma.req.bits.status.vs := custom_load_controller.io.io_dma_req_bits_status_vs
  io.dma.req.bits.status.spp := custom_load_controller.io.io_dma_req_bits_status_spp
  io.dma.req.bits.status.mpie := custom_load_controller.io.io_dma_req_bits_status_mpie
  io.dma.req.bits.status.ube := custom_load_controller.io.io_dma_req_bits_status_ube
  io.dma.req.bits.status.spie := custom_load_controller.io.io_dma_req_bits_status_spie
  io.dma.req.bits.status.upie := custom_load_controller.io.io_dma_req_bits_status_upie
  io.dma.req.bits.status.mie := custom_load_controller.io.io_dma_req_bits_status_mie
  io.dma.req.bits.status.hie := custom_load_controller.io.io_dma_req_bits_status_hie
  io.dma.req.bits.status.sie := custom_load_controller.io.io_dma_req_bits_status_sie
  io.dma.req.bits.status.uie := custom_load_controller.io.io_dma_req_bits_status_uie
  custom_load_controller.io.io_dma_resp_valid := io.dma.resp.valid
  custom_load_controller.io.io_dma_resp_bits_bytesRead := io.dma.resp.bits.bytesRead
  custom_load_controller.io.io_dma_resp_bits_cmd_id := io.dma.resp.bits.cmd_id

  custom_load_controller.io.io_completed_ready := io.completed.ready
  io.completed.valid := custom_load_controller.io.io_completed_valid
  io.completed.bits := custom_load_controller.io.io_completed_bits
}

// // TODO we need to check for WAW errors here
// // TODO deal with errors when reading scratchpad responses
// class LoadController[T <: Data, U <: Data, V <: Data](config: GemminiArrayConfig[T, U, V], coreMaxAddrBits: Int,
//                                                       local_addr_t: LocalAddr)
//                                (implicit p: Parameters) extends Module {
//   import config._

//   val io = IO(new Bundle {
//     val cmd = Flipped(Decoupled(new GemminiCmd(reservation_station_entries)))

//     val dma = new ScratchpadReadMemIO(local_addr_t, mvin_scale_t_bits)

//     val completed = Decoupled(UInt(log2Up(reservation_station_entries).W))

//     val busy = Output(Bool())

//     val counter = new CounterEventIO()
//   })

//   val waiting_for_command :: waiting_for_dma_req_ready :: sending_rows :: Nil = Enum(3)
//   val control_state = RegInit(waiting_for_command)

//   val strides = Reg(Vec(load_states, UInt(coreMaxAddrBits.W)))
//   val scales = Reg(Vec(load_states, UInt(mvin_scale_t_bits.W)))
//   val shrinks = Reg(Vec(load_states, Bool())) // Shrink inputs to accumulator
//   val block_strides = Reg(Vec(load_states, UInt(block_stride_bits.W))) // Spad stride during block move-ins
//   val pixel_repeats = Reg(Vec(load_states, UInt(pixel_repeats_bits.W)))
//   val block_rows = meshRows * tileRows
//   val block_cols = meshColumns * tileColumns
//   val row_counter = RegInit(0.U(log2Ceil(block_rows).W))

//   val cmd = Queue(io.cmd, ld_queue_length)

//   val vaddr = cmd.bits.cmd.rs1
//   val mvin_rs2 = cmd.bits.cmd.rs2.asTypeOf(new MvinRs2(mvin_rows_bits, mvin_cols_bits, local_addr_t))
//   val localaddr = mvin_rs2.local_addr
//   val cols = mvin_rs2.num_cols
//   val rows = mvin_rs2.num_rows

//   val config_stride = cmd.bits.cmd.rs2

//   val config_mvin_rs1 = cmd.bits.cmd.rs1.asTypeOf(new ConfigMvinRs1(mvin_scale_t_bits, block_stride_bits, pixel_repeats_bits))

//   val config_scale = config_mvin_rs1.scale
//   val config_shrink = config_mvin_rs1.shrink
//   val config_block_stride = config_mvin_rs1.stride
//   val config_pixel_repeats = config_mvin_rs1.pixel_repeats

//   val mstatus = cmd.bits.cmd.status

//   val load_state_id = MuxCase(0.U, Seq((cmd.bits.cmd.inst.funct === LOAD2_CMD) -> 1.U,
//     (cmd.bits.cmd.inst.funct === LOAD3_CMD) -> 2.U))
//   val config_state_id = config_mvin_rs1.state_id
//   val state_id = Mux(cmd.bits.cmd.inst.funct === CONFIG_CMD, config_state_id, load_state_id)

//   val stride = strides(state_id)
//   val scale = scales(state_id)
//   val shrink = shrinks(state_id)
//   val block_stride = block_strides(state_id)
//   val pixel_repeat = pixel_repeats(state_id)

//   val all_zeros = vaddr === 0.U

//   val localaddr_plus_row_counter = localaddr + row_counter

//   val actual_rows_read = Mux(stride === 0.U && !all_zeros, 1.U, rows)

//   val DoConfig = cmd.bits.cmd.inst.funct === CONFIG_CMD
//   val DoLoad = !DoConfig // TODO change this if more commands are added

//   cmd.ready := false.B

//   // Command tracker instantiation
//   val nCmds = (max_in_flight_mem_reqs / block_rows) + 1

//   val deps_t = new Bundle {
//     val rob_id = UInt(log2Up(reservation_station_entries).W)
//   }

//   val maxBytesInRowRequest = config.dma_maxbytes max (block_cols * config.inputType.getWidth / 8) max
//     (block_cols * config.accType.getWidth / 8)
//   val maxBytesInMatRequest = block_rows * maxBytesInRowRequest

//   val cmd_tracker = Module(new DMACommandTracker(nCmds, maxBytesInMatRequest, deps_t))

//   io.busy := cmd.valid || cmd_tracker.io.busy

//   // DMA IO wiring
//   io.dma.req.valid := (control_state === waiting_for_command && cmd.valid && DoLoad && cmd_tracker.io.alloc.ready) ||
//     control_state === waiting_for_dma_req_ready ||
//     (control_state === sending_rows && row_counter =/= 0.U)
//   io.dma.req.bits.vaddr := vaddr + row_counter * stride
//   io.dma.req.bits.laddr := localaddr_plus_row_counter
//   io.dma.req.bits.cols := cols
//   io.dma.req.bits.repeats := Mux(stride === 0.U && !all_zeros, rows - 1.U, 0.U)
//   io.dma.req.bits.block_stride := block_stride
//   io.dma.req.bits.scale := scale
//   io.dma.req.bits.has_acc_bitwidth := localaddr_plus_row_counter.is_acc_addr && !shrink
//   io.dma.req.bits.all_zeros := all_zeros
//   io.dma.req.bits.status := mstatus
//   io.dma.req.bits.pixel_repeats := pixel_repeat

//   // Command tracker IO
//   cmd_tracker.io.alloc.valid := control_state === waiting_for_command && cmd.valid && DoLoad
//   cmd_tracker.io.alloc.bits.bytes_to_read :=
//     Mux(io.dma.req.bits.has_acc_bitwidth, cols * actual_rows_read * config.accType.getWidth.U,
//       cols * actual_rows_read * config.inputType.getWidth.U) >> 3 // We replaced a very clear "/ 8.U" operation here with a ">> 3" operation, solely to satisfy Verilator's linter
//   cmd_tracker.io.alloc.bits.tag.rob_id := cmd.bits.rob_id.bits
//   cmd_tracker.io.request_returned.valid := io.dma.resp.fire // TODO use a bundle connect
//   cmd_tracker.io.request_returned.bits.cmd_id := io.dma.resp.bits.cmd_id // TODO use a bundle connect
//   cmd_tracker.io.request_returned.bits.bytes_read := io.dma.resp.bits.bytesRead
//   cmd_tracker.io.cmd_completed.ready := io.completed.ready

//   val cmd_id = RegEnableThru(cmd_tracker.io.alloc.bits.cmd_id, cmd_tracker.io.alloc.fire()) // TODO is this really better than a simple RegEnable?
//   io.dma.req.bits.cmd_id := cmd_id

//   io.completed.valid := cmd_tracker.io.cmd_completed.valid
//   io.completed.bits := cmd_tracker.io.cmd_completed.bits.tag.rob_id

//   io.busy := cmd.valid || cmd_tracker.io.busy

//   // Row counter
//   when (io.dma.req.fire) {
//     row_counter := wrappingAdd(row_counter, 1.U, actual_rows_read)

//     assert(block_stride >= rows)
//   }

//   // Control logic
//   switch (control_state) {
//     is (waiting_for_command) {
//       when (cmd.valid) {
//         when(DoConfig) {
//           stride := config_stride
//           scale := config_scale
//           shrink := config_shrink
//           block_stride := config_block_stride
//           pixel_repeat := Mux(config_pixel_repeats === 0.U, 1.U, config_pixel_repeats) // TODO this default value was just added to maintain backwards compatibility. we should deprecate and remove it later
//           cmd.ready := true.B
//         }

//         .elsewhen(DoLoad && cmd_tracker.io.alloc.fire()) {
//           control_state := Mux(io.dma.req.fire, sending_rows, waiting_for_dma_req_ready)
//         }
//       }
//     }

//     is (waiting_for_dma_req_ready) {
//       when (io.dma.req.fire) {
//         control_state := sending_rows
//       }
//     }

//     is (sending_rows) {
//       val last_row = row_counter === 0.U || (row_counter === actual_rows_read-1.U && io.dma.req.fire)

//       when (last_row) {
//         control_state := waiting_for_command
//         cmd.ready := true.B
//       }
//     }
//   }

//   // Optimizations based on config parameters
//   if (!has_first_layer_optimizations)
//     pixel_repeats.foreach(_ := 1.U)

//   // Performance counter
//   CounterEventIO.init(io.counter)
//   io.counter.connectEventSignal(CounterEvent.LOAD_ACTIVE_CYCLE, control_state === sending_rows)
//   io.counter.connectEventSignal(CounterEvent.LOAD_DMA_WAIT_CYCLE, control_state === waiting_for_dma_req_ready)
//   io.counter.connectEventSignal(CounterEvent.LOAD_SCRATCHPAD_WAIT_CYCLE, io.dma.req.valid && !io.dma.req.ready)

//   if (use_firesim_simulation_counters) {
//     PerfCounter(io.dma.req.valid && !io.dma.req.ready, "load_dma_wait_cycle", "cycles during which load controller is waiting for DMA to be available")
//   }

//   // Assertions
//   assert(!(cmd_tracker.io.alloc.fire() && cmd_tracker.io.alloc.bits.bytes_to_read === 0.U), "A single mvin instruction must load more than 0 bytes")
//   assert(has_first_layer_optimizations.B || !(cmd.valid && DoConfig && config_pixel_repeats > 1.U), "If first-layer optimizations are not enabled, then pixel-repeats cannot be greater than 1")
// }
