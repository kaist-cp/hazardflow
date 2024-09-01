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

//// This module is meant to go inside the Load controller, where it can track which commands are currently
//// in flight and which are completed
//class DMACommandTracker[T <: Data](val nCmds: Int, val maxBytes: Int, tag_t: => T) extends Module {
//  def cmd_id_t = UInt((log2Ceil(nCmds) max 1).W)
//
//  val io = IO(new Bundle {
//    // TODO is there an existing decoupled interface in the standard library which matches this use-case?
//    val alloc = new Bundle {
//      val valid = Input(Bool())
//      val ready = Output(Bool())
//
//      class BitsT(tag_t: => T, cmd_id_t: UInt) extends Bundle {
//        // This was only spun off as its own class to resolve CloneType errors
//        val tag = Input(tag_t.cloneType)
//        val bytes_to_read = Input(UInt(log2Up(maxBytes+1).W))
//        val cmd_id = Output(cmd_id_t.cloneType)
//      }
//
//      val bits = new BitsT(tag_t.cloneType, cmd_id_t.cloneType)
//
//      def fire(dummy: Int = 0) = valid && ready
//    }
//
//    class RequestReturnedT(cmd_id_t: UInt) extends Bundle {
//      // This was only spun off as its own class to resolve CloneType errors
//      val bytes_read = UInt(log2Up(maxBytes+1).W)
//      val cmd_id = cmd_id_t.cloneType
//
//    }
//
//    val request_returned = Flipped(Valid(new RequestReturnedT(cmd_id_t.cloneType)))
//
//    class CmdCompletedT(cmd_id_t: UInt, tag_t: T) extends Bundle {
//      val cmd_id = cmd_id_t.cloneType
//      val tag = tag_t.cloneType
//
//    }
//
//    val cmd_completed = Decoupled(new CmdCompletedT(cmd_id_t.cloneType, tag_t.cloneType))
//
//    val busy = Output(Bool())
//  })
//
//  class Entry extends Bundle {
//    val valid = Bool()
//    val tag = tag_t.cloneType
//    val bytes_left = UInt(log2Up(maxBytes+1).W)
//
//    def init(dummy: Int = 0): Unit = {
//      valid := false.B
//    }
//  }
//
//  // val cmds = RegInit(VecInit(Seq.fill(nCmds)(entry_init)))
//  val cmds = Reg(Vec(nCmds, new Entry))
//  val cmd_valids = cmds.map(_.valid)
//
//  val next_empty_alloc = MuxCase(0.U, cmd_valids.zipWithIndex.map { case (v, i) => (!v) -> i.U })
//
//  io.alloc.ready := !cmd_valids.reduce(_ && _)
//  io.alloc.bits.cmd_id := next_empty_alloc
//
//  io.busy := cmd_valids.reduce(_ || _)
//
//  val cmd_completed_id = MuxCase(0.U, cmds.zipWithIndex.map { case (cmd, i) =>
//    (cmd.valid && cmd.bytes_left === 0.U) -> i.U
//  })
//  io.cmd_completed.valid := cmds.map(cmd => cmd.valid && cmd.bytes_left === 0.U).reduce(_ || _)
//  io.cmd_completed.bits.cmd_id := cmd_completed_id
//  io.cmd_completed.bits.tag := cmds(cmd_completed_id).tag
//
//  when (io.alloc.fire()) {
//    cmds(next_empty_alloc).valid := true.B
//    cmds(next_empty_alloc).tag := io.alloc.bits.tag
//    cmds(next_empty_alloc).bytes_left := io.alloc.bits.bytes_to_read
//  }
//
//  when (io.request_returned.fire) {
//    val cmd_id = io.request_returned.bits.cmd_id
//    cmds(cmd_id).bytes_left := cmds(cmd_id).bytes_left - io.request_returned.bits.bytes_read
//
//    assert(cmds(cmd_id).valid)
//    assert(cmds(cmd_id).bytes_left >= io.request_returned.bits.bytes_read)
//  }
//
//  when (io.cmd_completed.fire) {
//    cmds(io.cmd_completed.bits.cmd_id).valid := false.B
//  }
//
//  when (reset.asBool) {
//    cmds.foreach(_.init())
//  }
//}
