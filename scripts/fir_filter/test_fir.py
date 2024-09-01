import random
import cocotb
from cocotb.clock import Clock
from cocotb.triggers import RisingEdge


def get_golden_signal(input_signal, coeffs):
    num_inputs, num_coeffs = len(input_signal), len(coeffs)
    golden_signal = [0 for _ in range(num_inputs)]

    for i in range(num_inputs):
        for j in range(num_coeffs):
            if i >= j:
                golden_signal[i] += input_signal[i-j] * coeffs[j]

    return golden_signal


@cocotb.test()
async def test_fir(dut):
    num_inputs = 10
    random.seed(2024)
    input_signal = [random.randrange(0, 256) for _ in range(num_inputs)]
    output_signal = [0 for _ in range(num_inputs)]
    golden_signal = get_golden_signal(input_signal, [4, 2, 3])

    # start simulator clock
    cocotb.start_soon(Clock(dut.clk, 4, units="ns").start())

    # Reset DUT
    await RisingEdge(dut.clk)
    dut.rst.value = 1
    await RisingEdge(dut.clk)
    dut.rst.value = 0

    # run through each clock
    for samp in range(num_inputs):
        # feed a new input in
        dut.data_in.value = input_signal[samp]

        await RisingEdge(dut.clk)

        # get the output at rising edge
        dut_data_out = dut.data_out.value

        output_signal[samp] = int(dut_data_out)

    assert output_signal == golden_signal, "\ninput:  %s\noutput: %s\ngolden: %s" % (input_signal, output_signal, golden_signal)
