
package gemmini

import chisel3._
import chisel3.util._
import chisel3.experimental._
import GemminiISA._
import Util._
import org.chipsalliance.cde.config.Parameters
import midas.targetutils.PerfCounter

class StoreControllerBlackBoxAdapter[T <: Data : Arithmetic, U <: Data, V <: Data](config: GemminiArrayConfig[T, U, V],
                                                                                   coreMaxAddrBits: Int, local_addr_t: LocalAddr) 
  extends BlackBox(Map("LOG_2_UP_RESERVATION_STATION_ENTRIES" -> log2Up(config.reservation_station_entries),
                       "ACC_SCALE_T_BITS" -> config.acc_scale_t_bits,
                       "ACC_TYPE_GET_WIDTH" -> config.accType.getWidth))
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
  })

  addResource("/vsrc/StoreControllerBlackBox.v")
}

class StoreController[T <: Data : Arithmetic, U <: Data, V <: Data](config: GemminiArrayConfig[T, U, V],
                                                                    coreMaxAddrBits: Int, local_addr_t: LocalAddr)(implicit p: Parameters) extends Module {
  import config._

  val io = IO(new Bundle {
    val cmd = Flipped(Decoupled(new GemminiCmd(reservation_station_entries)))

    val dma = new ScratchpadWriteMemIO(local_addr_t, accType.getWidth, acc_scale_t_bits)

    val completed = Decoupled(UInt(log2Up(reservation_station_entries).W))

    // val busy = Output(Bool())

    // val counter = new CounterEventIO()
  })

  val custom_store_controller = Module(new StoreControllerBlackBoxAdapter(config, coreMaxAddrBits, local_addr_t))

  custom_store_controller.io.clock := clock
  custom_store_controller.io.reset := reset

  io.cmd.ready := custom_store_controller.io.io_cmd_ready
  custom_store_controller.io.io_cmd_valid := io.cmd.valid
  custom_store_controller.io.io_cmd_bits_cmd_inst_funct := io.cmd.bits.cmd.inst.funct
  custom_store_controller.io.io_cmd_bits_cmd_inst_rs2 := io.cmd.bits.cmd.inst.rs2
  custom_store_controller.io.io_cmd_bits_cmd_inst_rs1 := io.cmd.bits.cmd.inst.rs1
  custom_store_controller.io.io_cmd_bits_cmd_inst_xd := io.cmd.bits.cmd.inst.xd
  custom_store_controller.io.io_cmd_bits_cmd_inst_xs1 := io.cmd.bits.cmd.inst.xs1
  custom_store_controller.io.io_cmd_bits_cmd_inst_xs2 := io.cmd.bits.cmd.inst.xs2
  custom_store_controller.io.io_cmd_bits_cmd_inst_rd := io.cmd.bits.cmd.inst.rd
  custom_store_controller.io.io_cmd_bits_cmd_inst_opcode := io.cmd.bits.cmd.inst.opcode
  custom_store_controller.io.io_cmd_bits_cmd_rs1 := io.cmd.bits.cmd.rs1
  custom_store_controller.io.io_cmd_bits_cmd_rs2 := io.cmd.bits.cmd.rs2
  custom_store_controller.io.io_cmd_bits_cmd_status_debug := io.cmd.bits.cmd.status.debug
  custom_store_controller.io.io_cmd_bits_cmd_status_cease := io.cmd.bits.cmd.status.cease
  custom_store_controller.io.io_cmd_bits_cmd_status_wfi := io.cmd.bits.cmd.status.wfi
  custom_store_controller.io.io_cmd_bits_cmd_status_isa := io.cmd.bits.cmd.status.isa
  custom_store_controller.io.io_cmd_bits_cmd_status_dprv := io.cmd.bits.cmd.status.dprv
  custom_store_controller.io.io_cmd_bits_cmd_status_dv := io.cmd.bits.cmd.status.dv
  custom_store_controller.io.io_cmd_bits_cmd_status_prv := io.cmd.bits.cmd.status.prv
  custom_store_controller.io.io_cmd_bits_cmd_status_v := io.cmd.bits.cmd.status.v
  custom_store_controller.io.io_cmd_bits_cmd_status_sd := io.cmd.bits.cmd.status.sd
  custom_store_controller.io.io_cmd_bits_cmd_status_zero2 := io.cmd.bits.cmd.status.zero2
  custom_store_controller.io.io_cmd_bits_cmd_status_mpv := io.cmd.bits.cmd.status.mpv
  custom_store_controller.io.io_cmd_bits_cmd_status_gva := io.cmd.bits.cmd.status.gva
  custom_store_controller.io.io_cmd_bits_cmd_status_mbe := io.cmd.bits.cmd.status.mbe
  custom_store_controller.io.io_cmd_bits_cmd_status_sbe := io.cmd.bits.cmd.status.sbe
  custom_store_controller.io.io_cmd_bits_cmd_status_sxl := io.cmd.bits.cmd.status.sxl
  custom_store_controller.io.io_cmd_bits_cmd_status_uxl := io.cmd.bits.cmd.status.uxl
  custom_store_controller.io.io_cmd_bits_cmd_status_sd_rv32 := io.cmd.bits.cmd.status.sd_rv32
  custom_store_controller.io.io_cmd_bits_cmd_status_zero1 := io.cmd.bits.cmd.status.zero1
  custom_store_controller.io.io_cmd_bits_cmd_status_tsr := io.cmd.bits.cmd.status.tsr
  custom_store_controller.io.io_cmd_bits_cmd_status_tw := io.cmd.bits.cmd.status.tw
  custom_store_controller.io.io_cmd_bits_cmd_status_tvm := io.cmd.bits.cmd.status.tvm
  custom_store_controller.io.io_cmd_bits_cmd_status_mxr := io.cmd.bits.cmd.status.mxr
  custom_store_controller.io.io_cmd_bits_cmd_status_sum := io.cmd.bits.cmd.status.sum
  custom_store_controller.io.io_cmd_bits_cmd_status_mprv := io.cmd.bits.cmd.status.mprv
  custom_store_controller.io.io_cmd_bits_cmd_status_xs := io.cmd.bits.cmd.status.xs
  custom_store_controller.io.io_cmd_bits_cmd_status_fs := io.cmd.bits.cmd.status.fs
  custom_store_controller.io.io_cmd_bits_cmd_status_mpp := io.cmd.bits.cmd.status.mpp
  custom_store_controller.io.io_cmd_bits_cmd_status_vs := io.cmd.bits.cmd.status.vs
  custom_store_controller.io.io_cmd_bits_cmd_status_spp := io.cmd.bits.cmd.status.spp
  custom_store_controller.io.io_cmd_bits_cmd_status_mpie := io.cmd.bits.cmd.status.mpie
  custom_store_controller.io.io_cmd_bits_cmd_status_ube := io.cmd.bits.cmd.status.ube
  custom_store_controller.io.io_cmd_bits_cmd_status_spie := io.cmd.bits.cmd.status.spie
  custom_store_controller.io.io_cmd_bits_cmd_status_upie := io.cmd.bits.cmd.status.upie
  custom_store_controller.io.io_cmd_bits_cmd_status_mie := io.cmd.bits.cmd.status.mie
  custom_store_controller.io.io_cmd_bits_cmd_status_hie := io.cmd.bits.cmd.status.hie
  custom_store_controller.io.io_cmd_bits_cmd_status_sie := io.cmd.bits.cmd.status.sie
  custom_store_controller.io.io_cmd_bits_cmd_status_uie := io.cmd.bits.cmd.status.uie
  custom_store_controller.io.io_cmd_bits_rob_id_valid := io.cmd.bits.rob_id.valid
  custom_store_controller.io.io_cmd_bits_rob_id_bits := io.cmd.bits.rob_id.bits
  custom_store_controller.io.io_cmd_bits_from_matmul_fsm := io.cmd.bits.from_matmul_fsm
  custom_store_controller.io.io_cmd_bits_from_conv_fsm := io.cmd.bits.from_conv_fsm

  custom_store_controller.io.io_dma_req_ready := io.dma.req.ready
  io.dma.req.valid := custom_store_controller.io.io_dma_req_valid
  io.dma.req.bits.vaddr := custom_store_controller.io.io_dma_req_bits_vaddr
  io.dma.req.bits.laddr.is_acc_addr := custom_store_controller.io.io_dma_req_bits_laddr_is_acc_addr
  io.dma.req.bits.laddr.accumulate := custom_store_controller.io.io_dma_req_bits_laddr_accumulate
  io.dma.req.bits.laddr.read_full_acc_row := custom_store_controller.io.io_dma_req_bits_laddr_read_full_acc_row
  io.dma.req.bits.laddr.norm_cmd := custom_store_controller.io.io_dma_req_bits_laddr_norm_cmd
  io.dma.req.bits.laddr.garbage := custom_store_controller.io.io_dma_req_bits_laddr_garbage
  io.dma.req.bits.laddr.garbage_bit := custom_store_controller.io.io_dma_req_bits_laddr_garbage_bit
  io.dma.req.bits.laddr.data := custom_store_controller.io.io_dma_req_bits_laddr_data
  io.dma.req.bits.acc_act := custom_store_controller.io.io_dma_req_bits_acc_act
  io.dma.req.bits.acc_scale := custom_store_controller.io.io_dma_req_bits_acc_scale
  io.dma.req.bits.acc_igelu_qb := custom_store_controller.io.io_dma_req_bits_acc_igelu_qb
  io.dma.req.bits.acc_igelu_qc := custom_store_controller.io.io_dma_req_bits_acc_igelu_qc
  io.dma.req.bits.acc_iexp_qln2 := custom_store_controller.io.io_dma_req_bits_acc_iexp_qln2
  io.dma.req.bits.acc_iexp_qln2_inv := custom_store_controller.io.io_dma_req_bits_acc_iexp_qln2_inv
  io.dma.req.bits.acc_norm_stats_id := custom_store_controller.io.io_dma_req_bits_acc_norm_stats_id
  io.dma.req.bits.len := custom_store_controller.io.io_dma_req_bits_len
  io.dma.req.bits.block := custom_store_controller.io.io_dma_req_bits_block
  io.dma.req.bits.cmd_id := custom_store_controller.io.io_dma_req_bits_cmd_id
  io.dma.req.bits.status.debug := custom_store_controller.io.io_dma_req_bits_status_debug
  io.dma.req.bits.status.cease := custom_store_controller.io.io_dma_req_bits_status_cease
  io.dma.req.bits.status.wfi := custom_store_controller.io.io_dma_req_bits_status_wfi
  io.dma.req.bits.status.isa := custom_store_controller.io.io_dma_req_bits_status_isa
  io.dma.req.bits.status.dprv := custom_store_controller.io.io_dma_req_bits_status_dprv
  io.dma.req.bits.status.dv := custom_store_controller.io.io_dma_req_bits_status_dv
  io.dma.req.bits.status.prv := custom_store_controller.io.io_dma_req_bits_status_prv
  io.dma.req.bits.status.v := custom_store_controller.io.io_dma_req_bits_status_v
  io.dma.req.bits.status.sd := custom_store_controller.io.io_dma_req_bits_status_sd
  io.dma.req.bits.status.zero2 := custom_store_controller.io.io_dma_req_bits_status_zero2
  io.dma.req.bits.status.mpv := custom_store_controller.io.io_dma_req_bits_status_mpv
  io.dma.req.bits.status.gva := custom_store_controller.io.io_dma_req_bits_status_gva
  io.dma.req.bits.status.mbe := custom_store_controller.io.io_dma_req_bits_status_mbe
  io.dma.req.bits.status.sbe := custom_store_controller.io.io_dma_req_bits_status_sbe
  io.dma.req.bits.status.sxl := custom_store_controller.io.io_dma_req_bits_status_sxl
  io.dma.req.bits.status.uxl := custom_store_controller.io.io_dma_req_bits_status_uxl
  io.dma.req.bits.status.sd_rv32 := custom_store_controller.io.io_dma_req_bits_status_sd_rv32
  io.dma.req.bits.status.zero1 := custom_store_controller.io.io_dma_req_bits_status_zero1
  io.dma.req.bits.status.tsr := custom_store_controller.io.io_dma_req_bits_status_tsr
  io.dma.req.bits.status.tw := custom_store_controller.io.io_dma_req_bits_status_tw
  io.dma.req.bits.status.tvm := custom_store_controller.io.io_dma_req_bits_status_tvm
  io.dma.req.bits.status.mxr := custom_store_controller.io.io_dma_req_bits_status_mxr
  io.dma.req.bits.status.sum := custom_store_controller.io.io_dma_req_bits_status_sum
  io.dma.req.bits.status.mprv := custom_store_controller.io.io_dma_req_bits_status_mprv
  io.dma.req.bits.status.xs := custom_store_controller.io.io_dma_req_bits_status_xs
  io.dma.req.bits.status.fs := custom_store_controller.io.io_dma_req_bits_status_fs
  io.dma.req.bits.status.mpp := custom_store_controller.io.io_dma_req_bits_status_mpp
  io.dma.req.bits.status.vs := custom_store_controller.io.io_dma_req_bits_status_vs
  io.dma.req.bits.status.spp := custom_store_controller.io.io_dma_req_bits_status_spp
  io.dma.req.bits.status.mpie := custom_store_controller.io.io_dma_req_bits_status_mpie
  io.dma.req.bits.status.ube := custom_store_controller.io.io_dma_req_bits_status_ube
  io.dma.req.bits.status.spie := custom_store_controller.io.io_dma_req_bits_status_spie
  io.dma.req.bits.status.upie := custom_store_controller.io.io_dma_req_bits_status_upie
  io.dma.req.bits.status.mie := custom_store_controller.io.io_dma_req_bits_status_mie
  io.dma.req.bits.status.hie := custom_store_controller.io.io_dma_req_bits_status_hie
  io.dma.req.bits.status.sie := custom_store_controller.io.io_dma_req_bits_status_sie
  io.dma.req.bits.status.uie := custom_store_controller.io.io_dma_req_bits_status_uie
  io.dma.req.bits.pool_en := custom_store_controller.io.io_dma_req_bits_pool_en
  io.dma.req.bits.store_en := custom_store_controller.io.io_dma_req_bits_store_en
  custom_store_controller.io.io_dma_resp_valid := io.dma.resp.valid
  custom_store_controller.io.io_dma_resp_bits_cmd_id := io.dma.resp.bits.cmd_id

  custom_store_controller.io.io_completed_ready := io.completed.ready
  io.completed.valid := custom_store_controller.io.io_completed_valid
  io.completed.bits := custom_store_controller.io.io_completed_bits
}
