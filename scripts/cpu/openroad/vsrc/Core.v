module Core(
  input         clock,
  input         reset,
  input  [4:0]  io_ddpath_addr,
  input  [31:0] io_ddpath_wdata,
  input         io_ddpath_validreq,
  output [31:0] io_ddpath_rdata,
  input         io_ddpath_resetpc,
  input         io_dcpath_halt,
  input         io_imem_req_ready,
  output        io_imem_req_valid,
  output [31:0] io_imem_req_bits_addr,
  output [31:0] io_imem_req_bits_data,
  output        io_imem_req_bits_fcn,
  output [2:0]  io_imem_req_bits_typ,
  input         io_imem_resp_valid,
  input  [31:0] io_imem_resp_bits_data,
  input         io_dmem_req_ready,
  output        io_dmem_req_valid,
  output [31:0] io_dmem_req_bits_addr,
  output [31:0] io_dmem_req_bits_data,
  output        io_dmem_req_bits_fcn,
  output [2:0]  io_dmem_req_bits_typ,
  input         io_dmem_resp_valid,
  input  [31:0] io_dmem_resp_bits_data
);
  wire  custom_core_clock; // @[core.scala 41:27]
  wire  custom_core_reset; // @[core.scala 41:27]
  wire  custom_core_imem_req_ready; // @[core.scala 41:27]
  wire  custom_core_imem_req_valid; // @[core.scala 41:27]
  wire [31:0] custom_core_imem_req_bits_addr; // @[core.scala 41:27]
  wire [31:0] custom_core_imem_req_bits_data; // @[core.scala 41:27]
  wire  custom_core_imem_req_bits_fcn; // @[core.scala 41:27]
  wire [2:0] custom_core_imem_req_bits_typ; // @[core.scala 41:27]
  wire  custom_core_imem_resp_valid; // @[core.scala 41:27]
  wire [31:0] custom_core_imem_resp_bits_data; // @[core.scala 41:27]
  wire  custom_core_dmem_req_ready; // @[core.scala 41:27]
  wire  custom_core_dmem_req_valid; // @[core.scala 41:27]
  wire [31:0] custom_core_dmem_req_bits_addr; // @[core.scala 41:27]
  wire [31:0] custom_core_dmem_req_bits_data; // @[core.scala 41:27]
  wire  custom_core_dmem_req_bits_fcn; // @[core.scala 41:27]
  wire [2:0] custom_core_dmem_req_bits_typ; // @[core.scala 41:27]
  wire  custom_core_dmem_resp_valid; // @[core.scala 41:27]
  wire [31:0] custom_core_dmem_resp_bits_data; // @[core.scala 41:27]
  CoreWrapper custom_core ( // @[core.scala 41:27]
    .clock(custom_core_clock),
    .reset(custom_core_reset),
    .imem_req_ready(custom_core_imem_req_ready),
    .imem_req_valid(custom_core_imem_req_valid),
    .imem_req_bits_addr(custom_core_imem_req_bits_addr),
    .imem_req_bits_data(custom_core_imem_req_bits_data),
    .imem_req_bits_fcn(custom_core_imem_req_bits_fcn),
    .imem_req_bits_typ(custom_core_imem_req_bits_typ),
    .imem_resp_valid(custom_core_imem_resp_valid),
    .imem_resp_bits_data(custom_core_imem_resp_bits_data),
    .dmem_req_ready(custom_core_dmem_req_ready),
    .dmem_req_valid(custom_core_dmem_req_valid),
    .dmem_req_bits_addr(custom_core_dmem_req_bits_addr),
    .dmem_req_bits_data(custom_core_dmem_req_bits_data),
    .dmem_req_bits_fcn(custom_core_dmem_req_bits_fcn),
    .dmem_req_bits_typ(custom_core_dmem_req_bits_typ),
    .dmem_resp_valid(custom_core_dmem_resp_valid),
    .dmem_resp_bits_data(custom_core_dmem_resp_bits_data)
  );
  assign io_ddpath_rdata = 32'h0;
  assign io_imem_req_valid = custom_core_imem_req_valid; // @[core.scala 49:15]
  assign io_imem_req_bits_addr = custom_core_imem_req_bits_addr; // @[core.scala 49:15]
  assign io_imem_req_bits_data = custom_core_imem_req_bits_data; // @[core.scala 49:15]
  assign io_imem_req_bits_fcn = custom_core_imem_req_bits_fcn; // @[core.scala 49:15]
  assign io_imem_req_bits_typ = custom_core_imem_req_bits_typ; // @[core.scala 49:15]
  assign io_dmem_req_valid = custom_core_dmem_req_valid; // @[core.scala 50:15]
  assign io_dmem_req_bits_addr = custom_core_dmem_req_bits_addr; // @[core.scala 50:15]
  assign io_dmem_req_bits_data = custom_core_dmem_req_bits_data; // @[core.scala 50:15]
  assign io_dmem_req_bits_fcn = custom_core_dmem_req_bits_fcn; // @[core.scala 50:15]
  assign io_dmem_req_bits_typ = custom_core_dmem_req_bits_typ; // @[core.scala 50:15]
  assign custom_core_clock = clock; // @[core.scala 43:24]
  assign custom_core_reset = reset; // @[core.scala 44:24]
  assign custom_core_imem_req_ready = io_imem_req_ready; // @[core.scala 49:15]
  assign custom_core_imem_resp_valid = io_imem_resp_valid; // @[core.scala 46:28]
  assign custom_core_imem_resp_bits_data = io_imem_resp_bits_data; // @[core.scala 46:28]
  assign custom_core_dmem_req_ready = io_dmem_req_ready; // @[core.scala 50:15]
  assign custom_core_dmem_resp_valid = io_dmem_resp_valid; // @[core.scala 47:28]
  assign custom_core_dmem_resp_bits_data = io_dmem_resp_bits_data; // @[core.scala 47:28]
endmodule
