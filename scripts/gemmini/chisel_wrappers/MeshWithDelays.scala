//DO NOT TOUCH
package gemmini

import chisel3._
import chisel3.util._

import gemmini.Util._

class MeshWithDelaysReq[T <: Data: Arithmetic, TagT <: TagQueueTag with Data](
    accType: T,
    tagType: => TagT,
    block_size: Int
) extends Bundle {
  val pe_control = new PEControl(accType)
  val a_transpose = Bool()
  val bd_transpose = Bool()
  val total_rows = UInt(log2Up(block_size + 1).W)
  val tag = tagType
  val flush = UInt(2.W) // TODO magic number

}

class MeshWithDelaysResp[T <: Data: Arithmetic, TagT <: TagQueueTag with Data](
    outType: T,
    meshCols: Int,
    tileCols: Int,
    block_size: Int,
    tagType: => TagT
) extends Bundle {
  val data = Vec(meshCols, Vec(tileCols, outType))
  val total_rows = UInt(log2Up(block_size + 1).W)
  val tag = tagType
  val last = Bool()

}

class MeshWithDelaysBlackBoxAdapter[
    T <: Data: Arithmetic,
    U <: TagQueueTag with Data
](
    inputType: T,
    val outputType: T,
    accType: T,
    tagType: => U,
    df: Dataflow.Value,
    tree_reduction: Boolean,
    tile_latency: Int,
    output_delay: Int,
    tileRows: Int,
    tileColumns: Int,
    meshRows: Int,
    meshColumns: Int,
    leftBanks: Int,
    upBanks: Int,
    outBanks: Int = 1,
    n_simultaneous_matmuls: Int = -1
) extends BlackBox
    with HasBlackBoxResource {
  val block_size = meshRows * tileRows
  val max_simultaneous_matmuls = 5
  val tagqlen = max_simultaneous_matmuls + 1

  val io = IO(new Bundle {
    val clock: Clock = Input(Clock())
    val reset: Reset = Input(Reset())

    val io_a = Flipped(Decoupled(Vec(meshRows, Vec(tileRows, inputType))))
    val io_b = Flipped(Decoupled(Vec(meshColumns, Vec(tileColumns, inputType))))
    val io_d = Flipped(Decoupled(Vec(meshColumns, Vec(tileColumns, inputType))))

    val io_req = Flipped(
      Decoupled(new MeshWithDelaysReq(accType, tagType.cloneType, block_size))
    )

    val io_resp = Valid(
      new MeshWithDelaysResp(
        outputType,
        meshColumns,
        tileColumns,
        block_size,
        tagType.cloneType
      )
    )

    val io_tags_in_progress = Output(Vec(tagqlen, tagType))
  })
  addResource("/vsrc/MeshWithDelaysBlackBox.v")
}

// TODO Add io.out.ready back in. Before it was removed, it didn't work when banking, and it seemed to assume that SRAM outputs stay steady when ren is low
// TODO Handle matrices where N1 =/= N2 =/= N3
// TODO do we flush for one cycle more than necessary?
// TODO make all inputs go straight into registers to help with physical design
// Bundle with TagQueueTag { val rob_id: UDValid[UInt]; val rows: UInt; val cols: UInt; val addr: LocalAddr },
class MeshWithDelays[T <: Data: Arithmetic, U <: TagQueueTag with Data](
    inputType: T,
    val outputType: T,
    accType: T,
    tagType: => U,
    df: Dataflow.Value,
    tree_reduction: Boolean,
    tile_latency: Int,
    output_delay: Int,
    tileRows: Int,
    tileColumns: Int,
    meshRows: Int,
    meshColumns: Int,
    leftBanks: Int,
    upBanks: Int,
    outBanks: Int = 1,
    n_simultaneous_matmuls: Int = -1
) extends Module {

  val A_TYPE = Vec(meshRows, Vec(tileRows, inputType))
  val B_TYPE = Vec(meshColumns, Vec(tileColumns, inputType))
  val C_TYPE = Vec(meshColumns, Vec(tileColumns, outputType))
  val D_TYPE = Vec(meshColumns, Vec(tileColumns, inputType))
  val S_TYPE = Vec(meshColumns, Vec(tileColumns, new PEControl(accType)))

  assert(meshRows * tileRows == meshColumns * tileColumns)
  val block_size = meshRows * tileRows

  val latency_per_pe =
    ((tile_latency + 1).toFloat / (tileRows min tileColumns)) max 1.0f
  val max_simultaneous_matmuls = if (n_simultaneous_matmuls == -1) {
    (5 * latency_per_pe).ceil.toInt
  } else {
    n_simultaneous_matmuls
  }
  assert(max_simultaneous_matmuls >= 5 * latency_per_pe)

  val tagqlen = max_simultaneous_matmuls + 1

  val io = IO(new Bundle {
    val a = Flipped(Decoupled(A_TYPE))
    val b = Flipped(Decoupled(B_TYPE))
    val d = Flipped(Decoupled(D_TYPE))

    val req = Flipped(
      Decoupled(new MeshWithDelaysReq(accType, tagType.cloneType, block_size))
    )

    val resp = Valid(
      new MeshWithDelaysResp(
        outputType,
        meshColumns,
        tileColumns,
        block_size,
        tagType.cloneType
      )
    )

    val tags_in_progress: Vec[U] = Output(Vec(tagqlen, tagType))
  })

  val custom_delays = Module(
    new MeshWithDelaysBlackBoxAdapter(
      inputType,
      outputType,
      accType,
      tagType,
      df,
      tree_reduction,
      tile_latency,
      output_delay,
      tileRows,
      tileColumns,
      meshRows,
      meshColumns,
      leftBanks,
      upBanks
    )
  )

  custom_delays.io.clock := clock
  custom_delays.io.reset := reset

  custom_delays.io.io_a.valid := io.a.valid
  custom_delays.io.io_a.bits := io.a.bits
  io.a.ready := custom_delays.io.io_a.ready

  custom_delays.io.io_b.valid := io.b.valid
  custom_delays.io.io_b.bits := io.b.bits
  io.b.ready := custom_delays.io.io_b.ready

  custom_delays.io.io_d.valid := io.d.valid
  custom_delays.io.io_d.bits := io.d.bits
  io.d.ready := custom_delays.io.io_d.ready

  custom_delays.io.io_req := DontCare
  custom_delays.io.io_req.valid := io.req.valid
  custom_delays.io.io_req.bits := io.req.bits
  io.req.ready := custom_delays.io.io_req.ready

  io.resp := DontCare
  io.resp.valid := custom_delays.io.io_resp.valid
  io.resp.bits := custom_delays.io.io_resp.bits

  io.tags_in_progress := DontCare
  io.tags_in_progress := custom_delays.io.io_tags_in_progress
}
