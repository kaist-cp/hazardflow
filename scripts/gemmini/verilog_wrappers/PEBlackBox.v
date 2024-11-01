module PEBlackBoxAdapter (
  input         clock,
  input  [7:0]  io_in_a,
  input  [19:0] io_in_b,
                io_in_d,
  input         io_in_control_dataflow,
                io_in_control_propagate,
  input  [4:0]  io_in_control_shift,
  input  [2:0]  io_in_id,
  input         io_in_last,
                io_in_valid,
  output [7:0]  io_out_a,
  output [19:0] io_out_b,
                io_out_c,
  output        io_out_control_dataflow,
                io_out_control_propagate,
  output [4:0]  io_out_control_shift,
  output [2:0]  io_out_id,
  output        io_out_last,
                io_out_valid,
                io_bad_dataflow
);

wire io_out_valid_0;
wire io_out_valid_1_0;
wire io_out_valid_1_1;

pe_top pe (
    .clk(clock),
    .rst(1'b0),

    ////////// Input //////////
    // PE Row Data
    .in_input_0_payload_discriminant(io_in_valid),
    .in_input_0_payload_Some_0_a_0(io_in_a),

    // PE Column Data
    .in_input_1_0_payload_discriminant(io_in_valid),
    .in_input_1_0_payload_Some_0_b_0(io_in_b),
    .in_input_1_0_payload_Some_0_d_0(io_in_d),
    
    // PE Column Control
    .in_input_1_1_payload_discriminant(io_in_valid),
    .in_input_1_1_payload_Some_0_id(io_in_id),
    .in_input_1_1_payload_Some_0_last(io_in_last),
    .in_input_1_1_payload_Some_0_control_dataflow_discriminant(io_in_control_dataflow),
    .in_input_1_1_payload_Some_0_control_propagate_discriminant(io_in_control_propagate),
    .in_input_1_1_payload_Some_0_control_shift(io_in_control_shift),

    ////////// Output //////////
    // PE Row Data
    .out_output_0_payload_discriminant(io_out_valid_0),
    .out_output_0_payload_Some_0_a_0(io_out_a),

    // PE Column Data
    .out_output_1_0_payload_discriminant(io_out_valid_1_0),
    .out_output_1_0_payload_Some_0_b_0(io_out_b),
    .out_output_1_0_payload_Some_0_d_0(io_out_c),
    
    // PE Column Control
    .out_output_1_1_payload_discriminant(io_out_valid_1_1),
    .out_output_1_1_payload_Some_0_id(io_out_id),
    .out_output_1_1_payload_Some_0_last(io_out_last),
    .out_output_1_1_payload_Some_0_control_dataflow_discriminant(io_out_control_dataflow),
    .out_output_1_1_payload_Some_0_control_propagate_discriminant(io_out_control_propagate),
    .out_output_1_1_payload_Some_0_control_shift(io_out_control_shift)
);

assign io_out_valid = io_out_valid_0 && io_out_valid_1_0 && io_out_valid_1_1;
assign io_bad_dataflow = 1'b0;

endmodule
