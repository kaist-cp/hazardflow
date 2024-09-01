import logging
import random
import os, sys

import numpy as np

import cocotb
from cocotb.clock import Clock
from cocotb.triggers import ClockCycles, RisingEdge
from cocotb.binary import BinaryValue
from cocotbext.axi.stream import define_stream

# Dataflow discriminant
OS = 0
WS = 1

# Propagate Discriminant
REG2 = 0
REG1 = 1


def unsigned_to_signed_8bit(value):
    return value - 256 if value >= 128 else value


def rounding_shift(value, shift):
    return round(float(value) / float(1 << shift))


def concatenate_data(data: list, width):
    data_concat = 0
    bitmask = (1 << width) - 1
    for i in range(len(data)):
        data_concat |= (int(data[i]) << (width * i)) & (bitmask << (width * i))
    return data_concat


def decopmose_data(data: BinaryValue, width):
    data_list = []
    data_len = len(data.binstr)
    num_data = data_len // width
    for i in reversed(range(num_data)):
        if "x" in data.binstr[i * width : (i + 1) * width]:
            data_list.append(None)
        else:
            data_list.append(
                BinaryValue(data.binstr[i * width : (i + 1) * width]).signed_integer
            )
    assert len(data_list) == num_data
    return data_list


# A/B/D input data stream
(
    InpDataBus,
    InpDataTransaction,
    InpDataSource,
    InpDataSink,
    InpDataMonitor,
) = define_stream(
    "InpData",
    signals=["payload_discriminant", "payload_Some_0", "resolver_ready"],
    valid_signal="payload_discriminant",
    ready_signal="resolver_ready",
)

# Input request control data stream
(
    InpCtrlBus,
    InpCtrlTransaction,
    InpCtrlSource,
    InpCtrlSink,
    InpCtrlMonitor,
) = define_stream(
    "InpCtrl",
    signals=[
        "payload_discriminant",
        "payload_Some_0_pe_control_dataflow_discriminant",
        "payload_Some_0_pe_control_propagate_discriminant",
        "payload_Some_0_pe_control_shift",
        "payload_Some_0_transpose_a",
        "payload_Some_0_transpose_bd",
        "payload_Some_0_total_rows",
        "payload_Some_0_tag_rob_id_discriminant",
        "payload_Some_0_tag_rob_id_Some_0",
        "payload_Some_0_tag_addr_is_acc_addr",
        "payload_Some_0_tag_addr_accumulate",
        "payload_Some_0_tag_addr_read_full_acc_row",
        "payload_Some_0_tag_addr_norm_cmd",
        "payload_Some_0_tag_addr_garbage",
        "payload_Some_0_tag_addr_is_garbage",
        "payload_Some_0_tag_addr_data",
        "payload_Some_0_tag_rows",
        "payload_Some_0_tag_cols",
        "payload_Some_0_flush",
        "resolver_ready",
    ],
    valid_signal="payload_discriminant",
    ready_signal="resolver_ready",
)


def os_flush_request(propagate, shift):
    return InpCtrlTransaction(
        payload_Some_0_transpose_a=False,
        payload_Some_0_transpose_bd=False,
        payload_Some_0_flush=1,
        payload_Some_0_pe_control_dataflow_discriminant=OS,
        payload_Some_0_pe_control_propagate_discriminant=propagate,
        payload_Some_0_pe_control_shift=shift,
        payload_Some_0_tag_addr_accumulate=False,
        payload_Some_0_tag_addr_data=0,
        payload_Some_0_tag_addr_garbage=False,
        payload_Some_0_tag_addr_is_garbage=False,
        payload_Some_0_tag_addr_is_acc_addr=False,
        payload_Some_0_tag_addr_read_full_acc_row=0,
        payload_Some_0_tag_cols=0,
        payload_Some_0_tag_rob_id_discriminant=False,
        payload_Some_0_tag_rob_id_Some_0=0,
        payload_Some_0_tag_rows=0,
        payload_Some_0_total_rows=16,
        payload_Some_0_tag_addr_norm_cmd=0,
    )


def req_with_none_rob_id(mode, transpose_a, transpose_bd, propagate):
    return InpCtrlTransaction(
        payload_Some_0_transpose_a=transpose_a,
        payload_Some_0_transpose_bd=transpose_bd,
        payload_Some_0_flush=0,
        payload_Some_0_pe_control_dataflow_discriminant=mode,
        payload_Some_0_pe_control_propagate_discriminant=propagate,
        payload_Some_0_pe_control_shift=0,
        payload_Some_0_total_rows=16,
        payload_Some_0_tag_rob_id_discriminant=False,
        payload_Some_0_tag_rob_id_Some_0=0,
        payload_Some_0_tag_addr_is_acc_addr=True,
        payload_Some_0_tag_addr_accumulate=True,
        payload_Some_0_tag_addr_read_full_acc_row=True,
        payload_Some_0_tag_addr_norm_cmd=0,
        payload_Some_0_tag_addr_garbage=True,
        payload_Some_0_tag_addr_is_garbage=True,
        payload_Some_0_tag_addr_data=0x3FFF,
        payload_Some_0_tag_rows=0,
        payload_Some_0_tag_cols=0,
    )


def req_with_rob_id(mode, transpose_a, transpose_bd, propagate):
    return InpCtrlTransaction(
        payload_Some_0_pe_control_dataflow_discriminant=mode,
        payload_Some_0_pe_control_propagate_discriminant=propagate,
        payload_Some_0_pe_control_shift=0,
        payload_Some_0_transpose_a=transpose_a,
        payload_Some_0_transpose_bd=transpose_bd,
        payload_Some_0_total_rows=16,
        payload_Some_0_tag_rob_id_discriminant=True,
        payload_Some_0_tag_rob_id_Some_0=16,
        payload_Some_0_tag_addr_is_acc_addr=1,
        payload_Some_0_tag_addr_accumulate=0,
        payload_Some_0_tag_addr_read_full_acc_row=0,
        payload_Some_0_tag_addr_norm_cmd=0,
        payload_Some_0_tag_addr_garbage=0,
        payload_Some_0_tag_addr_is_garbage=False,
        payload_Some_0_tag_addr_data=0,
        payload_Some_0_tag_rows=16,
        payload_Some_0_tag_cols=16,
        payload_Some_0_flush=0,
    )


def generate_ws_test_data(transpose_a, transpose_bd):
    activation = np.random.randint(-8, 8, (16, 16))
    weight = np.random.randint(-8, 8, (16, 16))
    bias = np.random.randint(-8, 8, (16, 16))

    if not transpose_a and not transpose_bd:
        expected_output = np.matmul(activation, weight) + bias
    elif transpose_a and not transpose_bd:
        expected_output = np.matmul(np.transpose(activation), weight) + bias
    elif not transpose_a and transpose_bd:
        expected_output = np.matmul(activation, np.transpose(weight)) + bias
    else:
        print("Invalid transpose mode")
        exit(1)

    return activation, weight, bias, expected_output


def generate_os_test_data(transpose_a, transpose_bd):
    activation = np.random.randint(-8, 8, (16, 16))
    weight = np.random.randint(-8, 8, (16, 16))
    bias = np.random.randint(-8, 8, (16, 16))
    rnd_shift = random.randint(1, 3)

    if not transpose_a and not transpose_bd:
        expected_output = np.matmul(activation, weight) + bias
    elif transpose_a and not transpose_bd:
        expected_output = np.matmul(np.transpose(activation), weight) + bias
    elif transpose_a and transpose_bd:
        expected_output = (
            np.matmul(np.transpose(activation), np.transpose(weight)) + bias
        )
    else:
        print("Invalid transpose mode")
        exit(1)

    expected_output = np.array(
        [[rounding_shift(value, rnd_shift) for value in row] for row in expected_output]
    )

    return activation, weight, bias, expected_output, rnd_shift


class TB(object):
    def __init__(self, dut):
        self.dut = dut

        self.log = logging.getLogger("cocotb.tb")
        self.log.setLevel(logging.DEBUG)

        cocotb.start_soon(Clock(dut.clk, 4, units="ns").start())

        self.in_a_data_req = InpDataSource(
            InpDataBus.from_prefix(dut, "in_input_0"), dut.clk, dut.rst
        )
        self.in_b_data_req = InpDataSource(
            InpDataBus.from_prefix(dut, "in_input_1"), dut.clk, dut.rst
        )
        self.in_d_data_req = InpDataSource(
            InpDataBus.from_prefix(dut, "in_input_2"), dut.clk, dut.rst
        )
        self.in_ctrl_req = InpCtrlSource(
            InpCtrlBus.from_prefix(dut, "in_input_3"), dut.clk, dut.rst
        )

        # TODO: Add resolver signals
        # output wire [6-1:0] in_input_3_resolver_inner_rob_id_discriminant,
        # output wire [36-1:0] in_input_3_resolver_inner_rob_id_Some_0,
        # output wire [6-1:0] in_input_3_resolver_inner_addr_is_acc_addr,
        # output wire [6-1:0] in_input_3_resolver_inner_addr_accumulate,
        # output wire [6-1:0] in_input_3_resolver_inner_addr_read_full_acc_row,
        # output wire [18-1:0] in_input_3_resolver_inner_addr_norm_cmd,
        # output wire [66-1:0] in_input_3_resolver_inner_addr_garbage,
        # output wire [6-1:0] in_input_3_resolver_inner_addr_is_garbage,
        # output wire [84-1:0] in_input_3_resolver_inner_addr_data,
        # output wire [30-1:0] in_input_3_resolver_inner_rows,
        # output wire [30-1:0] in_input_3_resolver_inner_cols,

        self.out_valid = self.dut.out_output_payload_discriminant
        self.out_total_rows = self.dut.out_output_payload_Some_0_total_rows
        self.out_tag_rob_id_discriminant = (
            self.dut.out_output_payload_Some_0_tag_rob_id_discriminant
        )
        self.out_tag_rob_id_Some_0 = (
            self.dut.out_output_payload_Some_0_tag_rob_id_Some_0
        )
        self.out_tag_addr_is_acc_addr = (
            self.dut.out_output_payload_Some_0_tag_addr_is_acc_addr
        )
        self.out_tag_addr_accumulate = (
            self.dut.out_output_payload_Some_0_tag_addr_accumulate
        )
        self.out_tag_addr_read_full_acc_row = (
            self.dut.out_output_payload_Some_0_tag_addr_read_full_acc_row
        )
        self.out_tag_addr_norm_cmd = (
            self.dut.out_output_payload_Some_0_tag_addr_norm_cmd
        )
        self.out_tag_addr_garbage = self.dut.out_output_payload_Some_0_tag_addr_garbage
        self.out_tag_addr_is_garbage = (
            self.dut.out_output_payload_Some_0_tag_addr_is_garbage
        )
        self.out_tag_addr_data = self.dut.out_output_payload_Some_0_tag_addr_data
        self.out_tag_rows = self.dut.out_output_payload_Some_0_tag_rows
        self.out_tag_cols = self.dut.out_output_payload_Some_0_tag_cols
        self.out_last = self.dut.out_output_payload_Some_0_last
        self.out_data = self.dut.out_output_payload_Some_0_data

    async def reset(self):
        self.dut.rst.setimmediatevalue(0)
        await ClockCycles(self.dut.clk, 5)
        self.dut.rst.setimmediatevalue(1)
        await ClockCycles(self.dut.clk, 5)
        self.dut.rst.setimmediatevalue(0)
        await ClockCycles(self.dut.clk, 5)


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def ws_no_transpose(dut):
    """
    WS Test without Transpose
    """
    # Start test
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Input data
    activation, weight, bias, expected_output = generate_ws_test_data(False, False)

    tb.log.info(f"[Mode] Weight-Stationary (No Transpose)")
    tb.log.info(f"Activation:\n{activation}")
    tb.log.info(f"weight:\n{weight}")
    tb.log.info(f"bias:\n{bias}")
    tb.log.info(f"Expected output: {expected_output}")

    # 1. Preload weight data
    await tb.in_ctrl_req.send(req_with_rob_id(WS, False, False, REG2))

    for i in range(16):
        await tb.in_a_data_req.send(
            InpDataTransaction(
                payload_Some_0=0,
            )
        )
        await tb.in_b_data_req.send(
            InpDataTransaction(
                payload_Some_0=0,
            )
        )
        # When running WS dataflow, weight should be sent in reverse order
        # due to the way the weight is preloaded in the PEs.
        await tb.in_d_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(weight[15 - i], 8),
            )
        )

    # 2. Send activation and bias data
    await tb.in_ctrl_req.send(req_with_none_rob_id(WS, False, False, REG1))

    for i in range(16):
        await tb.in_a_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(activation[i], 8),
            )
        )
        await tb.in_b_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(bias[i], 8),
            )
        )
        await tb.in_d_data_req.send(InpDataTransaction(payload_Some_0=0))

    output_data = []
    for _ in range(100):
        if tb.out_tag_cols.value.binstr == "10000":
            output_data.append(decopmose_data(tb.out_data.value, 20))
        await RisingEdge(dut.clk)

    tb.log.info(f"Output data: {output_data}")

    for i in range(16):
        for j in range(16):
            assert output_data[i][j] == expected_output[i][j]


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def ws_transpose_a(dut):
    """
    WS Test with A Transpose
    """
    # Start test
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Input data
    activation, weight, bias, expected_output = generate_ws_test_data(True, False)

    tb.log.info(f"[Mode] Weight-Stationary (Transpose A)")
    tb.log.info(f"Activation:\n{activation}")
    tb.log.info(f"weight:\n{weight}")
    tb.log.info(f"bias:\n{bias}")
    tb.log.info(f"Expected output: {expected_output}")

    # 1. Preload weight data
    await tb.in_ctrl_req.send(req_with_rob_id(WS, True, False, REG2))
    for i in range(16):
        await tb.in_a_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(activation[i], 8),
            )
        )
        await tb.in_b_data_req.send(InpDataTransaction(payload_Some_0=0))
        # When running WS dataflow, weight should be sent in reverse order
        # due to the way the weight is preloaded in the PEs.
        await tb.in_d_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(weight[15 - i], 8),
            )
        )

    # 2. Send activation and bias data
    await tb.in_ctrl_req.send(req_with_none_rob_id(WS, True, False, REG1))

    for i in range(16):
        await tb.in_a_data_req.send(InpDataTransaction(payload_Some_0=0))
        await tb.in_b_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(bias[i], 8),
            )
        )
        await tb.in_d_data_req.send(InpDataTransaction(payload_Some_0=0))

    output_data = []
    for _ in range(100):
        if (
            tb.out_tag_rob_id_discriminant.value.binstr == "1"
            and tb.out_tag_rob_id_Some_0.value.binstr == "010000"
        ):
            output_data.append(decopmose_data(tb.out_data.value, 20))
        await RisingEdge(dut.clk)

    tb.log.info(f"Output data: {output_data}")

    for i in range(16):
        for j in range(16):
            assert output_data[i][j] == expected_output[i][j]


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def ws_transpose_b(dut):
    """
    WS Test with B Transpose
    """
    # Start test
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Input data
    activation, weight, bias, expected_output = generate_ws_test_data(False, True)

    tb.log.info(f"[Mode] Weight-Stationary (Transpose B)")
    tb.log.info(f"Activation:\n{activation}")
    tb.log.info(f"weight:\n{weight}")
    tb.log.info(f"bias:\n{bias}")
    tb.log.info(f"Expected output: {expected_output}")

    # 1. Preload weight data
    await tb.in_ctrl_req.send(req_with_none_rob_id(WS, False, True, REG2))
    for i in range(16):
        await tb.in_a_data_req.send(InpDataTransaction(payload_Some_0=0))
        await tb.in_b_data_req.send(InpDataTransaction(payload_Some_0=0))
        # When running WS dataflow, weight should be sent in reverse order
        # due to the way the weight is preloaded in the PEs.
        await tb.in_d_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(weight[15 - i], 8),
            )
        )

    # 2. Wait until the data is loaded
    await tb.in_ctrl_req.send(req_with_rob_id(WS, False, True, REG2))
    for i in range(16):
        await tb.in_a_data_req.send(InpDataTransaction(payload_Some_0=0))
        await tb.in_b_data_req.send(InpDataTransaction(payload_Some_0=0))
        await tb.in_d_data_req.send(InpDataTransaction(payload_Some_0=0))

    # 3. Send activation and bias data
    await tb.in_ctrl_req.send(req_with_none_rob_id(WS, False, True, REG1))
    for i in range(16):
        await tb.in_a_data_req.send(
            InpDataTransaction(payload_Some_0=concatenate_data(activation[i], 8))
        )
        await tb.in_b_data_req.send(
            InpDataTransaction(payload_Some_0=concatenate_data(bias[i], 8))
        )
        await tb.in_d_data_req.send(InpDataTransaction(payload_Some_0=0))

    output_data = []
    for _ in range(100):
        if (
            tb.out_tag_rob_id_discriminant.value.binstr == "1"
            and tb.out_tag_rob_id_Some_0.value.binstr == "010000"
        ):
            output_data.append(decopmose_data(tb.out_data.value, 20))
        await RisingEdge(dut.clk)

    tb.log.info(f"Output data: {output_data}")

    for i in range(16):
        for j in range(16):
            assert output_data[i][j] == expected_output[i][j]


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def os_no_transpose(dut):
    """
    OS Test without Transpose
    """
    # Start test
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Input data
    activation = np.random.randint(-8, 8, (16, 16))
    weight = np.random.randint(-8, 8, (16, 16))
    bias = np.random.randint(-8, 8, (16, 16))

    rnd_shift = random.randint(1, 3)

    expected_output = np.matmul(activation, weight) + bias
    # Apply rounding shift to each element in the matrix
    expected_output = np.array(
        [[rounding_shift(value, rnd_shift) for value in row] for row in expected_output]
    )

    activation, weight, bias, expected_output, rnd_shift = generate_os_test_data(
        False, False
    )

    tb.log.info(f"[Ouptut-Stationary] No Transpose, Shift: {rnd_shift}")
    tb.log.info(f"Activation:\n{activation}")
    tb.log.info(f"weight:\n{weight}")
    tb.log.info(f"bias:\n{bias}")
    tb.log.info(f"Expected output: {expected_output}")

    tb.log.info(
        f"Test with activation: {activation}, weight: {weight}, bias: {bias}, shift: {rnd_shift}"
    )
    tb.log.info(f"Expected output: {expected_output}")

    # 1. Preload bias data
    await tb.in_ctrl_req.send(req_with_rob_id(OS, False, False, REG2))
    for i in range(16):
        await tb.in_a_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(activation[i], 8),
            )
        )
        await tb.in_b_data_req.send(
            InpDataTransaction(
                payload_Some_0=0,
            )
        )
        await tb.in_d_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(bias[15 - i], 8),
            )
        )

    await tb.in_ctrl_req.send(req_with_none_rob_id(OS, False, False, REG1))
    for i in range(16):
        await tb.in_a_data_req.send(InpDataTransaction(payload_Some_0=0))
        await tb.in_b_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(weight[i], 8),
            )
        )
        await tb.in_d_data_req.send(InpDataTransaction(payload_Some_0=0))

    await tb.in_ctrl_req.send(os_flush_request(REG1, rnd_shift))

    output_data = []
    for _ in range(100):
        if (
            tb.out_tag_rob_id_discriminant.value.binstr == "1"
            and tb.out_tag_rob_id_Some_0.value.binstr == "010000"
        ):
            output_data.append(decopmose_data(tb.out_data.value, 20))
        await RisingEdge(dut.clk)

    tb.log.info(f"Output data: {output_data}")

    for i in range(16):
        for j in range(16):
            # In the OS, the row index of output data is reversed
            assert output_data[15 - i][j] == expected_output[i][j]


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def os_transpose_a(dut):
    """
    OS Test with A Transpose
    """
    # Start test
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Input data
    activation, weight, bias, expected_output, rnd_shift = generate_os_test_data(
        True, False
    )

    tb.log.info(f"[Ouptut-Stationary] Transpose A, Shift: {rnd_shift}")
    tb.log.info(f"Activation:\n{activation}")
    tb.log.info(f"weight:\n{weight}")
    tb.log.info(f"bias:\n{bias}")
    tb.log.info(f"Expected output: {expected_output}")

    await tb.in_ctrl_req.send(req_with_rob_id(OS, True, False, REG2))
    for i in range(16):
        await tb.in_a_data_req.send(InpDataTransaction(payload_Some_0=0))
        await tb.in_b_data_req.send(InpDataTransaction(payload_Some_0=0))
        await tb.in_d_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(bias[15 - i], 8),
            )
        )

    await tb.in_ctrl_req.send(req_with_none_rob_id(OS, True, False, REG1))
    for i in range(16):
        await tb.in_a_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(activation[i], 8),
            )
        )
        await tb.in_b_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(weight[i], 8),
            )
        )
        await tb.in_d_data_req.send(InpDataTransaction(payload_Some_0=0))

    await tb.in_ctrl_req.send(os_flush_request(REG1, rnd_shift))

    output_data = []
    for _ in range(100):
        if (
            tb.out_tag_rob_id_discriminant.value.binstr == "1"
            and tb.out_tag_rob_id_Some_0.value.binstr == "010000"
        ):
            output_data.append(decopmose_data(tb.out_data.value, 20))
        await RisingEdge(dut.clk)

    tb.log.info(f"Output data: {output_data}")

    for i in range(16):
        for j in range(16):
            # In the OS, the row index of output data is reversed
            assert output_data[15 - i][j] == expected_output[i][j]


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def os_transpose_both(dut):
    """
    OS Test with Transpose both A and B
    """
    # Start test
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Input data
    activation, weight, bias, expected_output, rnd_shift = generate_os_test_data(
        True, True
    )

    tb.log.info(f"[Ouptut-Stationary] Transpose A and B, Shift: {rnd_shift}")
    tb.log.info(f"Activation:\n{activation}")
    tb.log.info(f"weight:\n{weight}")
    tb.log.info(f"bias:\n{bias}")
    tb.log.info(f"Expected output: {expected_output}")

    # 0. Preload the bias
    await tb.in_ctrl_req.send(req_with_none_rob_id(OS, True, True, REG2))
    for i in range(16):
        await tb.in_a_data_req.send(InpDataTransaction(payload_Some_0=0))
        await tb.in_b_data_req.send(InpDataTransaction(payload_Some_0=0))
        await tb.in_d_data_req.send(
            InpDataTransaction(payload_Some_0=concatenate_data(bias[15 - i], 8))
        )

    # 1. Send weight data
    await tb.in_ctrl_req.send(req_with_rob_id(OS, True, True, REG2))
    for i in range(16):
        await tb.in_a_data_req.send(InpDataTransaction(payload_Some_0=0))
        await tb.in_b_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(weight[i], 8),
            )
        )
        await tb.in_d_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(bias[15 - i], 8),
            )
        )

    # 2. Send activation data
    await tb.in_ctrl_req.send(req_with_none_rob_id(OS, True, True, REG1))
    for i in range(16):
        await tb.in_a_data_req.send(
            InpDataTransaction(
                payload_Some_0=concatenate_data(activation[i], 8),
            )
        )
        await tb.in_b_data_req.send(InpDataTransaction(payload_Some_0=0))
        await tb.in_d_data_req.send(InpDataTransaction(payload_Some_0=0))

    # 3. Flush the data
    await tb.in_ctrl_req.send(os_flush_request(REG1, rnd_shift))

    from collections import deque

    output_data = deque(maxlen=16)
    for _ in range(200):
        if (
            tb.out_tag_rob_id_discriminant.value.binstr == "1"
            and tb.out_tag_rob_id_Some_0.value.binstr == "010000"
        ):
            output_data.append(decopmose_data(tb.out_data.value, 20))
        await RisingEdge(dut.clk)

    tb.log.info(f"Output data: {np.array(output_data)}")

    for i in range(16):
        for j in range(16):
            # In the OS, the row index of output data is reversed
            assert output_data[15 - i][j] == expected_output[i][j]
