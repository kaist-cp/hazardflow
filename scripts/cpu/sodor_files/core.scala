//**************************************************************************
// RISCV Processor
//--------------------------------------------------------------------------

package Sodor
{

import chisel3._
import Common._

import java.io.File

import chisel3.util.HasBlackBoxResource
import chisel3.util.HasBlackBoxPath

class CoreIo(implicit val conf: SodorConfiguration) extends Bundle
{
   val ddpath = Flipped(new DebugDPath())
   val dcpath = Flipped(new DebugCPath())
   val imem = new MemPortIo(conf.xprlen)
   val dmem = new MemPortIo(conf.xprlen)
}

class CoreWrapperIo(implicit val conf: SodorConfiguration) extends Bundle
{
   val clock = Input(Clock())
   val reset = Input(Reset())
   val imem = new MemPortIo(conf.xprlen)
   val dmem = new MemPortIo(conf.xprlen)
}


class CoreWrapper()(implicit val conf: SodorConfiguration) extends BlackBox with HasBlackBoxPath {
  val io = IO(new CoreWrapperIo())
  addPath(new File("COREWRAPPERPATH").getCanonicalPath)
}

class Core()(implicit val conf: SodorConfiguration) extends Module()
{
  val io = IO(new CoreIo())
  val custom_core = Module(new CoreWrapper())

  custom_core.io.clock := clock
  custom_core.io.reset := reset

  custom_core.io.imem.resp <> io.imem.resp
  custom_core.io.dmem.resp <> io.dmem.resp

  io.imem.req <> custom_core.io.imem.req
  io.dmem.req <> custom_core.io.dmem.req

  io.ddpath <> DontCare
  io.dcpath <> DontCare
}

}