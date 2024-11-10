module TransposerBlackBox(
  input        clock,
               reset,
               io_inRow_valid,
  input  [7:0] io_inRow_bits_0,
               io_inRow_bits_1,
               io_inRow_bits_2,
               io_inRow_bits_3,
               io_inRow_bits_4,
               io_inRow_bits_5,
               io_inRow_bits_6,
               io_inRow_bits_7,
               io_inRow_bits_8,
               io_inRow_bits_9,
               io_inRow_bits_10,
               io_inRow_bits_11,
               io_inRow_bits_12,
               io_inRow_bits_13,
               io_inRow_bits_14,
               io_inRow_bits_15,
  output       io_inRow_ready,
               io_outCol_valid,
  output [7:0] io_outCol_bits_0,
               io_outCol_bits_1,
               io_outCol_bits_2,
               io_outCol_bits_3,
               io_outCol_bits_4,
               io_outCol_bits_5,
               io_outCol_bits_6,
               io_outCol_bits_7,
               io_outCol_bits_8,
               io_outCol_bits_9,
               io_outCol_bits_10,
               io_outCol_bits_11,
               io_outCol_bits_12,
               io_outCol_bits_13,
               io_outCol_bits_14,
               io_outCol_bits_15,
  input        io_outCol_ready
);

    wire in_input_0_payload_discriminant;
    wire [128-1:0] in_input_0_payload_Some_0_0;
    wire out_output_payload_discriminant;
    wire [128-1:0] out_output_payload_Some_0_0;

    transposer_default_top transposer_default
    (
        .clk(clock),
        .rst(reset),
        .in_input_0_payload_discriminant(in_input_0_payload_discriminant),
        .in_input_0_payload_Some_0_0(in_input_0_payload_Some_0_0),
        .out_output_payload_discriminant(out_output_payload_discriminant),
        .out_output_payload_Some_0_0(out_output_payload_Some_0_0)
    );

    assign in_input_0_payload_discriminant = io_inRow_valid;
    assign in_input_0_payload_Some_0_0[0 * 8 +: 8] = io_inRow_bits_0;
    assign in_input_0_payload_Some_0_0[1 * 8 +: 8] = io_inRow_bits_1;
    assign in_input_0_payload_Some_0_0[2 * 8 +: 8] = io_inRow_bits_2;
    assign in_input_0_payload_Some_0_0[3 * 8 +: 8] = io_inRow_bits_3;
    assign in_input_0_payload_Some_0_0[4 * 8 +: 8] = io_inRow_bits_4;
    assign in_input_0_payload_Some_0_0[5 * 8 +: 8] = io_inRow_bits_5;
    assign in_input_0_payload_Some_0_0[6 * 8 +: 8] = io_inRow_bits_6;
    assign in_input_0_payload_Some_0_0[7 * 8 +: 8] = io_inRow_bits_7;
    assign in_input_0_payload_Some_0_0[8 * 8 +: 8] = io_inRow_bits_8;
    assign in_input_0_payload_Some_0_0[9 * 8 +: 8] = io_inRow_bits_9;
    assign in_input_0_payload_Some_0_0[10 * 8 +: 8] = io_inRow_bits_10;
    assign in_input_0_payload_Some_0_0[11 * 8 +: 8] = io_inRow_bits_11;
    assign in_input_0_payload_Some_0_0[12 * 8 +: 8] = io_inRow_bits_12;
    assign in_input_0_payload_Some_0_0[13 * 8 +: 8] = io_inRow_bits_13;
    assign in_input_0_payload_Some_0_0[14 * 8 +: 8] = io_inRow_bits_14;
    assign in_input_0_payload_Some_0_0[15 * 8 +: 8] = io_inRow_bits_15;

    assign io_outCol_bits_0 = out_output_payload_Some_0_0[0 * 8 +: 8];
    assign io_outCol_bits_1 = out_output_payload_Some_0_0[1 * 8 +: 8];
    assign io_outCol_bits_2 = out_output_payload_Some_0_0[2 * 8 +: 8];
    assign io_outCol_bits_3 = out_output_payload_Some_0_0[3 * 8 +: 8];
    assign io_outCol_bits_4 = out_output_payload_Some_0_0[4 * 8 +: 8];
    assign io_outCol_bits_5 = out_output_payload_Some_0_0[5 * 8 +: 8];
    assign io_outCol_bits_6 = out_output_payload_Some_0_0[6 * 8 +: 8];
    assign io_outCol_bits_7 = out_output_payload_Some_0_0[7 * 8 +: 8];
    assign io_outCol_bits_8 = out_output_payload_Some_0_0[8 * 8 +: 8];
    assign io_outCol_bits_9 = out_output_payload_Some_0_0[9 * 8 +: 8];
    assign io_outCol_bits_10 = out_output_payload_Some_0_0[10 * 8 +: 8];
    assign io_outCol_bits_11 = out_output_payload_Some_0_0[11 * 8 +: 8];
    assign io_outCol_bits_12 = out_output_payload_Some_0_0[12 * 8 +: 8];
    assign io_outCol_bits_13 = out_output_payload_Some_0_0[13 * 8 +: 8];
    assign io_outCol_bits_14 = out_output_payload_Some_0_0[14 * 8 +: 8];
    assign io_outCol_bits_15 = out_output_payload_Some_0_0[15 * 8 +: 8];

    assign io_inRow_ready = 1'b1;
    assign io_outCol_valid = 1'b1;

endmodule