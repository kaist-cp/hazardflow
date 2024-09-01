module CoreWrapper(
  input         clock,  // clk
  input         reset,  // rst

  input         imem_req_ready,
  output        imem_req_valid,
  output [31:0] imem_req_bits_addr,
  output [31:0] imem_req_bits_data,
  output        imem_req_bits_fcn,
  output [2:0]  imem_req_bits_typ,
  input         imem_resp_valid,
  input  [31:0] imem_resp_bits_data,


  input         dmem_req_ready,
  output        dmem_req_valid,
  output [31:0] dmem_req_bits_addr,
  output [31:0] dmem_req_bits_data,
  output        dmem_req_bits_fcn,
  output [2:0]  dmem_req_bits_typ,
  input         dmem_resp_valid,
  input  [31:0] dmem_resp_bits_data
);
  // We ignore signals below:
  wire in_input_0_output_ready_drain;
  wire in_input_1_output_ready_drain;

  // Wiring

  core_top core_hf (
    .clk(clock),
    .rst(reset),

    // MemResp of Imem
    .in_input_0_output_payload_discriminant(imem_resp_valid),      // imem_resp_valid
    .in_input_0_output_payload_Some_0_data(imem_resp_bits_data),  // imem_resp_bits_data
    .in_input_0_output_payload_Some_0_addr(imem_req_bits_addr),
    .in_input_0_output_resolver_ready(in_input_0_output_ready_drain),
    // MemResp of Dmem
    .in_input_1_output_payload_discriminant(dmem_resp_valid),      // dmem_resp_valid
    .in_input_1_output_payload_Some_0_data(dmem_resp_bits_data),  // dmem_resp_bits_data
    .in_input_1_output_payload_Some_0_addr(dmem_req_bits_addr),
    .in_input_1_output_resolver_ready(in_input_1_output_ready_drain),
    // MemReq of Imem
    .out_input_0_input_0_payload_discriminant(imem_req_valid),           // imem_req_valid
    .out_input_0_input_0_payload_Some_0_addr(imem_req_bits_addr),   // imem_req_bits_addr
    .out_input_0_input_0_payload_Some_0_data(imem_req_bits_data),   // imem_req_bits_data
    .out_input_0_input_0_payload_Some_0_fcn_discriminant(imem_req_bits_fcn),    // imem_req_bits_fcn
    .out_input_0_input_0_payload_Some_0_typ_discriminant(imem_req_bits_typ),    // imem_req_bits_typ
    .out_input_0_input_0_resolver_ready(imem_req_ready & in_input_0_output_ready_drain),                   // imem_req_ready
    // Memreq of Dmem
    .out_input_1_input_0_payload_discriminant(dmem_req_valid),           // dmem_req_valid
    .out_input_1_input_0_payload_Some_0_addr(dmem_req_bits_addr),   // dmem_req_bits_addr
    .out_input_1_input_0_payload_Some_0_data(dmem_req_bits_data),   // dmem_req_bits_data
    .out_input_1_input_0_payload_Some_0_fcn_discriminant(dmem_req_bits_fcn),    // dmem_req_bits_fcn
    .out_input_1_input_0_payload_Some_0_typ_discriminant(dmem_req_bits_typ),    // dmem_req_bits_typ
    .out_input_1_input_0_resolver_ready(dmem_req_ready & in_input_1_output_ready_drain)                    // dmem_req_ready
  );

endmodule