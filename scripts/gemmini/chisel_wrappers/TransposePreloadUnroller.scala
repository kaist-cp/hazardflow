package gemmini

import chisel3._
import chisel3.util._
import chisel3.experimental.ChiselEnum
import org.chipsalliance.cde.config.Parameters
import Util._
import midas.targetutils.PerfCounter

class TransposePreloadUnrollerBlackBoxAdapter[T <: Data, U <: Data, V <: Data](config: GemminiArrayConfig[T, U, V])
                                                                 (implicit p: Parameters) extends BlackBox with HasBlackBoxResource {
  val io = IO(new Bundle {
    val clock = Input(Clock())
    val reset = Input(Bool())

    val io_in_valid = Input(Bool())
    val io_out_ready = Input(Bool())
    
    val io_in_ready = Output(Bool())
    val io_out_valid = Output(Bool())

    val io_in_cmd_bits_cmd_inst_funct = Input(Bits(7.W))
    val io_in_cmd_bits_cmd_inst_rs2 = Input(Bits(5.W))
    val io_in_cmd_bits_cmd_inst_rs1 = Input(Bits(5.W))
    val io_in_cmd_bits_cmd_inst_xd = Input(Bool())
    val io_in_cmd_bits_cmd_inst_xs1 = Input(Bool())
    val io_in_cmd_bits_cmd_inst_xs2 = Input(Bool())
    val io_in_cmd_bits_cmd_inst_rd = Input(Bits(5.W))
    val io_in_cmd_bits_cmd_inst_opcode = Input(Bits(7.W))
    val io_in_cmd_bits_cmd_rs1 = Input(Bits(64.W)) // xLen = 64
    val io_in_cmd_bits_cmd_rs2 = Input(Bits(64.W)) // xLen = 64
    val io_in_cmd_bits_cmd_status_debug = Input(Bool())
    val io_in_cmd_bits_cmd_status_cease = Input(Bool())
    val io_in_cmd_bits_cmd_status_wfi = Input(Bool())
    val io_in_cmd_bits_cmd_status_isa = Input(UInt(32.W))
    val io_in_cmd_bits_cmd_status_dprv = Input(UInt(2.W)) // PRV.SZ = 2
    val io_in_cmd_bits_cmd_status_dv = Input(Bool())
    val io_in_cmd_bits_cmd_status_prv = Input(UInt(2.W)) // PRV.SZ = 2
    val io_in_cmd_bits_cmd_status_v = Input(Bool())
    val io_in_cmd_bits_cmd_status_sd = Input(Bool())
    val io_in_cmd_bits_cmd_status_zero2 = Input(UInt(23.W))
    val io_in_cmd_bits_cmd_status_mpv = Input(Bool())
    val io_in_cmd_bits_cmd_status_gva = Input(Bool())
    val io_in_cmd_bits_cmd_status_mbe = Input(Bool())
    val io_in_cmd_bits_cmd_status_sbe = Input(Bool())
    val io_in_cmd_bits_cmd_status_sxl = Input(UInt(2.W))
    val io_in_cmd_bits_cmd_status_uxl = Input(UInt(2.W))
    val io_in_cmd_bits_cmd_status_sd_rv32 = Input(Bool())
    val io_in_cmd_bits_cmd_status_zero1 = Input(UInt(8.W))
    val io_in_cmd_bits_cmd_status_tsr = Input(Bool())
    val io_in_cmd_bits_cmd_status_tw = Input(Bool())
    val io_in_cmd_bits_cmd_status_tvm = Input(Bool())
    val io_in_cmd_bits_cmd_status_mxr = Input(Bool())
    val io_in_cmd_bits_cmd_status_sum = Input(Bool())
    val io_in_cmd_bits_cmd_status_mprv = Input(Bool())
    val io_in_cmd_bits_cmd_status_xs = Input(UInt(2.W))
    val io_in_cmd_bits_cmd_status_fs = Input(UInt(2.W))
    val io_in_cmd_bits_cmd_status_mpp = Input(UInt(2.W))
    val io_in_cmd_bits_cmd_status_vs = Input(UInt(2.W))
    val io_in_cmd_bits_cmd_status_spp = Input(UInt(1.W))
    val io_in_cmd_bits_cmd_status_mpie = Input(Bool())
    val io_in_cmd_bits_cmd_status_ube = Input(Bool())
    val io_in_cmd_bits_cmd_status_spie = Input(Bool())
    val io_in_cmd_bits_cmd_status_upie = Input(Bool())
    val io_in_cmd_bits_cmd_status_mie = Input(Bool())
    val io_in_cmd_bits_cmd_status_hie = Input(Bool())
    val io_in_cmd_bits_cmd_status_sie = Input(Bool())
    val io_in_cmd_bits_cmd_status_uie = Input(Bool())
    val io_in_cmd_bits_rob_id_valid = Input(Bool())
    val io_in_cmd_bits_rob_id_bits = Input(UInt(log2Up(config.reservation_station_entries).W))
    val io_in_cmd_bits_from_matmul_fsm = Input(Bool())
    val io_in_cmd_bits_from_conv_fsm = Input(Bool())

    val io_out_cmd_bits_cmd_inst_funct = Output(Bits(7.W))
    val io_out_cmd_bits_cmd_inst_rs2 = Output(Bits(5.W))
    val io_out_cmd_bits_cmd_inst_rs1 = Output(Bits(5.W))
    val io_out_cmd_bits_cmd_inst_xd = Output(Bool())
    val io_out_cmd_bits_cmd_inst_xs1 = Output(Bool())
    val io_out_cmd_bits_cmd_inst_xs2 = Output(Bool())
    val io_out_cmd_bits_cmd_inst_rd = Output(Bits(5.W))
    val io_out_cmd_bits_cmd_inst_opcode = Output(Bits(7.W))
    val io_out_cmd_bits_cmd_rs1 = Output(Bits(64.W)) // xLen = 64
    val io_out_cmd_bits_cmd_rs2 = Output(Bits(64.W)) // xLen = 64
    val io_out_cmd_bits_cmd_status_debug = Output(Bool())
    val io_out_cmd_bits_cmd_status_cease = Output(Bool())
    val io_out_cmd_bits_cmd_status_wfi = Output(Bool())
    val io_out_cmd_bits_cmd_status_isa = Output(UInt(32.W))
    val io_out_cmd_bits_cmd_status_dprv = Output(UInt(2.W)) // PRV.SZ = 2
    val io_out_cmd_bits_cmd_status_dv = Output(Bool())
    val io_out_cmd_bits_cmd_status_prv = Output(UInt(2.W)) // PRV.SZ = 2
    val io_out_cmd_bits_cmd_status_v = Output(Bool())
    val io_out_cmd_bits_cmd_status_sd = Output(Bool())
    val io_out_cmd_bits_cmd_status_zero2 = Output(UInt(23.W))
    val io_out_cmd_bits_cmd_status_mpv = Output(Bool())
    val io_out_cmd_bits_cmd_status_gva = Output(Bool())
    val io_out_cmd_bits_cmd_status_mbe = Output(Bool())
    val io_out_cmd_bits_cmd_status_sbe = Output(Bool())
    val io_out_cmd_bits_cmd_status_sxl = Output(UInt(2.W))
    val io_out_cmd_bits_cmd_status_uxl = Output(UInt(2.W))
    val io_out_cmd_bits_cmd_status_sd_rv32 = Output(Bool())
    val io_out_cmd_bits_cmd_status_zero1 = Output(UInt(8.W))
    val io_out_cmd_bits_cmd_status_tsr = Output(Bool())
    val io_out_cmd_bits_cmd_status_tw = Output(Bool())
    val io_out_cmd_bits_cmd_status_tvm = Output(Bool())
    val io_out_cmd_bits_cmd_status_mxr = Output(Bool())
    val io_out_cmd_bits_cmd_status_sum = Output(Bool())
    val io_out_cmd_bits_cmd_status_mprv = Output(Bool())
    val io_out_cmd_bits_cmd_status_xs = Output(UInt(2.W))
    val io_out_cmd_bits_cmd_status_fs = Output(UInt(2.W))
    val io_out_cmd_bits_cmd_status_mpp = Output(UInt(2.W))
    val io_out_cmd_bits_cmd_status_vs = Output(UInt(2.W))
    val io_out_cmd_bits_cmd_status_spp = Output(UInt(1.W))
    val io_out_cmd_bits_cmd_status_mpie = Output(Bool())
    val io_out_cmd_bits_cmd_status_ube = Output(Bool())
    val io_out_cmd_bits_cmd_status_spie = Output(Bool())
    val io_out_cmd_bits_cmd_status_upie = Output(Bool())
    val io_out_cmd_bits_cmd_status_mie = Output(Bool())
    val io_out_cmd_bits_cmd_status_hie = Output(Bool())
    val io_out_cmd_bits_cmd_status_sie = Output(Bool())
    val io_out_cmd_bits_cmd_status_uie = Output(Bool())
    val io_out_cmd_bits_rob_id_valid = Output(Bool())
    val io_out_cmd_bits_rob_id_bits = Output(UInt(log2Up(config.reservation_station_entries).W))
    val io_out_cmd_bits_from_matmul_fsm = Output(Bool())
    val io_out_cmd_bits_from_conv_fsm = Output(Bool())
  })

  addResource("/vsrc/TransposePreloadUnrollerBlackBox.v")
}


class TransposePreloadUnroller[T <: Data, U <: Data, V <: Data](config: GemminiArrayConfig[T, U, V])
                                                                 (implicit p: Parameters) extends Module {
  import config._
  import GemminiISA._

  val io = IO(new Bundle {
    val in = Flipped(Decoupled(new GemminiCmd(config.reservation_station_entries)))
    val out = Decoupled(new GemminiCmd(config.reservation_station_entries))
  })

  val custom_tpu = Module(new TransposePreloadUnrollerBlackBoxAdapter(config))

  custom_tpu.io.clock := clock
  custom_tpu.io.reset := reset

  custom_tpu.io.io_in_valid := io.in.valid
  custom_tpu.io.io_out_ready := io.out.ready

  io.out.valid := custom_tpu.io.io_out_valid
  io.in.ready := custom_tpu.io.io_in_ready

  custom_tpu.io.io_in_cmd_bits_cmd_inst_funct := io.in.bits.cmd.inst.funct
  custom_tpu.io.io_in_cmd_bits_cmd_inst_rs2 := io.in.bits.cmd.inst.rs2
  custom_tpu.io.io_in_cmd_bits_cmd_inst_rs1 := io.in.bits.cmd.inst.rs1
  custom_tpu.io.io_in_cmd_bits_cmd_inst_xd := io.in.bits.cmd.inst.xd
  custom_tpu.io.io_in_cmd_bits_cmd_inst_xs1 := io.in.bits.cmd.inst.xs1
  custom_tpu.io.io_in_cmd_bits_cmd_inst_xs2 := io.in.bits.cmd.inst.xs2
  custom_tpu.io.io_in_cmd_bits_cmd_inst_rd := io.in.bits.cmd.inst.rd
  custom_tpu.io.io_in_cmd_bits_cmd_inst_opcode := io.in.bits.cmd.inst.opcode
  custom_tpu.io.io_in_cmd_bits_cmd_rs1 := io.in.bits.cmd.rs1
  custom_tpu.io.io_in_cmd_bits_cmd_rs2 := io.in.bits.cmd.rs2
  custom_tpu.io.io_in_cmd_bits_cmd_status_debug := io.in.bits.cmd.status.debug
  custom_tpu.io.io_in_cmd_bits_cmd_status_cease := io.in.bits.cmd.status.cease
  custom_tpu.io.io_in_cmd_bits_cmd_status_wfi := io.in.bits.cmd.status.wfi
  custom_tpu.io.io_in_cmd_bits_cmd_status_isa := io.in.bits.cmd.status.isa
  custom_tpu.io.io_in_cmd_bits_cmd_status_dprv := io.in.bits.cmd.status.dprv
  custom_tpu.io.io_in_cmd_bits_cmd_status_dv := io.in.bits.cmd.status.dv
  custom_tpu.io.io_in_cmd_bits_cmd_status_prv := io.in.bits.cmd.status.prv
  custom_tpu.io.io_in_cmd_bits_cmd_status_v := io.in.bits.cmd.status.v
  custom_tpu.io.io_in_cmd_bits_cmd_status_sd := io.in.bits.cmd.status.sd
  custom_tpu.io.io_in_cmd_bits_cmd_status_zero2 := io.in.bits.cmd.status.zero2
  custom_tpu.io.io_in_cmd_bits_cmd_status_mpv := io.in.bits.cmd.status.mpv
  custom_tpu.io.io_in_cmd_bits_cmd_status_gva := io.in.bits.cmd.status.gva
  custom_tpu.io.io_in_cmd_bits_cmd_status_mbe := io.in.bits.cmd.status.mbe
  custom_tpu.io.io_in_cmd_bits_cmd_status_sbe := io.in.bits.cmd.status.sbe
  custom_tpu.io.io_in_cmd_bits_cmd_status_sxl := io.in.bits.cmd.status.sxl
  custom_tpu.io.io_in_cmd_bits_cmd_status_uxl := io.in.bits.cmd.status.uxl
  custom_tpu.io.io_in_cmd_bits_cmd_status_sd_rv32 := io.in.bits.cmd.status.sd_rv32
  custom_tpu.io.io_in_cmd_bits_cmd_status_zero1 := io.in.bits.cmd.status.zero1
  custom_tpu.io.io_in_cmd_bits_cmd_status_tsr := io.in.bits.cmd.status.tsr
  custom_tpu.io.io_in_cmd_bits_cmd_status_tw := io.in.bits.cmd.status.tw
  custom_tpu.io.io_in_cmd_bits_cmd_status_tvm := io.in.bits.cmd.status.tvm
  custom_tpu.io.io_in_cmd_bits_cmd_status_mxr := io.in.bits.cmd.status.mxr
  custom_tpu.io.io_in_cmd_bits_cmd_status_sum := io.in.bits.cmd.status.sum
  custom_tpu.io.io_in_cmd_bits_cmd_status_mprv := io.in.bits.cmd.status.mprv
  custom_tpu.io.io_in_cmd_bits_cmd_status_xs := io.in.bits.cmd.status.xs
  custom_tpu.io.io_in_cmd_bits_cmd_status_fs := io.in.bits.cmd.status.fs
  custom_tpu.io.io_in_cmd_bits_cmd_status_mpp := io.in.bits.cmd.status.mpp
  custom_tpu.io.io_in_cmd_bits_cmd_status_vs := io.in.bits.cmd.status.vs
  custom_tpu.io.io_in_cmd_bits_cmd_status_spp := io.in.bits.cmd.status.spp
  custom_tpu.io.io_in_cmd_bits_cmd_status_mpie := io.in.bits.cmd.status.mpie
  custom_tpu.io.io_in_cmd_bits_cmd_status_ube := io.in.bits.cmd.status.ube
  custom_tpu.io.io_in_cmd_bits_cmd_status_spie := io.in.bits.cmd.status.spie
  custom_tpu.io.io_in_cmd_bits_cmd_status_upie := io.in.bits.cmd.status.upie
  custom_tpu.io.io_in_cmd_bits_cmd_status_mie := io.in.bits.cmd.status.mie
  custom_tpu.io.io_in_cmd_bits_cmd_status_hie := io.in.bits.cmd.status.hie
  custom_tpu.io.io_in_cmd_bits_cmd_status_sie := io.in.bits.cmd.status.sie
  custom_tpu.io.io_in_cmd_bits_cmd_status_uie := io.in.bits.cmd.status.uie
  custom_tpu.io.io_in_cmd_bits_rob_id_valid := io.in.bits.rob_id.valid
  custom_tpu.io.io_in_cmd_bits_rob_id_bits := io.in.bits.rob_id.bits
  custom_tpu.io.io_in_cmd_bits_from_matmul_fsm := io.in.bits.from_matmul_fsm
  custom_tpu.io.io_in_cmd_bits_from_conv_fsm := io.in.bits.from_conv_fsm


  io.out.bits.cmd.inst.funct := custom_tpu.io.io_out_cmd_bits_cmd_inst_funct
  io.out.bits.cmd.inst.rs2 := custom_tpu.io.io_out_cmd_bits_cmd_inst_rs2
  io.out.bits.cmd.inst.rs1 := custom_tpu.io.io_out_cmd_bits_cmd_inst_rs1
  io.out.bits.cmd.inst.xd := custom_tpu.io.io_out_cmd_bits_cmd_inst_xd
  io.out.bits.cmd.inst.xs1 := custom_tpu.io.io_out_cmd_bits_cmd_inst_xs1
  io.out.bits.cmd.inst.xs2 := custom_tpu.io.io_out_cmd_bits_cmd_inst_xs2
  io.out.bits.cmd.inst.rd := custom_tpu.io.io_out_cmd_bits_cmd_inst_rd
  io.out.bits.cmd.inst.opcode := custom_tpu.io.io_out_cmd_bits_cmd_inst_opcode
  io.out.bits.cmd.rs1 := custom_tpu.io.io_out_cmd_bits_cmd_rs1
  io.out.bits.cmd.rs2 := custom_tpu.io.io_out_cmd_bits_cmd_rs2
  io.out.bits.cmd.status.debug := custom_tpu.io.io_out_cmd_bits_cmd_status_debug
  io.out.bits.cmd.status.cease := custom_tpu.io.io_out_cmd_bits_cmd_status_cease
  io.out.bits.cmd.status.wfi := custom_tpu.io.io_out_cmd_bits_cmd_status_wfi
  io.out.bits.cmd.status.isa := custom_tpu.io.io_out_cmd_bits_cmd_status_isa
  io.out.bits.cmd.status.dprv := custom_tpu.io.io_out_cmd_bits_cmd_status_dprv
  io.out.bits.cmd.status.dv := custom_tpu.io.io_out_cmd_bits_cmd_status_dv
  io.out.bits.cmd.status.prv := custom_tpu.io.io_out_cmd_bits_cmd_status_prv
  io.out.bits.cmd.status.v := custom_tpu.io.io_out_cmd_bits_cmd_status_v
  io.out.bits.cmd.status.sd := custom_tpu.io.io_out_cmd_bits_cmd_status_sd
  io.out.bits.cmd.status.zero2 := custom_tpu.io.io_out_cmd_bits_cmd_status_zero2
  io.out.bits.cmd.status.mpv := custom_tpu.io.io_out_cmd_bits_cmd_status_mpv
  io.out.bits.cmd.status.gva := custom_tpu.io.io_out_cmd_bits_cmd_status_gva
  io.out.bits.cmd.status.mbe := custom_tpu.io.io_out_cmd_bits_cmd_status_mbe
  io.out.bits.cmd.status.sbe := custom_tpu.io.io_out_cmd_bits_cmd_status_sbe
  io.out.bits.cmd.status.sxl := custom_tpu.io.io_out_cmd_bits_cmd_status_sxl
  io.out.bits.cmd.status.uxl := custom_tpu.io.io_out_cmd_bits_cmd_status_uxl
  io.out.bits.cmd.status.sd_rv32 := custom_tpu.io.io_out_cmd_bits_cmd_status_sd_rv32
  io.out.bits.cmd.status.zero1 := custom_tpu.io.io_out_cmd_bits_cmd_status_zero1
  io.out.bits.cmd.status.tsr := custom_tpu.io.io_out_cmd_bits_cmd_status_tsr
  io.out.bits.cmd.status.tw := custom_tpu.io.io_out_cmd_bits_cmd_status_tw
  io.out.bits.cmd.status.tvm := custom_tpu.io.io_out_cmd_bits_cmd_status_tvm
  io.out.bits.cmd.status.mxr := custom_tpu.io.io_out_cmd_bits_cmd_status_mxr
  io.out.bits.cmd.status.sum := custom_tpu.io.io_out_cmd_bits_cmd_status_sum
  io.out.bits.cmd.status.mprv := custom_tpu.io.io_out_cmd_bits_cmd_status_mprv
  io.out.bits.cmd.status.xs := custom_tpu.io.io_out_cmd_bits_cmd_status_xs
  io.out.bits.cmd.status.fs := custom_tpu.io.io_out_cmd_bits_cmd_status_fs
  io.out.bits.cmd.status.mpp := custom_tpu.io.io_out_cmd_bits_cmd_status_mpp
  io.out.bits.cmd.status.vs := custom_tpu.io.io_out_cmd_bits_cmd_status_vs
  io.out.bits.cmd.status.spp := custom_tpu.io.io_out_cmd_bits_cmd_status_spp
  io.out.bits.cmd.status.mpie := custom_tpu.io.io_out_cmd_bits_cmd_status_mpie
  io.out.bits.cmd.status.ube := custom_tpu.io.io_out_cmd_bits_cmd_status_ube
  io.out.bits.cmd.status.spie := custom_tpu.io.io_out_cmd_bits_cmd_status_spie
  io.out.bits.cmd.status.upie := custom_tpu.io.io_out_cmd_bits_cmd_status_upie
  io.out.bits.cmd.status.mie := custom_tpu.io.io_out_cmd_bits_cmd_status_mie
  io.out.bits.cmd.status.hie := custom_tpu.io.io_out_cmd_bits_cmd_status_hie
  io.out.bits.cmd.status.sie := custom_tpu.io.io_out_cmd_bits_cmd_status_sie
  io.out.bits.cmd.status.uie := custom_tpu.io.io_out_cmd_bits_cmd_status_uie
  io.out.bits.rob_id.valid := custom_tpu.io.io_out_cmd_bits_rob_id_valid
  io.out.bits.rob_id.bits := custom_tpu.io.io_out_cmd_bits_rob_id_bits
  io.out.bits.from_matmul_fsm := custom_tpu.io.io_out_cmd_bits_from_matmul_fsm
  io.out.bits.from_conv_fsm := custom_tpu.io.io_out_cmd_bits_from_conv_fsm

}

object TransposePreloadUnroller {
  def apply[T <: Data, U <: Data, V <: Data](in: ReadyValidIO[GemminiCmd], config: GemminiArrayConfig[T, U, V])(implicit p: Parameters): DecoupledIO[GemminiCmd] = {
    val mod = Module(new TransposePreloadUnroller(config))
    mod.io.in <> in
    mod.io.out
  }
}