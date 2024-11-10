module TransposerWrapper
(
    input wire clk,
    input wire rst,

    input wire in_input_0_payload_discriminant,
    input wire [128-1:0] in_input_0_payload_Some_0_0,

    output wire out_output_payload_discriminant,
    output wire [128-1:0] out_output_payload_Some_0_0
);  
    AlwaysOutTransposer always_out_transposer_inst(
        .clock(clk),
        .reset(rst),

        .io_inRow_valid(in_input_0_payload_discriminant),
        .io_inRow_bits_0(in_input_0_payload_Some_0_0[0*8 +: 8]),
        .io_inRow_bits_1(in_input_0_payload_Some_0_0[1*8 +: 8]),
        .io_inRow_bits_2(in_input_0_payload_Some_0_0[2*8 +: 8]),
        .io_inRow_bits_3(in_input_0_payload_Some_0_0[3*8 +: 8]),
        .io_inRow_bits_4(in_input_0_payload_Some_0_0[4*8 +: 8]),
        .io_inRow_bits_5(in_input_0_payload_Some_0_0[5*8 +: 8]),
        .io_inRow_bits_6(in_input_0_payload_Some_0_0[6*8 +: 8]),
        .io_inRow_bits_7(in_input_0_payload_Some_0_0[7*8 +: 8]),
        .io_inRow_bits_8(in_input_0_payload_Some_0_0[8*8 +: 8]),
        .io_inRow_bits_9(in_input_0_payload_Some_0_0[9*8 +: 8]),
        .io_inRow_bits_10(in_input_0_payload_Some_0_0[10*8 +: 8]),
        .io_inRow_bits_11(in_input_0_payload_Some_0_0[11*8 +: 8]),
        .io_inRow_bits_12(in_input_0_payload_Some_0_0[12*8 +: 8]),
        .io_inRow_bits_13(in_input_0_payload_Some_0_0[13*8 +: 8]),
        .io_inRow_bits_14(in_input_0_payload_Some_0_0[14*8 +: 8]),
        .io_inRow_bits_15(in_input_0_payload_Some_0_0[15*8 +: 8]),

        .io_outCol_bits_0(out_output_payload_Some_0_0[0*8 +: 8]),
        .io_outCol_bits_1(out_output_payload_Some_0_0[1*8 +: 8]),
        .io_outCol_bits_2(out_output_payload_Some_0_0[2*8 +: 8]),
        .io_outCol_bits_3(out_output_payload_Some_0_0[3*8 +: 8]),
        .io_outCol_bits_4(out_output_payload_Some_0_0[4*8 +: 8]),
        .io_outCol_bits_5(out_output_payload_Some_0_0[5*8 +: 8]),
        .io_outCol_bits_6(out_output_payload_Some_0_0[6*8 +: 8]),
        .io_outCol_bits_7(out_output_payload_Some_0_0[7*8 +: 8]),
        .io_outCol_bits_8(out_output_payload_Some_0_0[8*8 +: 8]),
        .io_outCol_bits_9(out_output_payload_Some_0_0[9*8 +: 8]),
        .io_outCol_bits_10(out_output_payload_Some_0_0[10*8 +: 8]),
        .io_outCol_bits_11(out_output_payload_Some_0_0[11*8 +: 8]),
        .io_outCol_bits_12(out_output_payload_Some_0_0[12*8 +: 8]),
        .io_outCol_bits_13(out_output_payload_Some_0_0[13*8 +: 8]),
        .io_outCol_bits_14(out_output_payload_Some_0_0[14*8 +: 8]),
        .io_outCol_bits_15(out_output_payload_Some_0_0[15*8 +: 8])
    );
    
    assign out_output_payload_discriminant = 1'b1;
    
endmodule