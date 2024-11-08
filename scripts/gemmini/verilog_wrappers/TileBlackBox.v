module TileBlackBoxAdapter (
    input         clock,
    input  [7:0]  io_in_a_0,
    input  [19:0] io_in_b_0,
                io_in_d_0,
    input         io_in_control_0_dataflow,
                io_in_control_0_propagate,
    input  [4:0]  io_in_control_0_shift,
    input  [2:0]  io_in_id_0,
    input         io_in_last_0,
                io_in_valid_0,
    output [7:0]  io_out_a_0,
    output [19:0] io_out_c_0,
                io_out_b_0,
    output        io_out_control_0_dataflow,
                io_out_control_0_propagate,
    output [4:0]  io_out_control_0_shift,
    output [2:0]  io_out_id_0,
    output        io_out_last_0,
                io_out_valid_0,
                io_bad_dataflow  
);

    wire in_input_0_payload_discriminant;
    wire [8-1:0] in_input_0_payload_Some_0_a_0;

    wire in_input_1_0_payload_discriminant;
    wire [20-1:0] in_input_1_0_payload_Some_0_b_0;
    wire [20-1:0] in_input_1_0_payload_Some_0_d_0;

    wire [3-1:0] in_input_1_1_payload_Some_0_id;
    wire in_input_1_1_payload_Some_0_last;
    wire in_input_1_1_payload_Some_0_control_dataflow_discriminant;
    wire in_input_1_1_payload_Some_0_control_propagate_discriminant;
    wire [5-1:0] in_input_1_1_payload_Some_0_control_shift;
    wire in_input_1_1_payload_Some_0_bad_dataflow;

    wire out_output_0_payload_discriminant;
    wire [8-1:0] out_output_0_payload_Some_0_a_0;

    wire out_output_1_0_payload_discriminant;
    wire [20-1:0] out_output_1_0_payload_Some_0_b_0;
    wire [20-1:0] out_output_1_0_payload_Some_0_d_0;

    wire out_output_1_0_payload_discriminant;
    wire [3-1:0] out_output_1_1_payload_Some_0_id;
    wire out_output_1_1_payload_Some_0_last;
    wire out_output_1_1_payload_discriminant;
    wire out_output_1_1_payload_Some_0_control_dataflow_discriminant;
    wire out_output_1_1_payload_Some_0_control_propagate_discriminant;
    wire [5-1:0] out_output_1_1_payload_Some_0_control_shift;
    wire out_output_1_1_payload_Some_0_bad_dataflow;

    tile_1_1_top tile_1_1
    (
        .clk(clock),
        .rst(1'b0),

        .in_input_0_payload_discriminant(in_input_0_payload_discriminant),
        .in_input_0_payload_Some_0_a_0(in_input_0_payload_Some_0_a_0),

        .in_input_1_0_payload_discriminant(in_input_1_0_payload_discriminant),
        .in_input_1_0_payload_Some_0_b_0(in_input_1_0_payload_Some_0_b_0),
        .in_input_1_0_payload_Some_0_d_0(in_input_1_0_payload_Some_0_d_0),
        
        .in_input_1_1_payload_discriminant(1'b1),
        .in_input_1_1_payload_Some_0_id(in_input_1_1_payload_Some_0_id),
        .in_input_1_1_payload_Some_0_last(in_input_1_1_payload_Some_0_last),
        .in_input_1_1_payload_Some_0_control_dataflow_discriminant(in_input_1_1_payload_Some_0_control_dataflow_discriminant),
        .in_input_1_1_payload_Some_0_control_propagate_discriminant(in_input_1_1_payload_Some_0_control_propagate_discriminant),
        .in_input_1_1_payload_Some_0_control_shift(in_input_1_1_payload_Some_0_control_shift),
        .in_input_1_1_payload_Some_0_bad_dataflow(in_input_1_1_payload_Some_0_bad_dataflow),

        .out_output_0_payload_discriminant(out_output_0_payload_discriminant),
        .out_output_0_payload_Some_0_a_0(out_output_0_payload_Some_0_a_0),
        
        .out_output_1_0_payload_discriminant(out_output_1_0_payload_discriminant),
        .out_output_1_0_payload_Some_0_b_0(out_output_1_0_payload_Some_0_b_0),
        .out_output_1_0_payload_Some_0_d_0(out_output_1_0_payload_Some_0_d_0),
        
        .out_output_1_1_payload_discriminant(out_output_1_1_payload_discriminant),
        .out_output_1_1_payload_Some_0_id(out_output_1_1_payload_Some_0_id),
        .out_output_1_1_payload_Some_0_last(out_output_1_1_payload_Some_0_last),
        .out_output_1_1_payload_Some_0_control_dataflow_discriminant(out_output_1_1_payload_Some_0_control_dataflow_discriminant),
        .out_output_1_1_payload_Some_0_control_propagate_discriminant(out_output_1_1_payload_Some_0_control_propagate_discriminant),
        .out_output_1_1_payload_Some_0_control_shift(out_output_1_1_payload_Some_0_control_shift),
        .out_output_1_1_payload_Some_0_bad_dataflow(out_output_1_1_payload_Some_0_bad_dataflow)
    );

    assign in_input_0_payload_discriminant = io_in_valid_0;
    assign in_input_0_payload_Some_0_a_0 = io_in_a_0;

    assign in_input_1_0_payload_discriminant = io_in_valid_0;
    assign in_input_1_0_payload_Some_0_b_0 = io_in_b_0;
    assign in_input_1_0_payload_Some_0_d_0 = io_in_d_0;

    assign in_input_1_1_payload_Some_0_id = io_in_id_0;
    assign in_input_1_1_payload_Some_0_last = io_in_last_0;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant = io_in_control_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant = io_in_control_0_propagate;
    assign in_input_1_1_payload_Some_0_control_shift = io_in_control_0_shift;
    assign in_input_1_1_payload_Some_0_bad_dataflow = 1'b0;

    assign io_out_a_0 = out_output_0_payload_Some_0_a_0;

    assign io_out_valid_0 = out_output_0_payload_discriminant && out_output_1_0_payload_discriminant;
    assign io_out_b_0 = out_output_1_0_payload_Some_0_b_0;
    assign io_out_c_0 = out_output_1_0_payload_Some_0_d_0;

    assign io_out_id_0 = out_output_1_1_payload_Some_0_id;
    assign io_out_last_0 = out_output_1_1_payload_Some_0_last;
    assign io_out_control_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant;
    assign io_out_control_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant;
    assign io_out_control_0_shift = out_output_1_1_payload_Some_0_control_shift;
    assign io_bad_dataflow = out_output_1_1_payload_Some_0_bad_dataflow;

endmodule