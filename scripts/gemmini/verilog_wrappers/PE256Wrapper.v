module PE256Wrapper (
    input wire clk,
    input wire rst,
    input wire in_input_0_payload_discriminant,
    input wire [8-1:0] in_input_0_payload_Some_0_a,
    input wire in_input_1_0_payload_discriminant,
    input wire [20-1:0] in_input_1_0_payload_Some_0_b,
    input wire [20-1:0] in_input_1_0_payload_Some_0_d,
    input wire in_input_1_1_payload_discriminant,
    input wire [3-1:0] in_input_1_1_payload_Some_0_id,
    input wire in_input_1_1_payload_Some_0_last,
    input wire in_input_1_1_payload_Some_0_control_dataflow_discriminant,
    input wire in_input_1_1_payload_Some_0_control_propagate_discriminant,
    input wire [5-1:0] in_input_1_1_payload_Some_0_control_shift,
    output wire out_output_0_payload_discriminant,
    output wire [8-1:0] out_output_0_payload_Some_0_a,
    output wire out_output_1_0_payload_discriminant,
    output wire [20-1:0] out_output_1_0_payload_Some_0_b,
    output wire [20-1:0] out_output_1_0_payload_Some_0_d,
    output wire out_output_1_1_payload_discriminant,
    output wire [3-1:0] out_output_1_1_payload_Some_0_id,
    output wire out_output_1_1_payload_Some_0_last,
    output wire out_output_1_1_payload_Some_0_control_dataflow_discriminant,
    output wire out_output_1_1_payload_Some_0_control_propagate_discriminant,
    output wire [5-1:0] out_output_1_1_payload_Some_0_control_shift
);
    wire io_in_valid = in_input_0_payload_discriminant || in_input_1_0_payload_discriminant;
    wire io_out_valid;

    PE_256 pe_256_inner(
        .clock(clk),
        .io_in_a(in_input_0_payload_Some_0_a),
        .io_in_b(in_input_1_0_payload_Some_0_b),
        .io_in_d(in_input_1_0_payload_Some_0_d),
        .io_in_control_dataflow(in_input_1_1_payload_Some_0_control_dataflow_discriminant),
        .io_in_control_propagate(in_input_1_1_payload_Some_0_control_propagate_discriminant),
        .io_in_control_shift(in_input_1_1_payload_Some_0_control_shift),
        .io_in_id(in_input_1_1_payload_Some_0_id),
        .io_in_last(in_input_1_1_payload_Some_0_last),
        .io_in_valid(io_in_valid),

        .io_out_a(out_output_0_payload_Some_0_a),
        .io_out_b(out_output_1_0_payload_Some_0_b),
        .io_out_c(out_output_1_0_payload_Some_0_d),
        .io_out_control_dataflow(out_output_1_1_payload_Some_0_control_dataflow_discriminant),
        .io_out_control_propagate(out_output_1_1_payload_Some_0_control_propagate_discriminant),
        .io_out_control_shift(out_output_1_1_payload_Some_0_control_shift),
        .io_out_id(out_output_1_1_payload_Some_0_id),
        .io_out_last(out_output_1_1_payload_Some_0_last),
        .io_out_valid(io_out_valid)
    );

    assign out_output_0_payload_discriminant = io_out_valid;
    assign out_output_1_0_payload_discriminant = io_out_valid;
    assign out_output_1_1_payload_discriminant = io_out_valid;

endmodule