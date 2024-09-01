//**************************************************************************
// RISCV Processor Register File
//--------------------------------------------------------------------------
//

package Sodor
{

import chisel3._
import chisel3.util._


import Constants._
import Common._

class RFileIo(implicit val conf: SodorConfiguration) extends Bundle()
{
   val rs1_addr = Input(UInt(5.W))
   val rs1_data = Output(UInt(conf.xprlen.W))
   val rs2_addr = Input(UInt(5.W))
   val rs2_data = Output(UInt(conf.xprlen.W))
   val dm_addr = Input(UInt(5.W))
   val dm_rdata = Output(UInt(conf.xprlen.W))
   val dm_wdata = Input(UInt(conf.xprlen.W))
   val dm_en = Input(Bool())

   val waddr    = Input(UInt(5.W))
   val wdata    = Input(UInt(conf.xprlen.W))
   val wen      = Input(Bool())
}

class RegisterFile(implicit val conf: SodorConfiguration) extends Module
{
   val io = IO(new RFileIo())

   val regfile = Mem(32, UInt(conf.xprlen.W))

   when (io.wen && (io.waddr =/= 0.U))
   {
      regfile(io.waddr) := io.wdata
   }

   when (io.dm_en && (io.dm_addr =/= 0.U))
   {
      regfile(io.dm_addr) := io.dm_wdata
   }

   io.rs1_data := Mux((io.rs1_addr =/= 0.U), regfile(io.rs1_addr), 0.U)
   io.rs2_data := Mux((io.rs2_addr =/= 0.U), regfile(io.rs2_addr), 0.U)
   io.dm_rdata := Mux((io.dm_addr =/= 0.U), regfile(io.dm_addr), 0.U)

   printf("tick_start\n")
   printf("Reg[0](%x)\n", regfile(0.U))
   printf("Reg[1](%x)\n", regfile(1.U))
   printf("Reg[2](%x)\n", regfile(2.U))
   printf("Reg[3](%x)\n", regfile(3.U))
   printf("Reg[4](%x)\n", regfile(4.U))
   printf("Reg[5](%x)\n", regfile(5.U))
   printf("Reg[6](%x)\n", regfile(6.U))
   printf("Reg[7](%x)\n", regfile(7.U))
   printf("Reg[8](%x)\n", regfile(8.U))
   printf("Reg[9](%x)\n", regfile(9.U))
   printf("Reg[10](%x)\n", regfile(10.U))
   printf("Reg[11](%x)\n", regfile(11.U))
   printf("Reg[12](%x)\n", regfile(12.U))
   printf("Reg[13](%x)\n", regfile(13.U))
   printf("Reg[14](%x)\n", regfile(14.U))
   printf("Reg[15](%x)\n", regfile(15.U))
   printf("Reg[16](%x)\n", regfile(16.U))
   printf("Reg[17](%x)\n", regfile(17.U))
   printf("Reg[18](%x)\n", regfile(18.U))
   printf("Reg[19](%x)\n", regfile(19.U))
   printf("Reg[20](%x)\n", regfile(20.U))
   printf("Reg[21](%x)\n", regfile(21.U))
   printf("Reg[22](%x)\n", regfile(22.U))
   printf("Reg[23](%x)\n", regfile(23.U))
   printf("Reg[24](%x)\n", regfile(24.U))
   printf("Reg[25](%x)\n", regfile(25.U))
   printf("Reg[26](%x)\n", regfile(26.U))
   printf("Reg[27](%x)\n", regfile(27.U))
   printf("Reg[28](%x)\n", regfile(28.U))
   printf("Reg[29](%x)\n", regfile(29.U))
   printf("Reg[30](%x)\n", regfile(30.U))
   printf("Reg[31](%x)\n", regfile(31.U))



}
}