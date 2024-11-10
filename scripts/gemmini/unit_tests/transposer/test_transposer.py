import logging
import random

import numpy as np

import cocotb
from cocotb.clock import Clock
from cocotb.triggers import ClockCycles, RisingEdge, FallingEdge
from cocotb.regression import TestFactory
from cocotb.binary import BinaryValue


random.seed(0)


class TB(object):
    def __init__(self, dut):
        self.dut = dut

        self.log = logging.getLogger("cocotb.tb")
        self.log.setLevel(logging.DEBUG)

        cocotb.start_soon(Clock(dut.clk, 4, units="ns").start())

        self.in_row_valid = self.dut.in_input_0_payload_discriminant
        self.in_row_data = self.dut.in_input_0_payload_Some_0_0

        self.out_col_valid = self.dut.out_output_payload_discriminant
        self.out_col_data = self.dut.out_output_payload_Some_0_0

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
        data_list.append(
            BinaryValue(data.binstr[i * width : (i + 1) * width]).signed_integer
        )
    return data_list


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def transpose_test1(dut):
    """
    Weight Stationary testcase1
    """
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Generate inputs randomly
    input_data = [[random.randint(-128, 127) for _ in range(16)] for _ in range(16)]

    cocotb.log.info("== Tranaposer testcase1 ==")
    cocotb.log.info(f"Input data: {input_data}")

    output_data = np.transpose(input_data)
    cocotb.log.info(f"Expected Output data: {output_data}")

    for i in range(16):
        in_row_data = concatenate_data(input_data[i], 8)
        tb.in_row_data.value = in_row_data
        tb.in_row_valid.value = 1
        await RisingEdge(dut.clk)
    for i in range(16):
        tb.in_row_valid.value = 0
        tb.in_row_data.value = 0
        await RisingEdge(dut.clk)

        tb.in_row_valid.value = 1
        tb.in_row_data.value = 0

        await FallingEdge(dut.clk)

        out_col_data = np.array(decopmose_data(tb.out_col_data.value, 8))
        assert np.array_equal(out_col_data, output_data[i])
        cocotb.log.info(f"Output data: {out_col_data}")

        await RisingEdge(dut.clk)
