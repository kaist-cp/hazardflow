
package gemmini

import chisel3._
import chisel3.util._
import chisel3.experimental._

class MeshBlackBoxAdapter[T <: Data : Arithmetic](inputType: T, outputType: T, accType: T,
                                   df: Dataflow.Value, tree_reduction: Boolean, tile_latency: Int,
                                   max_simultaneous_matmuls: Int, output_delay: Int,
                                   val tileRows: Int, val tileColumns: Int,
                                   val meshRows: Int, val meshColumns: Int) extends BlackBox with HasBlackBoxResource {
  val io = IO(new Bundle {
    val clock = Input(Clock())

    val io_in_a = Input(Vec(meshRows, Vec(tileRows, inputType)))
    val io_in_b = Input(Vec(meshColumns, Vec(tileColumns, inputType)))
    val io_in_d = Input(Vec(meshColumns, Vec(tileColumns, inputType)))
    val io_in_control = Input(Vec(meshColumns, Vec(tileColumns, new PEControl(accType))))
    val io_in_id = Input(Vec(meshColumns, Vec(tileColumns, UInt(log2Up(max_simultaneous_matmuls).W)))) // The unique id of this particular matmul
    val io_in_last = Input(Vec(meshColumns, Vec(tileColumns, Bool())))
    val io_in_valid = Input(Vec(meshColumns, Vec(tileColumns, Bool())))

    val io_out_b = Output(Vec(meshColumns, Vec(tileColumns, outputType)))
    val io_out_c = Output(Vec(meshColumns, Vec(tileColumns, outputType)))
    val io_out_valid = Output(Vec(meshColumns, Vec(tileColumns, Bool())))
    val io_out_control = Output(Vec(meshColumns, Vec(tileColumns, new PEControl(accType))))
    val io_out_id = Output(Vec(meshColumns, Vec(tileColumns, UInt(log2Up(max_simultaneous_matmuls).W))))
    val io_out_last = Output(Vec(meshColumns, Vec(tileColumns, Bool())))
  })

  addResource("/vsrc/MeshBlackBox.v")
}

/**
  * A Grid is a 2D array of Tile modules with registers in between each tile and
  * registers from the bottom row and rightmost column of tiles to the Grid outputs.
  * @param width
  * @param tileRows
  * @param tileColumns
  * @param meshRows
  * @param meshColumns
  */
class Mesh[T <: Data : Arithmetic](inputType: T, outputType: T, accType: T,
                                   df: Dataflow.Value, tree_reduction: Boolean, tile_latency: Int,
                                   max_simultaneous_matmuls: Int, output_delay: Int,
                                   val tileRows: Int, val tileColumns: Int,
                                   val meshRows: Int, val meshColumns: Int) extends Module {
  val io = IO(new Bundle {
    val in_a = Input(Vec(meshRows, Vec(tileRows, inputType)))
    val in_b = Input(Vec(meshColumns, Vec(tileColumns, inputType)))
    val in_d = Input(Vec(meshColumns, Vec(tileColumns, inputType)))
    val in_control = Input(Vec(meshColumns, Vec(tileColumns, new PEControl(accType))))
    val in_id = Input(Vec(meshColumns, Vec(tileColumns, UInt(log2Up(max_simultaneous_matmuls).W)))) // The unique id of this particular matmul
    val in_last = Input(Vec(meshColumns, Vec(tileColumns, Bool())))
    val in_valid = Input(Vec(meshColumns, Vec(tileColumns, Bool())))

    val out_b = Output(Vec(meshColumns, Vec(tileColumns, outputType)))
    val out_c = Output(Vec(meshColumns, Vec(tileColumns, outputType)))
    val out_valid = Output(Vec(meshColumns, Vec(tileColumns, Bool())))
    val out_control = Output(Vec(meshColumns, Vec(tileColumns, new PEControl(accType))))
    val out_id = Output(Vec(meshColumns, Vec(tileColumns, UInt(log2Up(max_simultaneous_matmuls).W))))
    val out_last = Output(Vec(meshColumns, Vec(tileColumns, Bool())))
  })

  val custom_mesh = Module(new MeshBlackBoxAdapter(inputType, outputType, accType, df, tree_reduction, tile_latency, max_simultaneous_matmuls, output_delay, tileRows, tileColumns, meshRows, meshColumns))

  custom_mesh.io.clock := clock

    custom_mesh.io.io_in_a := io.in_a
    custom_mesh.io.io_in_b := io.in_b
    custom_mesh.io.io_in_d := io.in_d
    custom_mesh.io.io_in_control := io.in_control
    custom_mesh.io.io_in_id := io.in_id
    custom_mesh.io.io_in_last := io.in_last
    custom_mesh.io.io_in_valid := io.in_valid

    io.out_b := custom_mesh.io.io_out_b
    io.out_c := custom_mesh.io.io_out_c
    io.out_valid := custom_mesh.io.io_out_valid
    io.out_control := custom_mesh.io.io_out_control
    io.out_id := custom_mesh.io.io_out_id
    io.out_last := custom_mesh.io.io_out_last
}
