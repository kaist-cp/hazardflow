module DMACommandTrackerBlackBoxAdapter #(parameter BYTE_WIDTH = 15) (
  input         clock,
                reset,

                io_alloc_valid,
  input  [5:0]  io_alloc_bits_tag_rob_id,
  input  [BYTE_WIDTH-1:0] io_alloc_bits_bytes_to_read,

  input         io_request_returned_valid,
  input  [BYTE_WIDTH-1:0] io_request_returned_bits_bytes_read,
  input         io_request_returned_bits_cmd_id,
                io_cmd_completed_ready,

  output        io_alloc_ready,
                io_alloc_bits_cmd_id,
                io_cmd_completed_valid,
  output [5:0]  io_cmd_completed_bits_tag_rob_id
);
    wire in_input_0_payload_discriminant = io_alloc_valid;
    wire [6-1:0] in_input_0_payload_Some_0_tag = io_alloc_bits_tag_rob_id;
    wire [BYTE_WIDTH-1:0] in_input_0_payload_Some_0_bytes_to_read = io_alloc_bits_bytes_to_read;

    wire in_input_1_payload_discriminant = io_request_returned_valid;
    wire [BYTE_WIDTH-1:0] in_input_1_payload_Some_0_bytes_read = io_request_returned_bits_bytes_read;
    wire in_input_1_payload_Some_0_cmd_id = io_request_returned_bits_cmd_id;
    
    wire out_output_1_resolver_ready = io_cmd_completed_ready;

    wire in_input_0_resolver_ready;
    wire out_output_0_payload_discriminant;
    wire out_output_0_payload_Some_0_cmd_id;
    wire out_output_1_payload_discriminant;
    wire [6-1:0] out_output_1_payload_Some_0_tag;

    dma_command_tracker_default_top dma_command_tracker_default(
        .clk(clock),
        .rst(reset),

        .in_input_0_payload_discriminant(in_input_0_payload_discriminant),
        .in_input_0_payload_Some_0_tag(in_input_0_payload_Some_0_tag),
        .in_input_0_payload_Some_0_bytes_to_read(in_input_0_payload_Some_0_bytes_to_read),
        .in_input_1_payload_discriminant(in_input_1_payload_discriminant),
        .in_input_1_payload_Some_0_bytes_read(in_input_1_payload_Some_0_bytes_read),
        .in_input_1_payload_Some_0_cmd_id(in_input_1_payload_Some_0_cmd_id),
        .out_output_1_resolver_ready(out_output_1_resolver_ready),

        .in_input_0_resolver_ready(in_input_0_resolver_ready),
        .out_output_0_payload_discriminant(out_output_0_payload_discriminant), // not used
        .out_output_0_payload_Some_0_cmd_id(out_output_0_payload_Some_0_cmd_id),
        .out_output_1_payload_discriminant(out_output_1_payload_discriminant),
        .out_output_1_payload_Some_0_tag(out_output_1_payload_Some_0_tag)
    );

    assign io_alloc_ready = in_input_0_resolver_ready;
    assign io_cmd_completed_valid = out_output_1_payload_discriminant;
    assign io_alloc_bits_cmd_id = out_output_0_payload_Some_0_cmd_id;
    assign io_cmd_completed_bits_tag_rob_id = out_output_1_payload_Some_0_tag;
endmodule