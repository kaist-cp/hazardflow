package gemmini

import chisel3._
import chisel3.util._

class DMACommandTrackerBlackBoxAdapter[T <: Data](
    val nCmds: Int,
    val maxBytes: Int,
    tag_t: => Bundle { val rob_id: UInt }
) extends BlackBox(Map("BYTE_WIDTH" -> log2Up(maxBytes + 1)))
    with HasBlackBoxResource {
  val io = IO(new Bundle {
    val clock: Clock = Input(Clock())
    val reset: Reset = Input(Reset())

    // Input signals
    val io_alloc_valid: Bool = Input(Bool())
    val io_alloc_bits_tag_rob_id: UInt = Input(UInt(6.W))
    val io_alloc_bits_bytes_to_read: UInt = Input(UInt(log2Up(maxBytes + 1).W))
    val io_request_returned_valid: Bool = Input(Bool())
    val io_request_returned_bits_bytes_read: UInt =
      Input(UInt(log2Up(maxBytes + 1).W))
    val io_request_returned_bits_cmd_id: UInt =
      Input(UInt((log2Ceil(nCmds) max 1).W))
    val io_cmd_completed_ready: Bool = Input(Bool())

    // Output signals
    val io_alloc_ready: Bool = Output(Bool())
    val io_alloc_bits_cmd_id: UInt = Output(UInt(1.W))
    val io_cmd_completed_valid: Bool = Output(Bool())
    val io_cmd_completed_bits_tag_rob_id: UInt = Output(UInt(6.W))
  })
  addResource("/vsrc/DMACommandTrackerBlackBox.v")
}

// This module is meant to go inside the Load controller, where it can track which commands are currently
// in flight and which are completed
class DMACommandTracker[T <: Data](
    val nCmds: Int,
    val maxBytes: Int,
    tag_t: => Bundle { val rob_id: UInt }
) extends Module {
  def cmd_id_t = UInt((log2Ceil(nCmds) max 1).W)

  val io = IO(new Bundle {
    // TODO is there an existing decoupled interface in the standard library which matches this use-case?
    val alloc = new Bundle {
      val valid = Input(Bool())
      val ready = Output(Bool())

      class BitsT(tag_t: => Bundle { val rob_id: UInt }, cmd_id_t: UInt)
          extends Bundle {
        // This was only spun off as its own class to resolve CloneType errors
        val tag = Input(tag_t.cloneType)
        val bytes_to_read = Input(UInt(log2Up(maxBytes + 1).W))
        val cmd_id = Output(cmd_id_t.cloneType)
      }

      val bits = new BitsT(tag_t.cloneType, cmd_id_t.cloneType)

      def fire(dummy: Int = 0) = valid && ready
    }

    class RequestReturnedT(cmd_id_t: UInt) extends Bundle {
      // This was only spun off as its own class to resolve CloneType errors
      val bytes_read = UInt(log2Up(maxBytes + 1).W)
      val cmd_id = cmd_id_t.cloneType

    }

    val request_returned =
      Flipped(Valid(new RequestReturnedT(cmd_id_t.cloneType)))

    class CmdCompletedT(tag_t: Bundle { val rob_id: UInt }) extends Bundle {
      val tag = tag_t.cloneType
    }

    val cmd_completed = Decoupled(new CmdCompletedT(tag_t.cloneType))

    val busy = Output(Bool())
  })

  val custom_tracker = Module(
    new DMACommandTrackerBlackBoxAdapter(nCmds, maxBytes, tag_t)
  )

  custom_tracker.io.clock := clock
  custom_tracker.io.reset := reset

  custom_tracker.io.io_alloc_valid := io.alloc.valid
  custom_tracker.io.io_alloc_bits_tag_rob_id := io.alloc.bits.tag.rob_id
  custom_tracker.io.io_alloc_bits_bytes_to_read := io.alloc.bits.bytes_to_read
  custom_tracker.io.io_request_returned_valid := io.request_returned.valid
  custom_tracker.io.io_request_returned_bits_bytes_read := io.request_returned.bits.bytes_read
  custom_tracker.io.io_request_returned_bits_cmd_id := io.request_returned.bits.cmd_id
  custom_tracker.io.io_cmd_completed_ready := io.cmd_completed.ready

  io.alloc.ready := custom_tracker.io.io_alloc_ready
  io.alloc.bits.cmd_id := custom_tracker.io.io_alloc_bits_cmd_id
  io.cmd_completed.valid := custom_tracker.io.io_cmd_completed_valid
  io.cmd_completed.bits.tag.rob_id := custom_tracker.io.io_cmd_completed_bits_tag_rob_id
  io.busy := custom_tracker.io.io_alloc_ready
}
