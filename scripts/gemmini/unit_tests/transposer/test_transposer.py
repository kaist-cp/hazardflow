import logging
import random

import numpy as np
from itertools import product

import cocotb
from cocotb.clock import Clock
from cocotb.triggers import ClockCycles, RisingEdge, FallingEdge
from cocotb.regression import TestFactory
from cocotb.binary import BinaryValue


DIM = 16


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


def decompose_data(data: BinaryValue, width):
    data_list = []
    data_len = len(data.binstr)
    num_data = data_len // width

    for i in reversed(range(num_data)):
        data_list.append(BinaryValue(data.binstr[i * width : (i + 1) * width]).signed_integer)

    return data_list


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def transpose_test(dut):
    """
    Testcase.
    """
    random.seed(0)
    np.set_printoptions(linewidth=200)
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Generate random matrices
    A = np.array([[random.randint(-128, 127) for _ in range(DIM)] for _ in range(DIM)])
    B = np.array([[random.randint(-128, 127) for _ in range(DIM)] for _ in range(DIM)])

    A_t = np.transpose(A)
    B_t = np.transpose(B)

    cocotb.log.info("== Transposer testcase ==")

    cocotb.log.info(f"A (input 1):\n{A}")
    cocotb.log.info(f"A transposed (expected output 1):\n{A_t}")

    cocotb.log.info(f"B (input 2):\n{B}")
    cocotb.log.info(f"B transposed (expected output 2):\n{B_t}")

    for (i, (delay1, delay2, delay3)) in enumerate(product([False, True], repeat=3)):
        cocotb.log.info(f"Config {i}:")
        if i == 0:
            cocotb.log.info("No delay")
        else:
            if delay1:
                cocotb.log.info("Add a delay for inputting A")
            if delay2:
                cocotb.log.info("Add a delay for outputting A and inputting B")
            if delay3:
                cocotb.log.info("Add a delay for outputting B")

        A_o, B_o = [], []

        for i in range(DIM):
            # Add a delay for inputting A
            if delay1:
                tb.in_row_valid.value = 0
                await RisingEdge(dut.clk)

            tb.in_row_valid.value = 1
            tb.in_row_data.value = concatenate_data(A[i], 8)
            await RisingEdge(dut.clk)

        for i in range(DIM):
            # Add a delay for outputting A and inputting B
            if delay2:
                tb.in_row_valid.value = 0
                await RisingEdge(dut.clk)

            tb.in_row_valid.value = 1
            tb.in_row_data.value = concatenate_data(B[i], 8)
            await FallingEdge(dut.clk)

            A_o.append(decompose_data(tb.out_col_data.value, 8))
            await RisingEdge(dut.clk)

        A_o = np.array(A_o)
        cocotb.log.info(f"A transposed (output):\n{A_o}")
        assert np.array_equal(A_o, A_t)

        for i in range(DIM):
            # Add a delay for outputting B
            if delay3:
                tb.in_row_valid.value = 0
                await RisingEdge(dut.clk)

            tb.in_row_valid.value = 1
            tb.in_row_data.value = 0
            await FallingEdge(dut.clk)

            B_o.append(decompose_data(tb.out_col_data.value, 8))
            await RisingEdge(dut.clk)

        B_o = np.array(B_o)
        cocotb.log.info(f"B transposed (output):\n{B_o}")
        assert np.array_equal(B_o, B_t)
