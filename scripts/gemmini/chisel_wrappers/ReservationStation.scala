
package gemmini

import chisel3._
import chisel3.util._
import freechips.rocketchip.tile.RoCCCommand
import freechips.rocketchip.util.PlusArg
import GemminiISA._
import Util._

import midas.targetutils.PerfCounter
import midas.targetutils.SynthesizePrintf

class ReservationStationBlackBoxAdapter extends BlackBox with HasBlackBoxResource {
  val io = IO(new Bundle {
    val clock = Input(Clock())
    val reset = Input(Reset())
    val io_alloc_valid = Input(Bool())
    val io_alloc_bits_cmd_inst_funct = Input(UInt(7.W))
    val io_alloc_bits_cmd_inst_rs2 = Input(UInt(5.W))
    val io_alloc_bits_cmd_inst_rs1 = Input(UInt(5.W))
    val io_alloc_bits_cmd_inst_xd = Input(Bool())
    val io_alloc_bits_cmd_inst_xs1 = Input(Bool())
    val io_alloc_bits_cmd_inst_xs2 = Input(Bool())
    val io_alloc_bits_cmd_inst_rd = Input(UInt(5.W))
    val io_alloc_bits_cmd_inst_opcode = Input(UInt(7.W))
    val io_alloc_bits_cmd_rs1 = Input(UInt(64.W))
    val io_alloc_bits_cmd_rs2 = Input(UInt(64.W))
    val io_alloc_bits_cmd_status_debug = Input(Bool())
    val io_alloc_bits_cmd_status_cease = Input(Bool())
    val io_alloc_bits_cmd_status_wfi = Input(Bool())
    val io_alloc_bits_cmd_status_isa = Input(UInt(32.W))
    val io_alloc_bits_cmd_status_dprv = Input(UInt(2.W))
    val io_alloc_bits_cmd_status_dv = Input(Bool())
    val io_alloc_bits_cmd_status_prv = Input(UInt(2.W))
    val io_alloc_bits_cmd_status_v = Input(Bool())
    val io_alloc_bits_cmd_status_sd = Input(Bool())
    val io_alloc_bits_cmd_status_zero2 = Input(UInt(23.W))
    val io_alloc_bits_cmd_status_mpv = Input(Bool())
    val io_alloc_bits_cmd_status_gva = Input(Bool())
    val io_alloc_bits_cmd_status_mbe = Input(Bool())
    val io_alloc_bits_cmd_status_sbe = Input(Bool())
    val io_alloc_bits_cmd_status_sxl = Input(UInt(2.W))
    val io_alloc_bits_cmd_status_uxl = Input(UInt(2.W))
    val io_alloc_bits_cmd_status_sd_rv32 = Input(Bool())
    val io_alloc_bits_cmd_status_zero1 = Input(UInt(8.W))
    val io_alloc_bits_cmd_status_tsr = Input(Bool())
    val io_alloc_bits_cmd_status_tw = Input(Bool())
    val io_alloc_bits_cmd_status_tvm = Input(Bool())
    val io_alloc_bits_cmd_status_mxr = Input(Bool())
    val io_alloc_bits_cmd_status_sum = Input(Bool())
    val io_alloc_bits_cmd_status_mprv = Input(Bool())
    val io_alloc_bits_cmd_status_xs = Input(UInt(2.W))
    val io_alloc_bits_cmd_status_fs = Input(UInt(2.W))
    val io_alloc_bits_cmd_status_mpp = Input(UInt(2.W))
    val io_alloc_bits_cmd_status_vs = Input(UInt(2.W))
    val io_alloc_bits_cmd_status_spp = Input(Bool())
    val io_alloc_bits_cmd_status_mpie = Input(Bool())
    val io_alloc_bits_cmd_status_ube = Input(Bool())
    val io_alloc_bits_cmd_status_spie = Input(Bool())
    val io_alloc_bits_cmd_status_upie = Input(Bool())
    val io_alloc_bits_cmd_status_mie = Input(Bool())
    val io_alloc_bits_cmd_status_hie = Input(Bool())
    val io_alloc_bits_cmd_status_sie = Input(Bool())
    val io_alloc_bits_cmd_status_uie = Input(Bool())
    val io_alloc_bits_from_matmul_fsm = Input(Bool())
    val io_alloc_bits_from_conv_fsm = Input(Bool())
    val io_completed_valid = Input(Bool())
    val io_completed_bits = Input(UInt(6.W))
    val io_issue_ld_ready = Input(Bool())
    val io_issue_st_ready = Input(Bool())
    val io_issue_ex_ready = Input(Bool())
    val io_alloc_ready = Output(Bool())
    val io_issue_ld_valid = Output(Bool())
    val io_issue_ld_cmd_cmd_inst_funct = Output(UInt(7.W))
    val io_issue_ld_cmd_cmd_inst_rs2 = Output(UInt(5.W))
    val io_issue_ld_cmd_cmd_inst_rs1 = Output(UInt(5.W))
    val io_issue_ld_cmd_cmd_inst_xd = Output(Bool())
    val io_issue_ld_cmd_cmd_inst_xs1 = Output(Bool())
    val io_issue_ld_cmd_cmd_inst_xs2 = Output(Bool())
    val io_issue_ld_cmd_cmd_inst_rd = Output(UInt(5.W))
    val io_issue_ld_cmd_cmd_inst_opcode = Output(UInt(7.W))
    val io_issue_ld_cmd_cmd_rs1 = Output(UInt(64.W))
    val io_issue_ld_cmd_cmd_rs2 = Output(UInt(64.W))
    val io_issue_ld_cmd_cmd_status_debug = Output(Bool())
    val io_issue_ld_cmd_cmd_status_cease = Output(Bool())
    val io_issue_ld_cmd_cmd_status_wfi = Output(Bool())
    val io_issue_ld_cmd_cmd_status_isa = Output(UInt(32.W))
    val io_issue_ld_cmd_cmd_status_dprv = Output(UInt(2.W))
    val io_issue_ld_cmd_cmd_status_dv = Output(Bool())
    val io_issue_ld_cmd_cmd_status_prv = Output(UInt(2.W))
    val io_issue_ld_cmd_cmd_status_v = Output(Bool())
    val io_issue_ld_cmd_cmd_status_sd = Output(Bool())
    val io_issue_ld_cmd_cmd_status_zero2 = Output(UInt(23.W))
    val io_issue_ld_cmd_cmd_status_mpv = Output(Bool())
    val io_issue_ld_cmd_cmd_status_gva = Output(Bool())
    val io_issue_ld_cmd_cmd_status_mbe = Output(Bool())
    val io_issue_ld_cmd_cmd_status_sbe = Output(Bool())
    val io_issue_ld_cmd_cmd_status_sxl = Output(UInt(2.W))
    val io_issue_ld_cmd_cmd_status_uxl = Output(UInt(2.W))
    val io_issue_ld_cmd_cmd_status_sd_rv32 = Output(Bool())
    val io_issue_ld_cmd_cmd_status_zero1 = Output(UInt(8.W))
    val io_issue_ld_cmd_cmd_status_tsr = Output(Bool())
    val io_issue_ld_cmd_cmd_status_tw = Output(Bool())
    val io_issue_ld_cmd_cmd_status_tvm = Output(Bool())
    val io_issue_ld_cmd_cmd_status_mxr = Output(Bool())
    val io_issue_ld_cmd_cmd_status_sum = Output(Bool())
    val io_issue_ld_cmd_cmd_status_mprv = Output(Bool())
    val io_issue_ld_cmd_cmd_status_xs = Output(UInt(2.W))
    val io_issue_ld_cmd_cmd_status_fs = Output(UInt(2.W))
    val io_issue_ld_cmd_cmd_status_mpp = Output(UInt(2.W))
    val io_issue_ld_cmd_cmd_status_vs = Output(UInt(2.W))
    val io_issue_ld_cmd_cmd_status_spp = Output(Bool())
    val io_issue_ld_cmd_cmd_status_mpie = Output(Bool())
    val io_issue_ld_cmd_cmd_status_ube = Output(Bool())
    val io_issue_ld_cmd_cmd_status_spie = Output(Bool())
    val io_issue_ld_cmd_cmd_status_upie = Output(Bool())
    val io_issue_ld_cmd_cmd_status_mie = Output(Bool())
    val io_issue_ld_cmd_cmd_status_hie = Output(Bool())
    val io_issue_ld_cmd_cmd_status_sie = Output(Bool())
    val io_issue_ld_cmd_cmd_status_uie = Output(Bool())
    val io_issue_ld_cmd_from_matmul_fsm = Output(Bool())
    val io_issue_ld_cmd_from_conv_fsm = Output(Bool())
    val io_issue_ld_rob_id = Output(UInt(6.W))
    val io_issue_st_valid = Output(Bool())
    val io_issue_st_cmd_cmd_inst_funct = Output(UInt(7.W))
    val io_issue_st_cmd_cmd_inst_rs2 = Output(UInt(5.W))
    val io_issue_st_cmd_cmd_inst_rs1 = Output(UInt(5.W))
    val io_issue_st_cmd_cmd_inst_xd = Output(Bool())
    val io_issue_st_cmd_cmd_inst_xs1 = Output(Bool())
    val io_issue_st_cmd_cmd_inst_xs2 = Output(Bool())
    val io_issue_st_cmd_cmd_inst_rd = Output(UInt(5.W))
    val io_issue_st_cmd_cmd_inst_opcode = Output(UInt(7.W))
    val io_issue_st_cmd_cmd_rs1 = Output(UInt(64.W))
    val io_issue_st_cmd_cmd_rs2 = Output(UInt(64.W))
    val io_issue_st_cmd_cmd_status_debug = Output(Bool())
    val io_issue_st_cmd_cmd_status_cease = Output(Bool())
    val io_issue_st_cmd_cmd_status_wfi = Output(Bool())
    val io_issue_st_cmd_cmd_status_isa = Output(UInt(32.W))
    val io_issue_st_cmd_cmd_status_dprv = Output(UInt(2.W))
    val io_issue_st_cmd_cmd_status_dv = Output(Bool())
    val io_issue_st_cmd_cmd_status_prv = Output(UInt(2.W))
    val io_issue_st_cmd_cmd_status_v = Output(Bool())
    val io_issue_st_cmd_cmd_status_sd = Output(Bool())
    val io_issue_st_cmd_cmd_status_zero2 = Output(UInt(23.W))
    val io_issue_st_cmd_cmd_status_mpv = Output(Bool())
    val io_issue_st_cmd_cmd_status_gva = Output(Bool())
    val io_issue_st_cmd_cmd_status_mbe = Output(Bool())
    val io_issue_st_cmd_cmd_status_sbe = Output(Bool())
    val io_issue_st_cmd_cmd_status_sxl = Output(UInt(2.W))
    val io_issue_st_cmd_cmd_status_uxl = Output(UInt(2.W))
    val io_issue_st_cmd_cmd_status_sd_rv32 = Output(Bool())
    val io_issue_st_cmd_cmd_status_zero1 = Output(UInt(8.W))
    val io_issue_st_cmd_cmd_status_tsr = Output(Bool())
    val io_issue_st_cmd_cmd_status_tw = Output(Bool())
    val io_issue_st_cmd_cmd_status_tvm = Output(Bool())
    val io_issue_st_cmd_cmd_status_mxr = Output(Bool())
    val io_issue_st_cmd_cmd_status_sum = Output(Bool())
    val io_issue_st_cmd_cmd_status_mprv = Output(Bool())
    val io_issue_st_cmd_cmd_status_xs = Output(UInt(2.W))
    val io_issue_st_cmd_cmd_status_fs = Output(UInt(2.W))
    val io_issue_st_cmd_cmd_status_mpp = Output(UInt(2.W))
    val io_issue_st_cmd_cmd_status_vs = Output(UInt(2.W))
    val io_issue_st_cmd_cmd_status_spp = Output(Bool())
    val io_issue_st_cmd_cmd_status_mpie = Output(Bool())
    val io_issue_st_cmd_cmd_status_ube = Output(Bool())
    val io_issue_st_cmd_cmd_status_spie = Output(Bool())
    val io_issue_st_cmd_cmd_status_upie = Output(Bool())
    val io_issue_st_cmd_cmd_status_mie = Output(Bool())
    val io_issue_st_cmd_cmd_status_hie = Output(Bool())
    val io_issue_st_cmd_cmd_status_sie = Output(Bool())
    val io_issue_st_cmd_cmd_status_uie = Output(Bool())
    val io_issue_st_cmd_from_matmul_fsm = Output(Bool())
    val io_issue_st_cmd_from_conv_fsm = Output(Bool())
    val io_issue_st_rob_id = Output(UInt(6.W))
    val io_issue_ex_valid = Output(Bool())
    val io_issue_ex_cmd_cmd_inst_funct = Output(UInt(7.W))
    val io_issue_ex_cmd_cmd_rs1 = Output(UInt(64.W))
    val io_issue_ex_cmd_cmd_rs2 = Output(UInt(64.W))
    val io_issue_ex_rob_id = Output(UInt(6.W))
    val io_conv_ld_completed = Output(UInt(2.W))
    val io_conv_ex_completed = Output(UInt(2.W))
    val io_conv_st_completed = Output(UInt(2.W))
    val io_matmul_ld_completed = Output(UInt(2.W))
    val io_matmul_ex_completed = Output(UInt(2.W))
    val io_matmul_st_completed = Output(UInt(2.W))
    val io_busy = Output(Bool())
  })
  addResource("/vsrc/ReservationStationBlackBox.v")
}

// TODO unify this class with GemminiCmdWithDeps
class ReservationStationIssue[T <: Data](cmd_t: T, id_width: Int) extends Bundle {
  val valid = Output(Bool())
  val ready = Input(Bool())
  val cmd = Output(cmd_t.cloneType)
  val rob_id = Output(UInt(id_width.W))

  def fire(dummy: Int=0) = valid && ready
}

// TODO we don't need to store the full command in here. We should be able to release the command directly into the relevant controller and only store the associated metadata in the ROB. This would reduce the size considerably
class ReservationStation[T <: Data : Arithmetic, U <: Data, V <: Data](config: GemminiArrayConfig[T, U, V],
                                                                       cmd_t: GemminiCmd) extends Module {
  import config._

  val block_rows = tileRows * meshRows
  val block_cols = tileColumns * meshColumns

  val max_instructions_completed_per_type_per_cycle = 2 // Every cycle, at most two instructions of a single "type" (ld/st/ex) can be completed: one through the io.completed port, and the other if it is a "complete-on-issue" instruction

  val io = IO(new Bundle {
    val alloc = Flipped(Decoupled(cmd_t.cloneType))

    val completed = Flipped(Valid(UInt(ROB_ID_WIDTH.W)))

    val issue = new Bundle {
      val ld = new ReservationStationIssue(cmd_t, ROB_ID_WIDTH)
      val st = new ReservationStationIssue(cmd_t, ROB_ID_WIDTH)
      val ex = new ReservationStationIssue(cmd_t, ROB_ID_WIDTH)
    }

    val conv_ld_completed = Output(UInt(log2Up(max_instructions_completed_per_type_per_cycle+1).W))
    val conv_ex_completed = Output(UInt(log2Up(max_instructions_completed_per_type_per_cycle+1).W))
    val conv_st_completed = Output(UInt(log2Up(max_instructions_completed_per_type_per_cycle+1).W))

    val matmul_ld_completed = Output(UInt(log2Up(max_instructions_completed_per_type_per_cycle+1).W))
    val matmul_ex_completed = Output(UInt(log2Up(max_instructions_completed_per_type_per_cycle+1).W))
    val matmul_st_completed = Output(UInt(log2Up(max_instructions_completed_per_type_per_cycle+1).W))

    val busy = Output(Bool())

    // val counter = new CounterEventIO()
  })

  val custom_reservation_station = Module(new ReservationStationBlackBoxAdapter)

  custom_reservation_station.io.clock := clock
  custom_reservation_station.io.reset := reset

  custom_reservation_station.io.io_alloc_valid := io.alloc.valid
  custom_reservation_station.io.io_alloc_bits_cmd_inst_funct := io.alloc.bits.cmd.inst.funct
  custom_reservation_station.io.io_alloc_bits_cmd_inst_rs2 := io.alloc.bits.cmd.inst.rs2
  custom_reservation_station.io.io_alloc_bits_cmd_inst_rs1 := io.alloc.bits.cmd.inst.rs1
  custom_reservation_station.io.io_alloc_bits_cmd_inst_xd := io.alloc.bits.cmd.inst.xd
  custom_reservation_station.io.io_alloc_bits_cmd_inst_xs1 := io.alloc.bits.cmd.inst.xs1
  custom_reservation_station.io.io_alloc_bits_cmd_inst_xs2 := io.alloc.bits.cmd.inst.xs2
  custom_reservation_station.io.io_alloc_bits_cmd_inst_rd := io.alloc.bits.cmd.inst.rd
  custom_reservation_station.io.io_alloc_bits_cmd_inst_opcode := io.alloc.bits.cmd.inst.opcode
  custom_reservation_station.io.io_alloc_bits_cmd_rs1 := io.alloc.bits.cmd.rs1
  custom_reservation_station.io.io_alloc_bits_cmd_rs2 := io.alloc.bits.cmd.rs2
  custom_reservation_station.io.io_alloc_bits_cmd_status_debug := io.alloc.bits.cmd.status.debug
  custom_reservation_station.io.io_alloc_bits_cmd_status_cease := io.alloc.bits.cmd.status.cease
  custom_reservation_station.io.io_alloc_bits_cmd_status_wfi := io.alloc.bits.cmd.status.wfi
  custom_reservation_station.io.io_alloc_bits_cmd_status_isa := io.alloc.bits.cmd.status.isa
  custom_reservation_station.io.io_alloc_bits_cmd_status_dprv := io.alloc.bits.cmd.status.dprv
  custom_reservation_station.io.io_alloc_bits_cmd_status_dv := io.alloc.bits.cmd.status.dv
  custom_reservation_station.io.io_alloc_bits_cmd_status_prv := io.alloc.bits.cmd.status.prv
  custom_reservation_station.io.io_alloc_bits_cmd_status_v := io.alloc.bits.cmd.status.v
  custom_reservation_station.io.io_alloc_bits_cmd_status_sd := io.alloc.bits.cmd.status.sd
  custom_reservation_station.io.io_alloc_bits_cmd_status_zero2 := io.alloc.bits.cmd.status.zero2
  custom_reservation_station.io.io_alloc_bits_cmd_status_mpv := io.alloc.bits.cmd.status.mpv
  custom_reservation_station.io.io_alloc_bits_cmd_status_gva := io.alloc.bits.cmd.status.gva
  custom_reservation_station.io.io_alloc_bits_cmd_status_mbe := io.alloc.bits.cmd.status.mbe
  custom_reservation_station.io.io_alloc_bits_cmd_status_sbe := io.alloc.bits.cmd.status.sbe
  custom_reservation_station.io.io_alloc_bits_cmd_status_sxl := io.alloc.bits.cmd.status.sxl
  custom_reservation_station.io.io_alloc_bits_cmd_status_uxl := io.alloc.bits.cmd.status.uxl
  custom_reservation_station.io.io_alloc_bits_cmd_status_sd_rv32 := io.alloc.bits.cmd.status.sd_rv32
  custom_reservation_station.io.io_alloc_bits_cmd_status_zero1 := io.alloc.bits.cmd.status.zero1
  custom_reservation_station.io.io_alloc_bits_cmd_status_tsr := io.alloc.bits.cmd.status.tsr
  custom_reservation_station.io.io_alloc_bits_cmd_status_tw := io.alloc.bits.cmd.status.tw
  custom_reservation_station.io.io_alloc_bits_cmd_status_tvm := io.alloc.bits.cmd.status.tvm
  custom_reservation_station.io.io_alloc_bits_cmd_status_mxr := io.alloc.bits.cmd.status.mxr
  custom_reservation_station.io.io_alloc_bits_cmd_status_sum := io.alloc.bits.cmd.status.sum
  custom_reservation_station.io.io_alloc_bits_cmd_status_mprv := io.alloc.bits.cmd.status.mprv
  custom_reservation_station.io.io_alloc_bits_cmd_status_xs := io.alloc.bits.cmd.status.xs
  custom_reservation_station.io.io_alloc_bits_cmd_status_fs := io.alloc.bits.cmd.status.fs
  custom_reservation_station.io.io_alloc_bits_cmd_status_mpp := io.alloc.bits.cmd.status.mpp
  custom_reservation_station.io.io_alloc_bits_cmd_status_vs := io.alloc.bits.cmd.status.vs
  custom_reservation_station.io.io_alloc_bits_cmd_status_spp := io.alloc.bits.cmd.status.spp
  custom_reservation_station.io.io_alloc_bits_cmd_status_mpie := io.alloc.bits.cmd.status.mpie
  custom_reservation_station.io.io_alloc_bits_cmd_status_ube := io.alloc.bits.cmd.status.ube
  custom_reservation_station.io.io_alloc_bits_cmd_status_spie := io.alloc.bits.cmd.status.spie
  custom_reservation_station.io.io_alloc_bits_cmd_status_upie := io.alloc.bits.cmd.status.upie
  custom_reservation_station.io.io_alloc_bits_cmd_status_mie := io.alloc.bits.cmd.status.mie
  custom_reservation_station.io.io_alloc_bits_cmd_status_hie := io.alloc.bits.cmd.status.hie
  custom_reservation_station.io.io_alloc_bits_cmd_status_sie := io.alloc.bits.cmd.status.sie
  custom_reservation_station.io.io_alloc_bits_cmd_status_uie := io.alloc.bits.cmd.status.uie
  custom_reservation_station.io.io_alloc_bits_from_matmul_fsm := io.alloc.bits.from_matmul_fsm
  custom_reservation_station.io.io_alloc_bits_from_conv_fsm := io.alloc.bits.from_conv_fsm
  io.alloc.ready := custom_reservation_station.io.io_alloc_ready

  custom_reservation_station.io.io_completed_valid := io.completed.valid
  custom_reservation_station.io.io_completed_bits := io.completed.bits

  custom_reservation_station.io.io_issue_ld_ready := io.issue.ld.ready
  io.issue.ld.valid := custom_reservation_station.io.io_issue_ld_valid
  io.issue.ld.cmd.cmd.inst.funct := custom_reservation_station.io.io_issue_ld_cmd_cmd_inst_funct
  io.issue.ld.cmd.cmd.inst.rs2 := custom_reservation_station.io.io_issue_ld_cmd_cmd_inst_rs2
  io.issue.ld.cmd.cmd.inst.rs1 := custom_reservation_station.io.io_issue_ld_cmd_cmd_inst_rs1
  io.issue.ld.cmd.cmd.inst.xd := custom_reservation_station.io.io_issue_ld_cmd_cmd_inst_xd
  io.issue.ld.cmd.cmd.inst.xs1 := custom_reservation_station.io.io_issue_ld_cmd_cmd_inst_xs1
  io.issue.ld.cmd.cmd.inst.xs2 := custom_reservation_station.io.io_issue_ld_cmd_cmd_inst_xs2
  io.issue.ld.cmd.cmd.inst.rd := custom_reservation_station.io.io_issue_ld_cmd_cmd_inst_rd
  io.issue.ld.cmd.cmd.inst.opcode := custom_reservation_station.io.io_issue_ld_cmd_cmd_inst_opcode
  io.issue.ld.cmd.cmd.rs1 := custom_reservation_station.io.io_issue_ld_cmd_cmd_rs1
  io.issue.ld.cmd.cmd.rs2 := custom_reservation_station.io.io_issue_ld_cmd_cmd_rs2
  io.issue.ld.cmd.cmd.status.debug := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_debug
  io.issue.ld.cmd.cmd.status.cease := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_cease
  io.issue.ld.cmd.cmd.status.wfi := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_wfi
  io.issue.ld.cmd.cmd.status.isa := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_isa
  io.issue.ld.cmd.cmd.status.dprv := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_dprv
  io.issue.ld.cmd.cmd.status.dv := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_dv
  io.issue.ld.cmd.cmd.status.prv := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_prv
  io.issue.ld.cmd.cmd.status.v := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_v
  io.issue.ld.cmd.cmd.status.sd := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_sd
  io.issue.ld.cmd.cmd.status.zero2 := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_zero2
  io.issue.ld.cmd.cmd.status.mpv := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_mpv
  io.issue.ld.cmd.cmd.status.gva := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_gva
  io.issue.ld.cmd.cmd.status.mbe := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_mbe
  io.issue.ld.cmd.cmd.status.sbe := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_sbe
  io.issue.ld.cmd.cmd.status.sxl := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_sxl
  io.issue.ld.cmd.cmd.status.uxl := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_uxl
  io.issue.ld.cmd.cmd.status.sd_rv32 := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_sd_rv32
  io.issue.ld.cmd.cmd.status.zero1 := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_zero1
  io.issue.ld.cmd.cmd.status.tsr := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_tsr
  io.issue.ld.cmd.cmd.status.tw := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_tw
  io.issue.ld.cmd.cmd.status.tvm := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_tvm
  io.issue.ld.cmd.cmd.status.mxr := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_mxr
  io.issue.ld.cmd.cmd.status.sum := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_sum
  io.issue.ld.cmd.cmd.status.mprv := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_mprv
  io.issue.ld.cmd.cmd.status.xs := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_xs
  io.issue.ld.cmd.cmd.status.fs := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_fs
  io.issue.ld.cmd.cmd.status.mpp := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_mpp
  io.issue.ld.cmd.cmd.status.vs := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_vs
  io.issue.ld.cmd.cmd.status.spp := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_spp
  io.issue.ld.cmd.cmd.status.mpie := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_mpie
  io.issue.ld.cmd.cmd.status.ube := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_ube
  io.issue.ld.cmd.cmd.status.spie := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_spie
  io.issue.ld.cmd.cmd.status.upie := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_upie
  io.issue.ld.cmd.cmd.status.mie := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_mie
  io.issue.ld.cmd.cmd.status.hie := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_hie
  io.issue.ld.cmd.cmd.status.sie := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_sie
  io.issue.ld.cmd.cmd.status.uie := custom_reservation_station.io.io_issue_ld_cmd_cmd_status_uie
  io.issue.ld.cmd.rob_id := DontCare
  io.issue.ld.cmd.from_matmul_fsm := custom_reservation_station.io.io_issue_ld_cmd_from_matmul_fsm
  io.issue.ld.cmd.from_conv_fsm := custom_reservation_station.io.io_issue_ld_cmd_from_conv_fsm
  io.issue.ld.rob_id := custom_reservation_station.io.io_issue_ld_rob_id

  custom_reservation_station.io.io_issue_st_ready := io.issue.st.ready
  io.issue.st.valid := custom_reservation_station.io.io_issue_st_valid
  io.issue.st.cmd.cmd.inst.funct := custom_reservation_station.io.io_issue_st_cmd_cmd_inst_funct
  io.issue.st.cmd.cmd.inst.rs2 := custom_reservation_station.io.io_issue_st_cmd_cmd_inst_rs2
  io.issue.st.cmd.cmd.inst.rs1 := custom_reservation_station.io.io_issue_st_cmd_cmd_inst_rs1
  io.issue.st.cmd.cmd.inst.xd := custom_reservation_station.io.io_issue_st_cmd_cmd_inst_xd
  io.issue.st.cmd.cmd.inst.xs1 := custom_reservation_station.io.io_issue_st_cmd_cmd_inst_xs1
  io.issue.st.cmd.cmd.inst.xs2 := custom_reservation_station.io.io_issue_st_cmd_cmd_inst_xs2
  io.issue.st.cmd.cmd.inst.rd := custom_reservation_station.io.io_issue_st_cmd_cmd_inst_rd
  io.issue.st.cmd.cmd.inst.opcode := custom_reservation_station.io.io_issue_st_cmd_cmd_inst_opcode
  io.issue.st.cmd.cmd.rs1 := custom_reservation_station.io.io_issue_st_cmd_cmd_rs1
  io.issue.st.cmd.cmd.rs2 := custom_reservation_station.io.io_issue_st_cmd_cmd_rs2
  io.issue.st.cmd.cmd.status.debug := custom_reservation_station.io.io_issue_st_cmd_cmd_status_debug
  io.issue.st.cmd.cmd.status.cease := custom_reservation_station.io.io_issue_st_cmd_cmd_status_cease
  io.issue.st.cmd.cmd.status.wfi := custom_reservation_station.io.io_issue_st_cmd_cmd_status_wfi
  io.issue.st.cmd.cmd.status.isa := custom_reservation_station.io.io_issue_st_cmd_cmd_status_isa
  io.issue.st.cmd.cmd.status.dprv := custom_reservation_station.io.io_issue_st_cmd_cmd_status_dprv
  io.issue.st.cmd.cmd.status.dv := custom_reservation_station.io.io_issue_st_cmd_cmd_status_dv
  io.issue.st.cmd.cmd.status.prv := custom_reservation_station.io.io_issue_st_cmd_cmd_status_prv
  io.issue.st.cmd.cmd.status.v := custom_reservation_station.io.io_issue_st_cmd_cmd_status_v
  io.issue.st.cmd.cmd.status.sd := custom_reservation_station.io.io_issue_st_cmd_cmd_status_sd
  io.issue.st.cmd.cmd.status.zero2 := custom_reservation_station.io.io_issue_st_cmd_cmd_status_zero2
  io.issue.st.cmd.cmd.status.mpv := custom_reservation_station.io.io_issue_st_cmd_cmd_status_mpv
  io.issue.st.cmd.cmd.status.gva := custom_reservation_station.io.io_issue_st_cmd_cmd_status_gva
  io.issue.st.cmd.cmd.status.mbe := custom_reservation_station.io.io_issue_st_cmd_cmd_status_mbe
  io.issue.st.cmd.cmd.status.sbe := custom_reservation_station.io.io_issue_st_cmd_cmd_status_sbe
  io.issue.st.cmd.cmd.status.sxl := custom_reservation_station.io.io_issue_st_cmd_cmd_status_sxl
  io.issue.st.cmd.cmd.status.uxl := custom_reservation_station.io.io_issue_st_cmd_cmd_status_uxl
  io.issue.st.cmd.cmd.status.sd_rv32 := custom_reservation_station.io.io_issue_st_cmd_cmd_status_sd_rv32
  io.issue.st.cmd.cmd.status.zero1 := custom_reservation_station.io.io_issue_st_cmd_cmd_status_zero1
  io.issue.st.cmd.cmd.status.tsr := custom_reservation_station.io.io_issue_st_cmd_cmd_status_tsr
  io.issue.st.cmd.cmd.status.tw := custom_reservation_station.io.io_issue_st_cmd_cmd_status_tw
  io.issue.st.cmd.cmd.status.tvm := custom_reservation_station.io.io_issue_st_cmd_cmd_status_tvm
  io.issue.st.cmd.cmd.status.mxr := custom_reservation_station.io.io_issue_st_cmd_cmd_status_mxr
  io.issue.st.cmd.cmd.status.sum := custom_reservation_station.io.io_issue_st_cmd_cmd_status_sum
  io.issue.st.cmd.cmd.status.mprv := custom_reservation_station.io.io_issue_st_cmd_cmd_status_mprv
  io.issue.st.cmd.cmd.status.xs := custom_reservation_station.io.io_issue_st_cmd_cmd_status_xs
  io.issue.st.cmd.cmd.status.fs := custom_reservation_station.io.io_issue_st_cmd_cmd_status_fs
  io.issue.st.cmd.cmd.status.mpp := custom_reservation_station.io.io_issue_st_cmd_cmd_status_mpp
  io.issue.st.cmd.cmd.status.vs := custom_reservation_station.io.io_issue_st_cmd_cmd_status_vs
  io.issue.st.cmd.cmd.status.spp := custom_reservation_station.io.io_issue_st_cmd_cmd_status_spp
  io.issue.st.cmd.cmd.status.mpie := custom_reservation_station.io.io_issue_st_cmd_cmd_status_mpie
  io.issue.st.cmd.cmd.status.ube := custom_reservation_station.io.io_issue_st_cmd_cmd_status_ube
  io.issue.st.cmd.cmd.status.spie := custom_reservation_station.io.io_issue_st_cmd_cmd_status_spie
  io.issue.st.cmd.cmd.status.upie := custom_reservation_station.io.io_issue_st_cmd_cmd_status_upie
  io.issue.st.cmd.cmd.status.mie := custom_reservation_station.io.io_issue_st_cmd_cmd_status_mie
  io.issue.st.cmd.cmd.status.hie := custom_reservation_station.io.io_issue_st_cmd_cmd_status_hie
  io.issue.st.cmd.cmd.status.sie := custom_reservation_station.io.io_issue_st_cmd_cmd_status_sie
  io.issue.st.cmd.cmd.status.uie := custom_reservation_station.io.io_issue_st_cmd_cmd_status_uie
  io.issue.st.cmd.rob_id := DontCare
  io.issue.st.cmd.from_matmul_fsm := custom_reservation_station.io.io_issue_st_cmd_from_matmul_fsm
  io.issue.st.cmd.from_conv_fsm := custom_reservation_station.io.io_issue_st_cmd_from_conv_fsm
  io.issue.st.rob_id := custom_reservation_station.io.io_issue_st_rob_id

  custom_reservation_station.io.io_issue_ex_ready := io.issue.ex.ready
  io.issue.ex.valid := custom_reservation_station.io.io_issue_ex_valid
  io.issue.ex.cmd := DontCare
  io.issue.ex.cmd.cmd.inst.funct := custom_reservation_station.io.io_issue_ex_cmd_cmd_inst_funct
  io.issue.ex.cmd.cmd.rs1 := custom_reservation_station.io.io_issue_ex_cmd_cmd_rs1
  io.issue.ex.cmd.cmd.rs2 := custom_reservation_station.io.io_issue_ex_cmd_cmd_rs2
  io.issue.ex.rob_id := custom_reservation_station.io.io_issue_ex_rob_id

  io.conv_ld_completed := custom_reservation_station.io.io_conv_ld_completed
  io.conv_ex_completed := custom_reservation_station.io.io_conv_ex_completed
  io.conv_st_completed := custom_reservation_station.io.io_conv_st_completed

  io.matmul_ld_completed := custom_reservation_station.io.io_matmul_ld_completed
  io.matmul_ex_completed := custom_reservation_station.io.io_matmul_ex_completed
  io.matmul_st_completed := custom_reservation_station.io.io_matmul_st_completed

  io.busy := custom_reservation_station.io.io_busy
}
