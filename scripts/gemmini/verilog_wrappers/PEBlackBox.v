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

    wire in_input_0_payload_discriminant = io_in_valid;
    wire [8-1:0] in_input_0_payload_Some_0_a = io_in_a;

    wire in_input_1_0_payload_discriminant = io_in_valid;
    wire [20-1:0] in_input_1_0_payload_Some_0_b = io_in_b;
    wire [20-1:0] in_input_1_0_payload_Some_0_d = io_in_d;
    wire in_input_1_1_payload_Some_0_control_dataflow_discriminant = io_in_control_dataflow;
    wire in_input_1_1_payload_Some_0_control_propagate_discriminant = io_in_control_propagate;
    wire [5-1:0] in_input_1_1_payload_Some_0_control_shift = io_in_control_shift;
    wire [3-1:0] in_input_1_1_payload_Some_0_id = io_in_id;
    wire in_input_1_1_payload_Some_0_bad_dataflow = 1'b0;
    wire in_input_1_1_payload_Some_0_last = io_in_last;

    wire out_output_0_payload_discriminant;
    wire [8-1:0] out_output_0_payload_Some_0_a;
    wire out_output_1_0_payload_discriminant;
    wire [20-1:0] out_output_1_0_payload_Some_0_b;
    wire [20-1:0] out_output_1_0_payload_Some_0_d;
    wire out_output_1_1_payload_discriminant;
    wire out_output_1_1_payload_Some_0_control_dataflow_discriminant;
    wire out_output_1_1_payload_Some_0_control_propagate_discriminant;
    wire [5-1:0] out_output_1_1_payload_Some_0_control_shift;
    wire [3-1:0] out_output_1_1_payload_Some_0_id;
    wire out_output_1_1_payload_Some_0_bad_dataflow;
    wire out_output_1_1_payload_Some_0_last;


    pe_top pe
    (
        .clk(clock),
        .rst(1'b0),

        ////////// Input //////////
        // PE Row Data
        .in_input_0_payload_discriminant(in_input_0_payload_discriminant),
        .in_input_0_payload_Some_0_a(in_input_0_payload_Some_0_a),
        
        // PE Column Data
        .in_input_1_0_payload_discriminant(in_input_1_0_payload_discriminant),
        .in_input_1_0_payload_Some_0_b(in_input_1_0_payload_Some_0_b),
        .in_input_1_0_payload_Some_0_d(in_input_1_0_payload_Some_0_d),
        
        // PE Column Control
        .in_input_1_1_payload_discriminant(1'b1),
        .in_input_1_1_payload_Some_0_id(in_input_1_1_payload_Some_0_id),
        .in_input_1_1_payload_Some_0_last(in_input_1_1_payload_Some_0_last),
        .in_input_1_1_payload_Some_0_control_dataflow_discriminant(in_input_1_1_payload_Some_0_control_dataflow_discriminant),
        .in_input_1_1_payload_Some_0_control_propagate_discriminant(in_input_1_1_payload_Some_0_control_propagate_discriminant),
        .in_input_1_1_payload_Some_0_control_shift(in_input_1_1_payload_Some_0_control_shift),
        .in_input_1_1_payload_Some_0_bad_dataflow(in_input_1_1_payload_Some_0_bad_dataflow),

        ////////// Output //////////
        // PE Row Data
        .out_output_0_payload_discriminant(out_output_0_payload_discriminant),
        .out_output_0_payload_Some_0_a(out_output_0_payload_Some_0_a),

        // PE Column Data
        .out_output_1_0_payload_discriminant(out_output_1_0_payload_discriminant),
        .out_output_1_0_payload_Some_0_b(out_output_1_0_payload_Some_0_b),
        .out_output_1_0_payload_Some_0_d(out_output_1_0_payload_Some_0_d),
        
        // PE Column Control
        .out_output_1_1_payload_discriminant(out_output_1_1_payload_discriminant),
        .out_output_1_1_payload_Some_0_id(out_output_1_1_payload_Some_0_id),
        .out_output_1_1_payload_Some_0_last(out_output_1_1_payload_Some_0_last),
        .out_output_1_1_payload_Some_0_control_dataflow_discriminant(out_output_1_1_payload_Some_0_control_dataflow_discriminant),
        .out_output_1_1_payload_Some_0_control_propagate_discriminant(out_output_1_1_payload_Some_0_control_propagate_discriminant),
        .out_output_1_1_payload_Some_0_control_shift(out_output_1_1_payload_Some_0_control_shift),
        .out_output_1_1_payload_Some_0_bad_dataflow(out_output_1_1_payload_Some_0_bad_dataflow)
    );

    assign io_out_a = out_output_0_payload_Some_0_a;
    assign io_out_b = out_output_1_0_payload_Some_0_b;
    assign io_out_c = out_output_1_0_payload_Some_0_d;
    assign io_out_control_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant;
    assign io_out_control_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant;
    
    assign io_out_control_shift = out_output_1_1_payload_Some_0_control_shift;
    assign io_out_id = out_output_1_1_payload_Some_0_id;
    assign io_out_last = out_output_1_1_payload_Some_0_last;
    assign io_out_valid = out_output_0_payload_discriminant && out_output_1_0_payload_discriminant;
    assign io_bad_dataflow = out_output_1_1_payload_Some_0_bad_dataflow;

endmodule
