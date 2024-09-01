
package gemmini

import chisel3._
import chisel3.util._
import GemminiISA._
import Util._
import org.chipsalliance.cde.config.Parameters
import midas.targetutils.PerfCounter

class ExecuteControllerBlackBoxAdapter[T <: Data, U <: Data, V <: Data](xLen: Int, tagWidth: Int, config: GemminiArrayConfig[T, U, V])
                                  (implicit p: Parameters, ev: Arithmetic[T]) extends BlackBox with HasBlackBoxResource {

    val io = IO(new Bundle {
        val clock = Input(Clock())
        val reset = Input(Reset())

        val io_cmd = Flipped(Decoupled(new GemminiCmd(config.reservation_station_entries)))

        val io_srams = new Bundle {
            val read = Vec(config.sp_banks, new ScratchpadReadIO(config.sp_bank_entries, config.sp_width))
            val write = Vec(config.sp_banks, new ScratchpadWriteIO(config.sp_bank_entries, config.sp_width, (config.sp_width / (config.aligned_to * 8)) max 1))
        }

        val io_acc = new Bundle {
            val read_req = Vec(config.acc_banks, Decoupled(new AccumulatorReadReq(
                config.acc_bank_entries, config.accType, config.acc_scale_t
            )))
            val read_resp = Vec(config.acc_banks, Flipped(Decoupled(new AccumulatorScaleResp(
                Vec(config.meshColumns, Vec(config.tileColumns, config.inputType)),
                Vec(config.meshColumns, Vec(config.tileColumns, config.accType))
            ))))
            val write = Vec(config.acc_banks, Decoupled(new AccumulatorWriteReq(config.acc_bank_entries, Vec(config.meshColumns, Vec(config.tileColumns, config.accType)))))
        }

        val io_completed = Valid(UInt(log2Up(config.reservation_station_entries).W))
        // val busy = Output(Bool())   // TODO
    })

    addResource("/vsrc/ExecuteControllerBlackBox.v")
}

// TODO do we still need to flush when the dataflow is weight stationary? Won't the result just keep travelling through on its own?
class ExecuteController[T <: Data, U <: Data, V <: Data](xLen: Int, tagWidth: Int, config: GemminiArrayConfig[T, U, V])
                                  (implicit p: Parameters, ev: Arithmetic[T]) extends Module {
  import config._
  import ev._

  val io = IO(new Bundle {
    val cmd = Flipped(Decoupled(new GemminiCmd(reservation_station_entries)))

    val srams = new Bundle {
      val read = Vec(sp_banks, new ScratchpadReadIO(sp_bank_entries, sp_width))
      val write = Vec(sp_banks, new ScratchpadWriteIO(sp_bank_entries, sp_width, (sp_width / (aligned_to * 8)) max 1))
    }

    val acc = new Bundle {
      val read_req = Vec(acc_banks, Decoupled(new AccumulatorReadReq(
          acc_bank_entries, accType, acc_scale_t
      )))

      val read_resp = Flipped(Vec(acc_banks, Decoupled(new AccumulatorScaleResp(
        Vec(meshColumns, Vec(tileColumns, inputType)),
        Vec(meshColumns, Vec(tileColumns, accType))
      ))))

      // val write = Vec(acc_banks, new AccumulatorWriteIO(acc_bank_entries, Vec(meshColumns, Vec(tileColumns, accType))))
      val write = Vec(acc_banks, Decoupled(new AccumulatorWriteReq(acc_bank_entries, Vec(meshColumns, Vec(tileColumns, accType)))))
    }

    val completed = Valid(UInt(log2Up(reservation_station_entries).W))
    val busy = Output(Bool())
  })

    val custom_execute_controller = Module(new ExecuteControllerBlackBoxAdapter(xLen, tagWidth, config))

    custom_execute_controller.io.clock := clock
    custom_execute_controller.io.reset := reset

    custom_execute_controller.io.io_cmd <> io.cmd

    custom_execute_controller.io.io_srams.read <> io.srams.read
    custom_execute_controller.io.io_srams.write <> io.srams.write

    custom_execute_controller.io.io_acc.read_req <> io.acc.read_req
    custom_execute_controller.io.io_acc.read_resp <> io.acc.read_resp
    custom_execute_controller.io.io_acc.write <> io.acc.write

    custom_execute_controller.io.io_completed <> io.completed
    io.busy := DontCare // TODO
}
