module MeshBlackBoxAdapter (
  input         clock,
  input  [7:0]  io_in_a_0_0,
                io_in_a_1_0,
                io_in_a_2_0,
                io_in_a_3_0,
                io_in_a_4_0,
                io_in_a_5_0,
                io_in_a_6_0,
                io_in_a_7_0,
                io_in_a_8_0,
                io_in_a_9_0,
                io_in_a_10_0,
                io_in_a_11_0,
                io_in_a_12_0,
                io_in_a_13_0,
                io_in_a_14_0,
                io_in_a_15_0,
                io_in_b_0_0,
                io_in_b_1_0,
                io_in_b_2_0,
                io_in_b_3_0,
                io_in_b_4_0,
                io_in_b_5_0,
                io_in_b_6_0,
                io_in_b_7_0,
                io_in_b_8_0,
                io_in_b_9_0,
                io_in_b_10_0,
                io_in_b_11_0,
                io_in_b_12_0,
                io_in_b_13_0,
                io_in_b_14_0,
                io_in_b_15_0,
                io_in_d_0_0,
                io_in_d_1_0,
                io_in_d_2_0,
                io_in_d_3_0,
                io_in_d_4_0,
                io_in_d_5_0,
                io_in_d_6_0,
                io_in_d_7_0,
                io_in_d_8_0,
                io_in_d_9_0,
                io_in_d_10_0,
                io_in_d_11_0,
                io_in_d_12_0,
                io_in_d_13_0,
                io_in_d_14_0,
                io_in_d_15_0,
  input         io_in_control_0_0_dataflow,
                io_in_control_0_0_propagate,
  input  [4:0]  io_in_control_0_0_shift,
  input         io_in_control_1_0_dataflow,
                io_in_control_1_0_propagate,
  input  [4:0]  io_in_control_1_0_shift,
  input         io_in_control_2_0_dataflow,
                io_in_control_2_0_propagate,
  input  [4:0]  io_in_control_2_0_shift,
  input         io_in_control_3_0_dataflow,
                io_in_control_3_0_propagate,
  input  [4:0]  io_in_control_3_0_shift,
  input         io_in_control_4_0_dataflow,
                io_in_control_4_0_propagate,
  input  [4:0]  io_in_control_4_0_shift,
  input         io_in_control_5_0_dataflow,
                io_in_control_5_0_propagate,
  input  [4:0]  io_in_control_5_0_shift,
  input         io_in_control_6_0_dataflow,
                io_in_control_6_0_propagate,
  input  [4:0]  io_in_control_6_0_shift,
  input         io_in_control_7_0_dataflow,
                io_in_control_7_0_propagate,
  input  [4:0]  io_in_control_7_0_shift,
  input         io_in_control_8_0_dataflow,
                io_in_control_8_0_propagate,
  input  [4:0]  io_in_control_8_0_shift,
  input         io_in_control_9_0_dataflow,
                io_in_control_9_0_propagate,
  input  [4:0]  io_in_control_9_0_shift,
  input         io_in_control_10_0_dataflow,
                io_in_control_10_0_propagate,
  input  [4:0]  io_in_control_10_0_shift,
  input         io_in_control_11_0_dataflow,
                io_in_control_11_0_propagate,
  input  [4:0]  io_in_control_11_0_shift,
  input         io_in_control_12_0_dataflow,
                io_in_control_12_0_propagate,
  input  [4:0]  io_in_control_12_0_shift,
  input         io_in_control_13_0_dataflow,
                io_in_control_13_0_propagate,
  input  [4:0]  io_in_control_13_0_shift,
  input         io_in_control_14_0_dataflow,
                io_in_control_14_0_propagate,
  input  [4:0]  io_in_control_14_0_shift,
  input         io_in_control_15_0_dataflow,
                io_in_control_15_0_propagate,
  input  [4:0]  io_in_control_15_0_shift,
  input  [2:0]  io_in_id_0_0,
                io_in_id_1_0,
                io_in_id_2_0,
                io_in_id_3_0,
                io_in_id_4_0,
                io_in_id_5_0,
                io_in_id_6_0,
                io_in_id_7_0,
                io_in_id_8_0,
                io_in_id_9_0,
                io_in_id_10_0,
                io_in_id_11_0,
                io_in_id_12_0,
                io_in_id_13_0,
                io_in_id_14_0,
                io_in_id_15_0,
  input         io_in_last_0_0,
                io_in_last_1_0,
                io_in_last_2_0,
                io_in_last_3_0,
                io_in_last_4_0,
                io_in_last_5_0,
                io_in_last_6_0,
                io_in_last_7_0,
                io_in_last_8_0,
                io_in_last_9_0,
                io_in_last_10_0,
                io_in_last_11_0,
                io_in_last_12_0,
                io_in_last_13_0,
                io_in_last_14_0,
                io_in_last_15_0,
                io_in_valid_0_0,
                io_in_valid_1_0,
                io_in_valid_2_0,
                io_in_valid_3_0,
                io_in_valid_4_0,
                io_in_valid_5_0,
                io_in_valid_6_0,
                io_in_valid_7_0,
                io_in_valid_8_0,
                io_in_valid_9_0,
                io_in_valid_10_0,
                io_in_valid_11_0,
                io_in_valid_12_0,
                io_in_valid_13_0,
                io_in_valid_14_0,
                io_in_valid_15_0,
  output [19:0] io_out_b_0_0,
                io_out_b_1_0,
                io_out_b_2_0,
                io_out_b_3_0,
                io_out_b_4_0,
                io_out_b_5_0,
                io_out_b_6_0,
                io_out_b_7_0,
                io_out_b_8_0,
                io_out_b_9_0,
                io_out_b_10_0,
                io_out_b_11_0,
                io_out_b_12_0,
                io_out_b_13_0,
                io_out_b_14_0,
                io_out_b_15_0,
                io_out_c_0_0,
                io_out_c_1_0,
                io_out_c_2_0,
                io_out_c_3_0,
                io_out_c_4_0,
                io_out_c_5_0,
                io_out_c_6_0,
                io_out_c_7_0,
                io_out_c_8_0,
                io_out_c_9_0,
                io_out_c_10_0,
                io_out_c_11_0,
                io_out_c_12_0,
                io_out_c_13_0,
                io_out_c_14_0,
                io_out_c_15_0,
  output        io_out_valid_0_0,
                io_out_valid_1_0,
                io_out_valid_2_0,
                io_out_valid_3_0,
                io_out_valid_4_0,
                io_out_valid_5_0,
                io_out_valid_6_0,
                io_out_valid_7_0,
                io_out_valid_8_0,
                io_out_valid_9_0,
                io_out_valid_10_0,
                io_out_valid_11_0,
                io_out_valid_12_0,
                io_out_valid_13_0,
                io_out_valid_14_0,
                io_out_valid_15_0,
                io_out_control_0_0_dataflow,
                io_out_control_1_0_dataflow,
                io_out_control_2_0_dataflow,
                io_out_control_3_0_dataflow,
                io_out_control_4_0_dataflow,
                io_out_control_5_0_dataflow,
                io_out_control_6_0_dataflow,
                io_out_control_7_0_dataflow,
                io_out_control_8_0_dataflow,
                io_out_control_9_0_dataflow,
                io_out_control_10_0_dataflow,
                io_out_control_11_0_dataflow,
                io_out_control_12_0_dataflow,
                io_out_control_13_0_dataflow,
                io_out_control_14_0_dataflow,
                io_out_control_15_0_dataflow,
                io_out_control_0_0_propagate,
                io_out_control_1_0_propagate,
                io_out_control_2_0_propagate,
                io_out_control_3_0_propagate,
                io_out_control_4_0_propagate,
                io_out_control_5_0_propagate,
                io_out_control_6_0_propagate,
                io_out_control_7_0_propagate,
                io_out_control_8_0_propagate,
                io_out_control_9_0_propagate,
                io_out_control_10_0_propagate,
                io_out_control_11_0_propagate,
                io_out_control_12_0_propagate,
                io_out_control_13_0_propagate,
                io_out_control_14_0_propagate,
                io_out_control_15_0_propagate,
  output [4:0]  io_out_control_0_0_shift,
                io_out_control_1_0_shift,
                io_out_control_2_0_shift,
                io_out_control_3_0_shift,
                io_out_control_4_0_shift,
                io_out_control_5_0_shift,
                io_out_control_6_0_shift,
                io_out_control_7_0_shift,
                io_out_control_8_0_shift,
                io_out_control_9_0_shift,
                io_out_control_10_0_shift,
                io_out_control_11_0_shift,
                io_out_control_12_0_shift,
                io_out_control_13_0_shift,
                io_out_control_14_0_shift,
                io_out_control_15_0_shift,
  output [2:0]  io_out_id_0_0,
                io_out_id_1_0,
                io_out_id_2_0,
                io_out_id_3_0,
                io_out_id_4_0,
                io_out_id_5_0,
                io_out_id_6_0,
                io_out_id_7_0,
                io_out_id_8_0,
                io_out_id_9_0,
                io_out_id_10_0,
                io_out_id_11_0,
                io_out_id_12_0,
                io_out_id_13_0,
                io_out_id_14_0,
                io_out_id_15_0,
  output        io_out_last_0_0,
                io_out_last_1_0,
                io_out_last_2_0,
                io_out_last_3_0,
                io_out_last_4_0,
                io_out_last_5_0,
                io_out_last_6_0,
                io_out_last_7_0,
                io_out_last_8_0,
                io_out_last_9_0,
                io_out_last_10_0,
                io_out_last_11_0,
                io_out_last_12_0,
                io_out_last_13_0,
                io_out_last_14_0,
                io_out_last_15_0
);

    wire [16-1:0] in_input_0_payload_discriminant;
    wire [128-1:0] in_input_0_payload_Some_0_a_0;
    wire [16-1:0] in_input_1_0_payload_discriminant;
    wire [320-1:0] in_input_1_0_payload_Some_0_b_0;
    wire [320-1:0] in_input_1_0_payload_Some_0_d_0;
    wire [16-1:0] in_input_1_1_payload_discriminant = 16'hFFFF;
    wire [48-1:0] in_input_1_1_payload_Some_0_id;
    wire [16-1:0] in_input_1_1_payload_Some_0_last;
    wire [16-1:0] in_input_1_1_payload_Some_0_control_dataflow_discriminant;
    wire [16-1:0] in_input_1_1_payload_Some_0_control_propagate_discriminant;
    wire [80-1:0] in_input_1_1_payload_Some_0_control_shift;
    
    wire [16-1:0] out_output_0_payload_discriminant;
    wire [128-1:0] out_output_0_payload_Some_0_a_0;
    wire [16-1:0] out_output_1_0_payload_discriminant;
    wire [320-1:0] out_output_1_0_payload_Some_0_b_0;
    wire [320-1:0] out_output_1_0_payload_Some_0_d_0;
    wire [16-1:0] out_output_1_1_payload_discriminant;
    wire [48-1:0] out_output_1_1_payload_Some_0_id;
    wire [16-1:0] out_output_1_1_payload_Some_0_last;
    wire [16-1:0] out_output_1_1_payload_Some_0_control_dataflow_discriminant;
    wire [16-1:0] out_output_1_1_payload_Some_0_control_propagate_discriminant;
    wire [80-1:0] out_output_1_1_payload_Some_0_control_shift;

    mesh_default_top mesh_default
    (
        .clk(clock),
        .rst(1'b0),
        .in_input_0_payload_discriminant(in_input_0_payload_discriminant),
        .in_input_0_payload_Some_0_a_0(in_input_0_payload_Some_0_a_0),
        .in_input_1_0_payload_discriminant(in_input_1_0_payload_discriminant),
        .in_input_1_0_payload_Some_0_b_0(in_input_1_0_payload_Some_0_b_0),
        .in_input_1_0_payload_Some_0_d_0(in_input_1_0_payload_Some_0_d_0),
        .in_input_1_1_payload_discriminant(in_input_1_1_payload_discriminant),
        .in_input_1_1_payload_Some_0_id(in_input_1_1_payload_Some_0_id),
        .in_input_1_1_payload_Some_0_last(in_input_1_1_payload_Some_0_last),
        .in_input_1_1_payload_Some_0_control_dataflow_discriminant(in_input_1_1_payload_Some_0_control_dataflow_discriminant),
        .in_input_1_1_payload_Some_0_control_propagate_discriminant(in_input_1_1_payload_Some_0_control_propagate_discriminant),
        .in_input_1_1_payload_Some_0_control_shift(in_input_1_1_payload_Some_0_control_shift),
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
        .out_output_1_1_payload_Some_0_control_shift(out_output_1_1_payload_Some_0_control_shift)
    );

    // assign io_in_a
    assign in_input_0_payload_Some_0_a_0[0*8 +: 8] = io_in_a_0_0;
    assign in_input_0_payload_Some_0_a_0[1*8 +: 8] = io_in_a_1_0;
    assign in_input_0_payload_Some_0_a_0[2*8 +: 8] = io_in_a_2_0;
    assign in_input_0_payload_Some_0_a_0[3*8 +: 8] = io_in_a_3_0;
    assign in_input_0_payload_Some_0_a_0[4*8 +: 8] = io_in_a_4_0;
    assign in_input_0_payload_Some_0_a_0[5*8 +: 8] = io_in_a_5_0;
    assign in_input_0_payload_Some_0_a_0[6*8 +: 8] = io_in_a_6_0;
    assign in_input_0_payload_Some_0_a_0[7*8 +: 8] = io_in_a_7_0;
    assign in_input_0_payload_Some_0_a_0[8*8 +: 8] = io_in_a_8_0;
    assign in_input_0_payload_Some_0_a_0[9*8 +: 8] = io_in_a_9_0;
    assign in_input_0_payload_Some_0_a_0[10*8 +: 8] = io_in_a_10_0;
    assign in_input_0_payload_Some_0_a_0[11*8 +: 8] = io_in_a_11_0;
    assign in_input_0_payload_Some_0_a_0[12*8 +: 8] = io_in_a_12_0;
    assign in_input_0_payload_Some_0_a_0[13*8 +: 8] = io_in_a_13_0;
    assign in_input_0_payload_Some_0_a_0[14*8 +: 8] = io_in_a_14_0;
    assign in_input_0_payload_Some_0_a_0[15*8 +: 8] = io_in_a_15_0;


    // assign io_in_b
    assign in_input_1_0_payload_Some_0_b_0[0*20 +: 20] = { {12{io_in_b_0_0[7]}}, io_in_b_0_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[1*20 +: 20] = { {12{io_in_b_1_0[7]}}, io_in_b_1_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[2*20 +: 20] = { {12{io_in_b_2_0[7]}}, io_in_b_2_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[3*20 +: 20] = { {12{io_in_b_3_0[7]}}, io_in_b_3_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[4*20 +: 20] = { {12{io_in_b_4_0[7]}}, io_in_b_4_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[5*20 +: 20] = { {12{io_in_b_5_0[7]}}, io_in_b_5_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[6*20 +: 20] = { {12{io_in_b_6_0[7]}}, io_in_b_6_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[7*20 +: 20] = { {12{io_in_b_7_0[7]}}, io_in_b_7_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[8*20 +: 20] = { {12{io_in_b_8_0[7]}}, io_in_b_8_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[9*20 +: 20] = { {12{io_in_b_9_0[7]}}, io_in_b_9_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[10*20 +: 20] = { {12{io_in_b_10_0[7]}}, io_in_b_10_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[11*20 +: 20] = { {12{io_in_b_11_0[7]}}, io_in_b_11_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[12*20 +: 20] = { {12{io_in_b_12_0[7]}}, io_in_b_12_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[13*20 +: 20] = { {12{io_in_b_13_0[7]}}, io_in_b_13_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[14*20 +: 20] = { {12{io_in_b_14_0[7]}}, io_in_b_14_0[7:0] };
    assign in_input_1_0_payload_Some_0_b_0[15*20 +: 20] = { {12{io_in_b_15_0[7]}}, io_in_b_15_0[7:0] };

    // assign io_in_d
    assign in_input_1_0_payload_Some_0_d_0[0*20 +: 20] = { {12{io_in_d_0_0[7]}}, io_in_d_0_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[1*20 +: 20] = { {12{io_in_d_1_0[7]}}, io_in_d_1_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[2*20 +: 20] = { {12{io_in_d_2_0[7]}}, io_in_d_2_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[3*20 +: 20] = { {12{io_in_d_3_0[7]}}, io_in_d_3_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[4*20 +: 20] = { {12{io_in_d_4_0[7]}}, io_in_d_4_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[5*20 +: 20] = { {12{io_in_d_5_0[7]}}, io_in_d_5_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[6*20 +: 20] = { {12{io_in_d_6_0[7]}}, io_in_d_6_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[7*20 +: 20] = { {12{io_in_d_7_0[7]}}, io_in_d_7_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[8*20 +: 20] = { {12{io_in_d_8_0[7]}}, io_in_d_8_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[9*20 +: 20] = { {12{io_in_d_9_0[7]}}, io_in_d_9_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[10*20 +: 20] = { {12{io_in_d_10_0[7]}}, io_in_d_10_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[11*20 +: 20] = { {12{io_in_d_11_0[7]}}, io_in_d_11_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[12*20 +: 20] = { {12{io_in_d_12_0[7]}}, io_in_d_12_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[13*20 +: 20] = { {12{io_in_d_13_0[7]}}, io_in_d_13_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[14*20 +: 20] = { {12{io_in_d_14_0[7]}}, io_in_d_14_0[7:0] };
    assign in_input_1_0_payload_Some_0_d_0[15*20 +: 20] = { {12{io_in_d_15_0[7]}}, io_in_d_15_0[7:0] };

    // assign io_in_control_dataflow
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[0] = io_in_control_0_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[1] = io_in_control_1_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[2] = io_in_control_2_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[3] = io_in_control_3_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[4] = io_in_control_4_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[5] = io_in_control_5_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[6] = io_in_control_6_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[7] = io_in_control_7_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[8] = io_in_control_8_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[9] = io_in_control_9_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[10] = io_in_control_10_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[11] = io_in_control_11_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[12] = io_in_control_12_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[13] = io_in_control_13_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[14] = io_in_control_14_0_dataflow;
    assign in_input_1_1_payload_Some_0_control_dataflow_discriminant[15] = io_in_control_15_0_dataflow;

    // assign io_in_control_propagate
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[0] = io_in_control_0_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[1] = io_in_control_1_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[2] = io_in_control_2_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[3] = io_in_control_3_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[4] = io_in_control_4_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[5] = io_in_control_5_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[6] = io_in_control_6_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[7] = io_in_control_7_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[8] = io_in_control_8_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[9] = io_in_control_9_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[10] = io_in_control_10_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[11] = io_in_control_11_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[12] = io_in_control_12_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[13] = io_in_control_13_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[14] = io_in_control_14_0_propagate;
    assign in_input_1_1_payload_Some_0_control_propagate_discriminant[15] = io_in_control_15_0_propagate;

    // assign io_in_control_shift
    assign in_input_1_1_payload_Some_0_control_shift[0*5 +: 5] = io_in_control_0_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[1*5 +: 5] = io_in_control_1_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[2*5 +: 5] = io_in_control_2_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[3*5 +: 5] = io_in_control_3_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[4*5 +: 5] = io_in_control_4_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[5*5 +: 5] = io_in_control_5_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[6*5 +: 5] = io_in_control_6_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[7*5 +: 5] = io_in_control_7_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[8*5 +: 5] = io_in_control_8_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[9*5 +: 5] = io_in_control_9_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[10*5 +: 5] = io_in_control_10_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[11*5 +: 5] = io_in_control_11_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[12*5 +: 5] = io_in_control_12_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[13*5 +: 5] = io_in_control_13_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[14*5 +: 5] = io_in_control_14_0_shift;
    assign in_input_1_1_payload_Some_0_control_shift[15*5 +: 5] = io_in_control_15_0_shift;

    // assign io_in_id
    assign in_input_1_1_payload_Some_0_id[0*3 +: 3] = io_in_id_0_0;
    assign in_input_1_1_payload_Some_0_id[1*3 +: 3] = io_in_id_1_0;
    assign in_input_1_1_payload_Some_0_id[2*3 +: 3] = io_in_id_2_0;
    assign in_input_1_1_payload_Some_0_id[3*3 +: 3] = io_in_id_3_0;
    assign in_input_1_1_payload_Some_0_id[4*3 +: 3] = io_in_id_4_0;
    assign in_input_1_1_payload_Some_0_id[5*3 +: 3] = io_in_id_5_0;
    assign in_input_1_1_payload_Some_0_id[6*3 +: 3] = io_in_id_6_0;
    assign in_input_1_1_payload_Some_0_id[7*3 +: 3] = io_in_id_7_0;
    assign in_input_1_1_payload_Some_0_id[8*3 +: 3] = io_in_id_8_0;
    assign in_input_1_1_payload_Some_0_id[9*3 +: 3] = io_in_id_9_0;
    assign in_input_1_1_payload_Some_0_id[10*3 +: 3] = io_in_id_10_0;
    assign in_input_1_1_payload_Some_0_id[11*3 +: 3] = io_in_id_11_0;
    assign in_input_1_1_payload_Some_0_id[12*3 +: 3] = io_in_id_12_0;
    assign in_input_1_1_payload_Some_0_id[13*3 +: 3] = io_in_id_13_0;
    assign in_input_1_1_payload_Some_0_id[14*3 +: 3] = io_in_id_14_0;
    assign in_input_1_1_payload_Some_0_id[15*3 +: 3] = io_in_id_15_0;

    // assign io_in_last
    assign in_input_1_1_payload_Some_0_last[0] = io_in_last_0_0;
    assign in_input_1_1_payload_Some_0_last[1] = io_in_last_1_0;
    assign in_input_1_1_payload_Some_0_last[2] = io_in_last_2_0;
    assign in_input_1_1_payload_Some_0_last[3] = io_in_last_3_0;
    assign in_input_1_1_payload_Some_0_last[4] = io_in_last_4_0;
    assign in_input_1_1_payload_Some_0_last[5] = io_in_last_5_0;
    assign in_input_1_1_payload_Some_0_last[6] = io_in_last_6_0;
    assign in_input_1_1_payload_Some_0_last[7] = io_in_last_7_0;
    assign in_input_1_1_payload_Some_0_last[8] = io_in_last_8_0;
    assign in_input_1_1_payload_Some_0_last[9] = io_in_last_9_0;
    assign in_input_1_1_payload_Some_0_last[10] = io_in_last_10_0;
    assign in_input_1_1_payload_Some_0_last[11] = io_in_last_11_0;
    assign in_input_1_1_payload_Some_0_last[12] = io_in_last_12_0;
    assign in_input_1_1_payload_Some_0_last[13] = io_in_last_13_0;
    assign in_input_1_1_payload_Some_0_last[14] = io_in_last_14_0;
    assign in_input_1_1_payload_Some_0_last[15] = io_in_last_15_0;

    // assign io_in_valid
    assign in_input_0_payload_discriminant[0] = io_in_valid_0_0;
    assign in_input_0_payload_discriminant[1] = io_in_valid_1_0;
    assign in_input_0_payload_discriminant[2] = io_in_valid_2_0;
    assign in_input_0_payload_discriminant[3] = io_in_valid_3_0;
    assign in_input_0_payload_discriminant[4] = io_in_valid_4_0;
    assign in_input_0_payload_discriminant[5] = io_in_valid_5_0;
    assign in_input_0_payload_discriminant[6] = io_in_valid_6_0;
    assign in_input_0_payload_discriminant[7] = io_in_valid_7_0;
    assign in_input_0_payload_discriminant[8] = io_in_valid_8_0;
    assign in_input_0_payload_discriminant[9] = io_in_valid_9_0;
    assign in_input_0_payload_discriminant[10] = io_in_valid_10_0;
    assign in_input_0_payload_discriminant[11] = io_in_valid_11_0;
    assign in_input_0_payload_discriminant[12] = io_in_valid_12_0;
    assign in_input_0_payload_discriminant[13] = io_in_valid_13_0;
    assign in_input_0_payload_discriminant[14] = io_in_valid_14_0;
    assign in_input_0_payload_discriminant[15] = io_in_valid_15_0;

    assign in_input_1_0_payload_discriminant[0] = io_in_valid_0_0;
    assign in_input_1_0_payload_discriminant[1] = io_in_valid_1_0;
    assign in_input_1_0_payload_discriminant[2] = io_in_valid_2_0;
    assign in_input_1_0_payload_discriminant[3] = io_in_valid_3_0;
    assign in_input_1_0_payload_discriminant[4] = io_in_valid_4_0;
    assign in_input_1_0_payload_discriminant[5] = io_in_valid_5_0;
    assign in_input_1_0_payload_discriminant[6] = io_in_valid_6_0;
    assign in_input_1_0_payload_discriminant[7] = io_in_valid_7_0;
    assign in_input_1_0_payload_discriminant[8] = io_in_valid_8_0;
    assign in_input_1_0_payload_discriminant[9] = io_in_valid_9_0;
    assign in_input_1_0_payload_discriminant[10] = io_in_valid_10_0;
    assign in_input_1_0_payload_discriminant[11] = io_in_valid_11_0;
    assign in_input_1_0_payload_discriminant[12] = io_in_valid_12_0;
    assign in_input_1_0_payload_discriminant[13] = io_in_valid_13_0;
    assign in_input_1_0_payload_discriminant[14] = io_in_valid_14_0;
    assign in_input_1_0_payload_discriminant[15] = io_in_valid_15_0;

    // assign io_out_b
    assign io_out_b_0_0 = out_output_1_0_payload_Some_0_b_0[0*20 +: 20];
    assign io_out_b_1_0 = out_output_1_0_payload_Some_0_b_0[1*20 +: 20];
    assign io_out_b_2_0 = out_output_1_0_payload_Some_0_b_0[2*20 +: 20];
    assign io_out_b_3_0 = out_output_1_0_payload_Some_0_b_0[3*20 +: 20];
    assign io_out_b_4_0 = out_output_1_0_payload_Some_0_b_0[4*20 +: 20];
    assign io_out_b_5_0 = out_output_1_0_payload_Some_0_b_0[5*20 +: 20];
    assign io_out_b_6_0 = out_output_1_0_payload_Some_0_b_0[6*20 +: 20];
    assign io_out_b_7_0 = out_output_1_0_payload_Some_0_b_0[7*20 +: 20];
    assign io_out_b_8_0 = out_output_1_0_payload_Some_0_b_0[8*20 +: 20];
    assign io_out_b_9_0 = out_output_1_0_payload_Some_0_b_0[9*20 +: 20];
    assign io_out_b_10_0 = out_output_1_0_payload_Some_0_b_0[10*20 +: 20];
    assign io_out_b_11_0 = out_output_1_0_payload_Some_0_b_0[11*20 +: 20];
    assign io_out_b_12_0 = out_output_1_0_payload_Some_0_b_0[12*20 +: 20];
    assign io_out_b_13_0 = out_output_1_0_payload_Some_0_b_0[13*20 +: 20];
    assign io_out_b_14_0 = out_output_1_0_payload_Some_0_b_0[14*20 +: 20];
    assign io_out_b_15_0 = out_output_1_0_payload_Some_0_b_0[15*20 +: 20];

    // assign io_out_c
    assign io_out_c_0_0 = out_output_1_0_payload_Some_0_d_0[0*20 +: 20];
    assign io_out_c_1_0 = out_output_1_0_payload_Some_0_d_0[1*20 +: 20];
    assign io_out_c_2_0 = out_output_1_0_payload_Some_0_d_0[2*20 +: 20];
    assign io_out_c_3_0 = out_output_1_0_payload_Some_0_d_0[3*20 +: 20];
    assign io_out_c_4_0 = out_output_1_0_payload_Some_0_d_0[4*20 +: 20];
    assign io_out_c_5_0 = out_output_1_0_payload_Some_0_d_0[5*20 +: 20];
    assign io_out_c_6_0 = out_output_1_0_payload_Some_0_d_0[6*20 +: 20];
    assign io_out_c_7_0 = out_output_1_0_payload_Some_0_d_0[7*20 +: 20];
    assign io_out_c_8_0 = out_output_1_0_payload_Some_0_d_0[8*20 +: 20];
    assign io_out_c_9_0 = out_output_1_0_payload_Some_0_d_0[9*20 +: 20];
    assign io_out_c_10_0 = out_output_1_0_payload_Some_0_d_0[10*20 +: 20];
    assign io_out_c_11_0 = out_output_1_0_payload_Some_0_d_0[11*20 +: 20];
    assign io_out_c_12_0 = out_output_1_0_payload_Some_0_d_0[12*20 +: 20];
    assign io_out_c_13_0 = out_output_1_0_payload_Some_0_d_0[13*20 +: 20];
    assign io_out_c_14_0 = out_output_1_0_payload_Some_0_d_0[14*20 +: 20];
    assign io_out_c_15_0 = out_output_1_0_payload_Some_0_d_0[15*20 +: 20];

    // // assign io_out_valid
    assign io_out_valid_0_0 = out_output_1_0_payload_discriminant[0] & out_output_0_payload_discriminant[0];
    assign io_out_valid_1_0 = out_output_1_0_payload_discriminant[1] & out_output_0_payload_discriminant[1];
    assign io_out_valid_2_0 = out_output_1_0_payload_discriminant[2] & out_output_0_payload_discriminant[2];
    assign io_out_valid_3_0 = out_output_1_0_payload_discriminant[3] & out_output_0_payload_discriminant[3];
    assign io_out_valid_4_0 = out_output_1_0_payload_discriminant[4] & out_output_0_payload_discriminant[4];
    assign io_out_valid_5_0 = out_output_1_0_payload_discriminant[5] & out_output_0_payload_discriminant[5];
    assign io_out_valid_6_0 = out_output_1_0_payload_discriminant[6] & out_output_0_payload_discriminant[6];
    assign io_out_valid_7_0 = out_output_1_0_payload_discriminant[7] & out_output_0_payload_discriminant[7];
    assign io_out_valid_8_0 = out_output_1_0_payload_discriminant[8] & out_output_0_payload_discriminant[8];
    assign io_out_valid_9_0 = out_output_1_0_payload_discriminant[9] & out_output_0_payload_discriminant[9];
    assign io_out_valid_10_0 = out_output_1_0_payload_discriminant[10] & out_output_0_payload_discriminant[10];
    assign io_out_valid_11_0 = out_output_1_0_payload_discriminant[11] & out_output_0_payload_discriminant[11];
    assign io_out_valid_12_0 = out_output_1_0_payload_discriminant[12] & out_output_0_payload_discriminant[12];
    assign io_out_valid_13_0 = out_output_1_0_payload_discriminant[13] & out_output_0_payload_discriminant[13];
    assign io_out_valid_14_0 = out_output_1_0_payload_discriminant[14] & out_output_0_payload_discriminant[14];
    assign io_out_valid_15_0 = out_output_1_0_payload_discriminant[15] & out_output_0_payload_discriminant[15];
    // assign io_out_valid_0_0 = out_output_1_0_payload_discriminant[0] & out_output_0_payload_discriminant[0];
    // always @(posedge clock) begin
    //   if ((out_output_1_0_payload_discriminant != 16'h00) || (out_output_1_0_payload_discriminant != 16'hFF)) begin
    //     $error("[%t] Check out_output_1_0_payload_discriminant", $time);
    //   end
    //   if ((out_output_0_payload_discriminant != 16'h00) || (out_output_0_payload_discriminant != 16'hFF)) begin
    //     $error("[%t] Check out_output_0_payload_discriminant", $time);
    //   end
    //   if (out_output_0_payload_discriminant != out_output_1_0_payload_discriminant) begin
    //     $error("[%t] out_output_0_payload_discriminant and out_output_1_0_payload_discriminant doesn't match", $time);
    //   end
    // end

    // // assign io_out_control_0_0_dataflow
    assign io_out_control_0_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[0];
    assign io_out_control_1_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[1];
    assign io_out_control_2_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[2];
    assign io_out_control_3_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[3];
    assign io_out_control_4_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[4];
    assign io_out_control_5_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[5];
    assign io_out_control_6_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[6];
    assign io_out_control_7_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[7];
    assign io_out_control_8_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[8];
    assign io_out_control_9_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[9];
    assign io_out_control_10_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[10];
    assign io_out_control_11_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[11];
    assign io_out_control_12_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[12];
    assign io_out_control_13_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[13];
    assign io_out_control_14_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[14];
    assign io_out_control_15_0_dataflow = out_output_1_1_payload_Some_0_control_dataflow_discriminant[15];

    assign io_out_control_0_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[0];
    assign io_out_control_1_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[1];
    assign io_out_control_2_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[2];
    assign io_out_control_3_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[3];
    assign io_out_control_4_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[4];
    assign io_out_control_5_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[5];
    assign io_out_control_6_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[6];
    assign io_out_control_7_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[7];
    assign io_out_control_8_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[8];
    assign io_out_control_9_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[9];
    assign io_out_control_10_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[10];
    assign io_out_control_11_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[11];
    assign io_out_control_12_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[12];
    assign io_out_control_13_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[13];
    assign io_out_control_14_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[14];
    assign io_out_control_15_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[15];

    assign io_out_control_0_0_shift = out_output_1_1_payload_Some_0_control_shift[0*5 +: 5];
    assign io_out_control_1_0_shift = out_output_1_1_payload_Some_0_control_shift[1*5 +: 5];
    assign io_out_control_2_0_shift = out_output_1_1_payload_Some_0_control_shift[2*5 +: 5];
    assign io_out_control_3_0_shift = out_output_1_1_payload_Some_0_control_shift[3*5 +: 5];
    assign io_out_control_4_0_shift = out_output_1_1_payload_Some_0_control_shift[4*5 +: 5];
    assign io_out_control_5_0_shift = out_output_1_1_payload_Some_0_control_shift[5*5 +: 5];
    assign io_out_control_6_0_shift = out_output_1_1_payload_Some_0_control_shift[6*5 +: 5];
    assign io_out_control_7_0_shift = out_output_1_1_payload_Some_0_control_shift[7*5 +: 5];
    assign io_out_control_8_0_shift = out_output_1_1_payload_Some_0_control_shift[8*5 +: 5];
    assign io_out_control_9_0_shift = out_output_1_1_payload_Some_0_control_shift[9*5 +: 5];
    assign io_out_control_10_0_shift = out_output_1_1_payload_Some_0_control_shift[10*5 +: 5];
    assign io_out_control_11_0_shift = out_output_1_1_payload_Some_0_control_shift[11*5 +: 5];
    assign io_out_control_12_0_shift = out_output_1_1_payload_Some_0_control_shift[12*5 +: 5];
    assign io_out_control_13_0_shift = out_output_1_1_payload_Some_0_control_shift[13*5 +: 5];
    assign io_out_control_14_0_shift = out_output_1_1_payload_Some_0_control_shift[14*5 +: 5];
    assign io_out_control_15_0_shift = out_output_1_1_payload_Some_0_control_shift[15*5 +: 5];
    // assign io_out_control_0_0_propagate = out_output_1_1_payload_Some_0_control_propagate_discriminant[0];
    // always @(posedge clock) begin
    //   if ((out_output_1_1_payload_Some_0_control_dataflow_discriminant != 16'h00) || (out_output_1_1_payload_Some_0_control_dataflow_discriminant != 16'hFF)) begin
    //     $error("[%t] Check out_output_1_1_payload_Some_0_control_dataflow_discriminant", $time);
    //   end
    // end

    // // assign io_out_id
    assign io_out_id_0_0 = out_output_1_1_payload_Some_0_id[0*3 +: 3];
    assign io_out_id_1_0 = out_output_1_1_payload_Some_0_id[1*3 +: 3];
    assign io_out_id_2_0 = out_output_1_1_payload_Some_0_id[2*3 +: 3];
    assign io_out_id_3_0 = out_output_1_1_payload_Some_0_id[3*3 +: 3];
    assign io_out_id_4_0 = out_output_1_1_payload_Some_0_id[4*3 +: 3];
    assign io_out_id_5_0 = out_output_1_1_payload_Some_0_id[5*3 +: 3];
    assign io_out_id_6_0 = out_output_1_1_payload_Some_0_id[6*3 +: 3];
    assign io_out_id_7_0 = out_output_1_1_payload_Some_0_id[7*3 +: 3];
    assign io_out_id_8_0 = out_output_1_1_payload_Some_0_id[8*3 +: 3];
    assign io_out_id_9_0 = out_output_1_1_payload_Some_0_id[9*3 +: 3];
    assign io_out_id_10_0 = out_output_1_1_payload_Some_0_id[10*3 +: 3];
    assign io_out_id_11_0 = out_output_1_1_payload_Some_0_id[11*3 +: 3];
    assign io_out_id_12_0 = out_output_1_1_payload_Some_0_id[12*3 +: 3];
    assign io_out_id_13_0 = out_output_1_1_payload_Some_0_id[13*3 +: 3];
    assign io_out_id_14_0 = out_output_1_1_payload_Some_0_id[14*3 +: 3];
    assign io_out_id_15_0 = out_output_1_1_payload_Some_0_id[15*3 +: 3];
    // assign io_out_id_0_0 = out_output_1_1_payload_Some_0_id[0 +: 3];

    // // assign io_out_last
    assign io_out_last_0_0 = out_output_1_1_payload_Some_0_last[0];
    assign io_out_last_1_0 = out_output_1_1_payload_Some_0_last[1];
    assign io_out_last_2_0 = out_output_1_1_payload_Some_0_last[2];
    assign io_out_last_3_0 = out_output_1_1_payload_Some_0_last[3];
    assign io_out_last_4_0 = out_output_1_1_payload_Some_0_last[4];
    assign io_out_last_5_0 = out_output_1_1_payload_Some_0_last[5];
    assign io_out_last_6_0 = out_output_1_1_payload_Some_0_last[6];
    assign io_out_last_7_0 = out_output_1_1_payload_Some_0_last[7];
    assign io_out_last_8_0 = out_output_1_1_payload_Some_0_last[8];
    assign io_out_last_9_0 = out_output_1_1_payload_Some_0_last[9];
    assign io_out_last_10_0 = out_output_1_1_payload_Some_0_last[10];
    assign io_out_last_11_0 = out_output_1_1_payload_Some_0_last[11];
    assign io_out_last_12_0 = out_output_1_1_payload_Some_0_last[12];
    assign io_out_last_13_0 = out_output_1_1_payload_Some_0_last[13];
    assign io_out_last_14_0 = out_output_1_1_payload_Some_0_last[14];
    assign io_out_last_15_0 = out_output_1_1_payload_Some_0_last[15];
    // assign io_out_last_0_0 = out_output_1_1_payload_Some_0_last[0];
    // always @(posedge clock) begin
    //   if ((out_output_1_1_payload_Some_0_last != 16'h00) || (out_output_1_1_payload_Some_0_last != 16'hFF)) begin
    //     $error("[%t] Check out_output_1_1_payload_Some_0_last", $time);
    //   end
    // end

endmodule