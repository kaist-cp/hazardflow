import logging
import random
import os, sys

import numpy as np

import cocotb
from cocotb.clock import Clock
from cocotb.triggers import ClockCycles, RisingEdge
from cocotb.binary import BinaryValue


gemmini_unit_tb_dir = os.path.dirname(os.path.dirname(os.path.realpath(__file__)))
sys.path.append(gemmini_unit_tb_dir)

random.seed(0)

# Dataflow discriminant
OS = 0
WS = 1

# Propagate Discriminant
REG2 = 0
REG1 = 1


class TB(object):
    def __init__(self, dut):
        self.dut = dut

        self.log = logging.getLogger("cocotb.tb")
        self.log.setLevel(logging.DEBUG)

        cocotb.start_soon(Clock(dut.clk, 4, units="ns").start())

        self.in_row_data_valids = self.dut.in_input_0_payload_discriminant
        self.in_row_data_a = self.dut.in_input_0_payload_Some_0_a_0

        self.in_col_data_valids = self.dut.in_input_1_0_payload_discriminant
        self.in_col_data_b = self.dut.in_input_1_0_payload_Some_0_b_0
        self.in_col_data_d = self.dut.in_input_1_0_payload_Some_0_d_0

        self.in_col_ctrl_valids = self.dut.in_input_1_1_payload_discriminant
        self.in_col_ctrl_id = self.dut.in_input_1_1_payload_Some_0_id
        self.in_col_ctrl_last = self.dut.in_input_1_1_payload_Some_0_last
        self.in_col_ctrl_dataflow = (
            self.dut.in_input_1_1_payload_Some_0_control_dataflow_discriminant
        )
        self.in_col_ctrl_propagate = (
            self.dut.in_input_1_1_payload_Some_0_control_propagate_discriminant
        )
        self.in_col_ctrl_shift = self.dut.in_input_1_1_payload_Some_0_control_shift

        self.out_row_data_valids = self.dut.out_output_0_payload_discriminant
        self.out_row_data_a = self.dut.out_output_0_payload_Some_0_a_0

        self.out_col_data_valids = self.dut.out_output_1_0_payload_discriminant
        self.out_col_data_b = self.dut.out_output_1_0_payload_Some_0_b_0
        self.out_col_data_d = self.dut.out_output_1_0_payload_Some_0_d_0

        self.out_col_ctrl_valids = self.dut.out_output_1_1_payload_discriminant
        self.out_col_ctrl_id = self.dut.out_output_1_1_payload_Some_0_id
        self.out_col_ctrl_last = self.dut.out_output_1_1_payload_Some_0_last
        self.out_col_ctrl_dataflow = (
            self.dut.out_output_1_1_payload_Some_0_control_dataflow_discriminant
        )
        self.out_col_ctrl_propagate = (
            self.dut.out_output_1_1_payload_Some_0_control_propagate_discriminant
        )
        self.out_col_ctrl_shift = self.dut.out_output_1_1_payload_Some_0_control_shift

    async def reset(self):
        self.dut.rst.setimmediatevalue(0)
        await ClockCycles(self.dut.clk, 5)
        self.dut.rst.setimmediatevalue(1)
        await ClockCycles(self.dut.clk, 5)
        self.dut.rst.setimmediatevalue(0)
        await ClockCycles(self.dut.clk, 5)


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


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def ws_simple(dut):
    """
    Test Weight Stationary with ones
    """
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Generate inputs randomly
    activation = np.ones((16, 16))
    weight = np.ones((16, 16))
    bias = np.ones((16, 16))

    golden_output_data = np.matmul(activation, weight) + bias

    cocotb.log.info("== Weight Stationary testcase1 ==")
    cocotb.log.info(f"Input data: {activation}")
    cocotb.log.info(f"Weight data: {weight}")
    cocotb.log.info(f"Bias data: {bias}")
    cocotb.log.info(f"Expected Output data: {golden_output_data}")

    output_data = []

    # Below control signals don't affect the computation in WS
    col_ctrl_valids = [True] * 16
    col_ctrl_id = [random.randint(0, 7) for _ in range(16)]
    col_ctrl_last = [False] * 16
    col_ctrl_dataflow = [WS] * 16
    col_ctrl_shift = [0] * 16

    tb.in_col_ctrl_valids.value = concatenate_data(col_ctrl_valids, 1)
    tb.in_col_ctrl_dataflow.value = concatenate_data(col_ctrl_dataflow, 1)
    tb.in_col_ctrl_id.value = concatenate_data(col_ctrl_id, 3)
    tb.in_col_ctrl_last.value = concatenate_data(col_ctrl_last, 1)
    tb.in_col_ctrl_shift.value = concatenate_data(col_ctrl_shift, 4)

    # Preload weight
    for i in range(16):
        data_valids = [False] * 16
        col_data = [0] * 16

        for j in range(i + 1):
            data_valids[j] = True
            col_data[j] = weight[15 - i + j][j]

        tb.in_row_data_valids.value = 0
        tb.in_row_data_a.value = 0

        tb.in_col_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_col_data_b.value = 0
        tb.in_col_data_d.value = concatenate_data(col_data, 20)

        tb.in_col_ctrl_propagate.value = concatenate_data([REG2] * 16, 1)

        await RisingEdge(dut.clk)

    for i in range(15):
        data_valids = [False] * 16
        col_data = [0] * 16

        for j in range(16):
            data_valids[j] = j > i
            col_data[j] = weight[j - i - 1][j] if j > i else 0

        tb.in_row_data_valids.value = 0
        tb.in_row_data_a.value = 0

        tb.in_col_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_col_data_b.value = 0
        tb.in_col_data_d.value = concatenate_data(col_data, 20)

        tb.in_col_ctrl_propagate.value = concatenate_data([REG2] * 16, 1)

        await RisingEdge(dut.clk)

    # Compute WS
    for i in range(16):
        data_valids = [False] * 16
        row_data = [0] * 16
        col_data = [0] * 16

        for j in range(16):
            data_valids[j] = j <= i
            row_data[j] = activation[i - j][j] if j <= i else 0
            col_data[j] = bias[i-j][j] if j <= i else 0

        tb.in_row_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_row_data_a.value = concatenate_data(row_data, 8)

        tb.in_col_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_col_data_b.value = concatenate_data(col_data, 20)
        tb.in_col_data_d.value = 0

        tb.in_col_ctrl_propagate.value = concatenate_data([REG1] * 16, 1)

        await RisingEdge(dut.clk)

    for i in range(15):
        data_valids = [False] * 16
        row_data = [0] * 16
        col_data = [0] * 16

        for j in range(16):
            data_valids[j] = i < j
            row_data[j] = activation[16 + i - j][j] if i < j else 0
            col_data[j] = bias[16 + i - j][j] if i < j else 0

        tb.in_row_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_row_data_a.value = concatenate_data(row_data, 8)

        tb.in_col_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_col_data_b.value = concatenate_data(col_data, 20)
        tb.in_col_data_d.value = 0

        tb.in_col_ctrl_propagate.value = concatenate_data([REG1] * 16, 1)

        if tb.out_col_ctrl_propagate.value.binstr == "1111111111111111":
            data_decomposed = decopmose_data(tb.out_col_data_b.value, 20)
            output_data.append(data_decomposed)
        await RisingEdge(dut.clk)

    tb.in_row_data_valids.value = concatenate_data([False] * 16, 1)
    tb.in_col_data_valids.value = concatenate_data([False] * 16, 1)

    for i in range(24):
        if tb.out_col_ctrl_propagate.value.binstr == "1111111111111111":
            data_decomposed = decopmose_data(tb.out_col_data_b.value, 20)
            output_data.append(data_decomposed)
        await RisingEdge(dut.clk)

    tb.log.info(f"output_data: {output_data}")

    # Check output
    for i in range(16):
        for j in range(16):
            assert output_data[i + j][j] == golden_output_data[i][j]


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def ws_random(dut):
    """
    Test Weight Stationary with random inputs
    """
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Generate inputs randomly
    activation = np.random.randint(-8, 8, (16, 16))
    weight = np.random.randint(-8, 8, (16, 16))
    bias = np.random.randint(-8, 8, (16, 16))

    golden_output_data = np.matmul(activation, weight) + bias

    cocotb.log.info("== Weight Stationary testcase1 ==")
    cocotb.log.info(f"Input data: {activation}")
    cocotb.log.info(f"Weight data: {weight}")
    cocotb.log.info(f"Bias data: {bias}")
    cocotb.log.info(f"Expected Output data: {golden_output_data}")

    output_data = []

    # Below control signals don't affect the computation in WS
    col_ctrl_valids = [True] * 16
    col_ctrl_id = [random.randint(0, 7) for _ in range(16)]
    col_ctrl_last = [False] * 16
    col_ctrl_dataflow = [WS] * 16
    col_ctrl_shift = [0] * 16

    tb.in_col_ctrl_valids.value = concatenate_data(col_ctrl_valids, 1)
    tb.in_col_ctrl_dataflow.value = concatenate_data(col_ctrl_dataflow, 1)
    tb.in_col_ctrl_id.value = concatenate_data(col_ctrl_id, 3)
    tb.in_col_ctrl_last.value = concatenate_data(col_ctrl_last, 1)
    tb.in_col_ctrl_shift.value = concatenate_data(col_ctrl_shift, 4)

    # Preload weight
    for i in range(16):
        data_valids = [False] * 16
        col_data = [0] * 16

        for j in range(i + 1):
            data_valids[j] = True
            col_data[j] = weight[15 - i + j][j]

        tb.in_row_data_valids.value = 0
        tb.in_row_data_a.value = 0

        tb.in_col_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_col_data_b.value = 0
        tb.in_col_data_d.value = concatenate_data(col_data, 20)

        tb.in_col_ctrl_propagate.value = concatenate_data([REG2] * 16, 1)

        await RisingEdge(dut.clk)

    for i in range(15):
        data_valids = [False] * 16
        col_data = [0] * 16

        for j in range(16):
            data_valids[j] = j > i
            col_data[j] = weight[j - i - 1][j] if j > i else 0

        tb.in_row_data_valids.value = 0
        tb.in_row_data_a.value = 0

        tb.in_col_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_col_data_b.value = 0
        tb.in_col_data_d.value = concatenate_data(col_data, 20)

        tb.in_col_ctrl_propagate.value = concatenate_data([REG2] * 16, 1)

        await RisingEdge(dut.clk)

    # Compute WS
    for i in range(16):
        data_valids = [False] * 16
        row_data = [0] * 16
        col_data = [0] * 16

        for j in range(16):
            data_valids[j] = j <= i
            row_data[j] = activation[i - j][j] if j <= i else 0
            col_data[j] = bias[i-j][j] if j <= i else 0

        tb.in_row_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_row_data_a.value = concatenate_data(row_data, 8)

        tb.in_col_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_col_data_b.value = concatenate_data(col_data, 20)
        tb.in_col_data_d.value = 0

        tb.in_col_ctrl_propagate.value = concatenate_data([REG1] * 16, 1)

        await RisingEdge(dut.clk)

    for i in range(15):
        data_valids = [False] * 16
        row_data = [0] * 16
        col_data = [0] * 16

        for j in range(16):
            data_valids[j] = i < j
            row_data[j] = activation[16 + i - j][j] if i < j else 0
            col_data[j] = bias[16 + i - j][j] if i < j else 0

        tb.in_row_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_row_data_a.value = concatenate_data(row_data, 8)

        tb.in_col_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_col_data_b.value = concatenate_data(col_data, 20)
        tb.in_col_data_d.value = 0

        tb.in_col_ctrl_propagate.value = concatenate_data([REG1] * 16, 1)

        if tb.out_col_ctrl_propagate.value.binstr == "1111111111111111":
            data_decomposed = decopmose_data(tb.out_col_data_b.value, 20)
            output_data.append(data_decomposed)
        await RisingEdge(dut.clk)

    tb.in_row_data_valids.value = concatenate_data([False] * 16, 1)
    tb.in_col_data_valids.value = concatenate_data([False] * 16, 1)

    for i in range(24):
        if tb.out_col_ctrl_propagate.value.binstr == "1111111111111111":
            data_decomposed = decopmose_data(tb.out_col_data_b.value, 20)
            output_data.append(data_decomposed)
        await RisingEdge(dut.clk)

    tb.log.info(f"output_data: {output_data}")

    # Check output
    for i in range(16):
        for j in range(16):
            assert output_data[i + j][j] == golden_output_data[i][j]


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def os_simple(dut):
    """
    Test Output Stationary with ones
    """
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Generate inputs randomly
    activation = np.ones((16, 16))
    weight = np.ones((16, 16))
    bias = np.ones((16, 16))
    golden_output_data = np.matmul(activation, weight)

    cocotb.log.info("== Output Stationary testcase1 ==")

    cocotb.log.info(f"Input data: {activation}")
    cocotb.log.info(f"Weight data: {weight}")
    cocotb.log.info(f"Expected Output data: {golden_output_data}")

    output_data = []

    # Below control signals don't affect the computation in OS
    col_ctrl_valids = [True] * 16
    col_ctrl_id = [random.randint(0, 7) for _ in range(16)]
    col_ctrl_last = [False] * 16
    col_ctrl_dataflow = [OS] * 16
    col_ctrl_shift = [0] * 16

    tb.in_col_ctrl_valids.value = concatenate_data(col_ctrl_valids, 1)
    tb.in_col_ctrl_dataflow.value = concatenate_data(col_ctrl_dataflow, 1)
    tb.in_col_ctrl_id.value = concatenate_data(col_ctrl_id, 3)
    tb.in_col_ctrl_last.value = concatenate_data(col_ctrl_last, 1)
    tb.in_col_ctrl_shift.value = concatenate_data(col_ctrl_shift, 4)

    # Preload bias to c2 (Preload doesn't need to be done in diamond shape)
    for i in range(16):
        tb.in_col_data_valids.value = concatenate_data([True] * 16, 1)
        tb.in_col_data_d.value = concatenate_data(bias[i], 20)
        tb.in_col_ctrl_propagate.value = concatenate_data([REG1] * 16, 1)
        await RisingEdge(dut.clk)
    for i in range(16):
        tb.in_col_data_valids.value = concatenate_data([True] * 16, 1)
        tb.in_col_data_d.value = 0
        tb.in_col_ctrl_propagate.value = concatenate_data([REG2] * 16, 1)
        await RisingEdge(dut.clk)

    # Compute OS
    for i in range(16):
        data_valids = [False] * 16
        row_data = [0] * 16
        col_data = [0] * 16
        propagate = [REG2] * 16

        for j in range(i + 1):
            data_valids[j] = True
            row_data[j] = activation[j][i - j]
            col_data[j] = weight[i - j][j]
            propagate[j] = REG1 if j <= i else REG2

        # tb.log.info(f"i: {i}, row_data: {row_data}, col_data: {col_data}, propagete: {propagate}")

        tb.in_row_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_col_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_row_data_a.value = concatenate_data(row_data, 8)
        tb.in_col_data_b.value = concatenate_data(col_data, 20)
        tb.in_col_data_d.value = 0
        tb.in_col_ctrl_propagate.value = concatenate_data(propagate, 1)

        await RisingEdge(dut.clk)

    for i in range(15):
        data_valids = [True] * 16
        row_data = [0] * 16
        col_data = [0] * 16
        propagate = [REG1] * 16

        for j in range(16):
            row_data[j] = activation[j][16 - (j - i)] if j > i else 0
            col_data[j] = weight[16 - (j - i)][j] if j > i else 0
            propagate[j] = REG1 if j > i else REG2

        # tb.log.info(f"i: {i}, row_data: {row_data}, col_data: {col_data}, propagete: {propagate}")

        tb.in_row_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_col_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_row_data_a.value = concatenate_data(row_data, 8)
        tb.in_col_data_b.value = concatenate_data(col_data, 20)
        tb.in_col_data_d.value = 0
        tb.in_col_ctrl_propagate.value = concatenate_data(propagate, 1)

        await RisingEdge(dut.clk)

    tb.in_row_data_valids.value = concatenate_data([False] * 16, 1)
    tb.in_col_data_valids.value = concatenate_data([False] * 16, 1)
    tb.in_col_ctrl_propagate.value = concatenate_data([REG2] * 16, 1)

    for i in range(64):
        output_data.append(decopmose_data(tb.out_col_data_d.value, 20))
        await RisingEdge(dut.clk)
    output_data = output_data[2:]
    tb.log.info(f"output_data: {output_data}")

    for i in range(16):
        for j in range(16):
            assert golden_output_data[i][j] == output_data[15 - i + j][j]

@cocotb.test(timeout_time=10, timeout_unit="ms")
async def os_random(dut):
    """
    Test Output Stationary with random numbers
    """
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Generate inputs randomly
    activation = np.random.randint(-8, 8, (16, 16))
    weight = np.random.randint(-8, 8, (16, 16))
    bias = np.random.randint(-8, 8, (16, 16))
    golden_output_data = np.matmul(activation, weight)

    cocotb.log.info("== Output Stationary testcase1 ==")

    cocotb.log.info(f"Input data: {activation}")
    cocotb.log.info(f"Weight data: {weight}")
    cocotb.log.info(f"Expected Output data: {golden_output_data}")

    output_data = []

    # Below control signals don't affect the computation in OS
    col_ctrl_valids = [True] * 16
    col_ctrl_id = [random.randint(0, 7) for _ in range(16)]
    col_ctrl_last = [False] * 16
    col_ctrl_dataflow = [OS] * 16
    col_ctrl_shift = [0] * 16

    tb.in_col_ctrl_valids.value = concatenate_data(col_ctrl_valids, 1)
    tb.in_col_ctrl_dataflow.value = concatenate_data(col_ctrl_dataflow, 1)
    tb.in_col_ctrl_id.value = concatenate_data(col_ctrl_id, 3)
    tb.in_col_ctrl_last.value = concatenate_data(col_ctrl_last, 1)
    tb.in_col_ctrl_shift.value = concatenate_data(col_ctrl_shift, 4)

    # Preload bias to c2 (Preload doesn't need to be done in diamond shape)
    for i in range(16):
        tb.in_col_data_valids.value = concatenate_data([True] * 16, 1)
        tb.in_col_data_d.value = concatenate_data(bias[i], 20)
        tb.in_col_ctrl_propagate.value = concatenate_data([REG1] * 16, 1)
        await RisingEdge(dut.clk)
    for i in range(16):
        tb.in_col_data_valids.value = concatenate_data([True] * 16, 1)
        tb.in_col_data_d.value = 0
        tb.in_col_ctrl_propagate.value = concatenate_data([REG2] * 16, 1)
        await RisingEdge(dut.clk)

    # Compute OS
    for i in range(16):
        data_valids = [False] * 16
        row_data = [0] * 16
        col_data = [0] * 16
        propagate = [REG2] * 16

        for j in range(i + 1):
            data_valids[j] = True
            row_data[j] = activation[j][i - j]
            col_data[j] = weight[i - j][j]
            propagate[j] = REG1 if j <= i else REG2

        # tb.log.info(f"i: {i}, row_data: {row_data}, col_data: {col_data}, propagete: {propagate}")

        tb.in_row_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_col_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_row_data_a.value = concatenate_data(row_data, 8)
        tb.in_col_data_b.value = concatenate_data(col_data, 20)
        tb.in_col_data_d.value = 0
        tb.in_col_ctrl_propagate.value = concatenate_data(propagate, 1)

        await RisingEdge(dut.clk)

    for i in range(15):
        data_valids = [True] * 16
        row_data = [0] * 16
        col_data = [0] * 16
        propagate = [REG1] * 16

        for j in range(16):
            row_data[j] = activation[j][16 - (j - i)] if j > i else 0
            col_data[j] = weight[16 - (j - i)][j] if j > i else 0
            propagate[j] = REG1 if j > i else REG2

        # tb.log.info(f"i: {i}, row_data: {row_data}, col_data: {col_data}, propagete: {propagate}")

        tb.in_row_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_col_data_valids.value = concatenate_data(data_valids, 1)
        tb.in_row_data_a.value = concatenate_data(row_data, 8)
        tb.in_col_data_b.value = concatenate_data(col_data, 20)
        tb.in_col_data_d.value = 0
        tb.in_col_ctrl_propagate.value = concatenate_data(propagate, 1)

        await RisingEdge(dut.clk)

    tb.in_row_data_valids.value = concatenate_data([False] * 16, 1)
    tb.in_col_data_valids.value = concatenate_data([False] * 16, 1)
    tb.in_col_ctrl_propagate.value = concatenate_data([REG2] * 16, 1)

    for i in range(64):
        output_data.append(decopmose_data(tb.out_col_data_d.value, 20))
        await RisingEdge(dut.clk)
    output_data = output_data[2:]
    tb.log.info(f"output_data: {output_data}")

    for i in range(16):
        for j in range(16):
            assert golden_output_data[i][j] == output_data[15 - i + j][j]
