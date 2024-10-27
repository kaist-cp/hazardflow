// See README.md for license details.
package gemmini

import chisel3._
import chisel3.util._
import chisel3.experimental._

class PEControl[T <: Data : Arithmetic](accType: T) extends Bundle {
  val dataflow = UInt(1.W) // TODO make this an Enum
  val propagate = UInt(1.W) // Which register should be propagated (and which should be accumulated)?
  val shift = UInt(log2Up(accType.getWidth).W) // TODO this isn't correct for Floats

}

class MacUnit[T <: Data](inputType: T, cType: T, dType: T) (implicit ev: Arithmetic[T]) extends Module {
  import ev._
  val io = IO(new Bundle {
    val in_a  = Input(inputType)
    val in_b  = Input(inputType)
    val in_c  = Input(cType)
    val out_d = Output(dType)
  })

  io.out_d := io.in_c.mac(io.in_a, io.in_b)
}

class PEBlackBoxAdapter[T <: Data](inputType: T, outputType: T, accType: T, df: Dataflow.Value, max_simultaneous_matmuls: Int)
                  extends BlackBox with HasBlackBoxResource { // Debugging variables
  val io = IO(new Bundle {
    val clock = Input(Clock())

    val io_in_a = Input(inputType)
    val io_in_b = Input(outputType)
    val io_in_d = Input(outputType)

    val io_in_control_dataflow = Input(UInt(1.W))
    val io_in_control_propagate = Input(UInt(1.W))
    val io_in_control_shift = Input(UInt(5.W))
    val io_in_id = Input(UInt(log2Up(max_simultaneous_matmuls).W))
    val io_in_last = Input(Bool())
    val io_in_valid = Input(Bool())

    val io_out_a = Output(inputType)
    val io_out_b = Output(outputType)
    val io_out_c = Output(outputType)

    val io_out_control_dataflow = Output(UInt(1.W))
    val io_out_control_propagate = Output(UInt(1.W))
    val io_out_control_shift = Output(UInt(5.W))
    val io_out_id = Output(UInt(log2Up(max_simultaneous_matmuls).W))
    val io_out_last = Output(Bool())
    val io_out_valid = Output(Bool())
    val io_bad_dataflow = Output(Bool())
  })
  addResource("/vsrc/PEBlackBox.v")
}

class PE[T <: Data](inputType: T, outputType: T, accType: T, df: Dataflow.Value, max_simultaneous_matmuls: Int)
                   (implicit ev: Arithmetic[T]) extends Module { // Debugging variables
  import ev._

  val io = IO(new Bundle {
    val in_a = Input(inputType)
    val in_b = Input(outputType)
    val in_d = Input(outputType)
    val out_a = Output(inputType)
    val out_b = Output(outputType)
    val out_c = Output(outputType)

    val in_control = Input(new PEControl(accType))
    val out_control = Output(new PEControl(accType))

    val in_id = Input(UInt(log2Up(max_simultaneous_matmuls).W))
    val out_id = Output(UInt(log2Up(max_simultaneous_matmuls).W))

    val in_last = Input(Bool())
    val out_last = Output(Bool())

    val in_valid = Input(Bool())
    val out_valid = Output(Bool())

    val bad_dataflow = Output(Bool())
  })
  val custom_pe = Module(new PEBlackBoxAdapter(inputType, outputType, accType, df, max_simultaneous_matmuls))
  
  custom_pe.io.clock := clock
    custom_pe.io.io_in_a := io.in_a
    custom_pe.io.io_in_b := io.in_b
    custom_pe.io.io_in_d := io.in_d

    custom_pe.io.io_in_control_dataflow := io.in_control.dataflow
    custom_pe.io.io_in_control_propagate := io.in_control.propagate
    custom_pe.io.io_in_control_shift := io.in_control.shift
    custom_pe.io.io_in_id := io.in_id
    custom_pe.io.io_in_last := io.in_last
    custom_pe.io.io_in_valid := io.in_valid

    io.out_a := custom_pe.io.io_out_a
    io.out_b := custom_pe.io.io_out_b
    io.out_c := custom_pe.io.io_out_c

    io.out_control.dataflow := custom_pe.io.io_out_control_dataflow
    io.out_control.propagate := custom_pe.io.io_out_control_propagate
    io.out_control.shift := custom_pe.io.io_out_control_shift
    io.out_id := custom_pe.io.io_out_id
    io.out_last := custom_pe.io.io_out_last
    io.out_valid := custom_pe.io.io_out_valid
    io.bad_dataflow := custom_pe.io.io_bad_dataflow
}
