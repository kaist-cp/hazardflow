// See README.md for license details.

package gemmini

import chisel3._
import chisel3.util._
import Util._

class TileBlackBoxAdapter[T <: Data](inputType: T, outputType: T, accType: T, df: Dataflow.Value, tree_reduction: Boolean, max_simultaneous_matmuls: Int, val rows: Int, val columns: Int)(implicit ev: Arithmetic[T])
    extends BlackBox with HasBlackBoxResource {
    val io = IO(new Bundle {
        val clock = Input(Clock())

        val io_in_a        = Input(Vec(rows, inputType))
        val io_in_b        = Input(Vec(columns, outputType)) // This is the output of the tile next to it
        val io_in_d        = Input(Vec(columns, outputType))

        val io_in_control  = Input(Vec(columns, new PEControl(accType)))
        val io_in_id       = Input(Vec(columns, UInt(log2Up(max_simultaneous_matmuls).W)))
        val io_in_last  = Input(Vec(columns, Bool()))

        val io_out_a       = Output(Vec(rows, inputType))
        val io_out_c       = Output(Vec(columns, outputType))
        val io_out_b       = Output(Vec(columns, outputType))

        val io_out_control = Output(Vec(columns, new PEControl(accType)))
        val io_out_id      = Output(Vec(columns, UInt(log2Up(max_simultaneous_matmuls).W)))
        val io_out_last    = Output(Vec(columns, Bool()))

        val io_in_valid = Input(Vec(columns, Bool()))
        val io_out_valid = Output(Vec(columns, Bool()))

        val io_bad_dataflow = Output(Bool())
    })
    addResource("/vsrc/TileBlackBox.v")
}

/**
  * A Tile is a purely combinational 2D array of passThrough PEs.
  * a, b, s, and in_propag are broadcast across the entire array and are passed through to the Tile's outputs
  * @param width The data width of each PE in bits
  * @param rows Number of PEs on each row
  * @param columns Number of PEs on each column
  */
class Tile[T <: Data](inputType: T, outputType: T, accType: T, df: Dataflow.Value, tree_reduction: Boolean, max_simultaneous_matmuls: Int, val rows: Int, val columns: Int)(implicit ev: Arithmetic[T]) extends Module {
  val io = IO(new Bundle {
    val in_a        = Input(Vec(rows, inputType))
    val in_b        = Input(Vec(columns, outputType)) // This is the output of the tile next to it
    val in_d        = Input(Vec(columns, outputType))

    val in_control  = Input(Vec(columns, new PEControl(accType)))
    val in_id       = Input(Vec(columns, UInt(log2Up(max_simultaneous_matmuls).W)))
    val in_last  = Input(Vec(columns, Bool()))

    val out_a       = Output(Vec(rows, inputType))
    val out_c       = Output(Vec(columns, outputType))
    val out_b       = Output(Vec(columns, outputType))

    val out_control = Output(Vec(columns, new PEControl(accType)))
    val out_id      = Output(Vec(columns, UInt(log2Up(max_simultaneous_matmuls).W)))
    val out_last    = Output(Vec(columns, Bool()))

    val in_valid = Input(Vec(columns, Bool()))
    val out_valid = Output(Vec(columns, Bool()))

    val bad_dataflow = Output(Bool())
  })

  import ev._

  val custom_tile = Module(new TileBlackBoxAdapter(inputType, outputType, accType, df, tree_reduction, max_simultaneous_matmuls, rows, columns))

    custom_tile.io.clock := clock
    custom_tile.io.io_in_a := io.in_a
    custom_tile.io.io_in_b := io.in_b
    custom_tile.io.io_in_d := io.in_d
    custom_tile.io.io_in_control := io.in_control
    custom_tile.io.io_in_id := io.in_id
    custom_tile.io.io_in_last := io.in_last
    custom_tile.io.io_in_valid := io.in_valid

    io.out_a := custom_tile.io.io_out_a
    io.out_c := custom_tile.io.io_out_c
    io.out_b := custom_tile.io.io_out_b
    io.out_control := custom_tile.io.io_out_control
    io.out_id := custom_tile.io.io_out_id
    io.out_last := custom_tile.io.io_out_last
    io.out_valid := custom_tile.io.io_out_valid
    io.bad_dataflow := custom_tile.io.io_bad_dataflow

}
