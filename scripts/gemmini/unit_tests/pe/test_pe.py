import logging
import random

import cocotb
from cocotb.clock import Clock
from cocotb.triggers import ClockCycles
from cocotb.regression import TestFactory
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


# PE Data Row stream
(
    PeDataRowBus,
    PeDataRowTransaction,
    PeDataRowSource,
    PeDataRowSink,
    PeDataRowMonitor,
) = define_stream(
    "PeDataRow",
    signals=["payload_discriminant", "payload_Some_0_a"],
    valid_signal="payload_discriminant",
)

# PE Data Column Stream
(
    PeDataColBus,
    PeDataColTransaction,
    PeDataColSource,
    PeDataColSink,
    PeDataColMonitor,
) = define_stream(
    "PeDataCol",
    signals=["payload_discriminant", "payload_Some_0_b", "payload_Some_0_d"],
    valid_signal="payload_discriminant",
)

# PE Control Column Stream
(
    PeControlColBus,
    PeControlColTransaction,
    PeControlColSource,
    PeControlColSink,
    PeControlColMonitor,
) = define_stream(
    "PeControlCol",
    signals=[
        "payload_discriminant",
        "payload_Some_0_id",
        "payload_Some_0_last",
        "payload_Some_0_control_dataflow_discriminant",
        "payload_Some_0_control_propagate_discriminant",
        "payload_Some_0_control_shift",
        "payload_Some_0_bad_dataflow",
    ],
    valid_signal="payload_discriminant",
)


class PE:
    def __init__(self):
        self.c1 = 0
        self.c2 = 0

    def preload_weight(self, data, tgt_reg):
        if tgt_reg == REG1:
            self.c1 = data
        else:  # tgt_reg == REG2
            self.c2 = data

    def reset(self):
        self.c1 = 0
        self.c2 = 0

    def compute_os(self, data_a, data_b, data_d, propagate, shift):
        if propagate == REG1:
            mac_result = data_a * unsigned_to_signed_8bit(data_b & 0xFF) + self.c2
            self.c1 = data_d
            self.c2 = mac_result
            return rounding_shift(mac_result, shift)
        elif propagate == REG2:
            mac_result = data_a * unsigned_to_signed_8bit(data_b & 0xFF) + self.c1
            self.c1 = mac_result
            self.c2 = data_d
            return rounding_shift(mac_result, shift)

    def compute_ws(self, data_a, data_b, data_d, propagate):
        if propagate == REG1:
            mac_result = data_a * unsigned_to_signed_8bit(self.c2 & 0xFF) + data_b
            self.c1 = data_d
            return mac_result
        elif propagate == REG2:
            mac_result = data_a * unsigned_to_signed_8bit(self.c1 & 0xFF) + data_b
            self.c2 = data_b
            return mac_result


class TB(object):
    def __init__(self, dut):
        self.dut = dut

        self.log = logging.getLogger("cocotb.tb")
        self.log.setLevel(logging.DEBUG)

        cocotb.start_soon(Clock(dut.clk, 4, units="ns").start())

        self.pe_row_data_req = PeDataRowSource(
            PeDataRowBus.from_prefix(dut, "in_input_0"), dut.clk, dut.rst
        )
        self.pe_col_data_req = PeDataColSource(
            PeDataColBus.from_prefix(dut, "in_input_1_0"), dut.clk, dut.rst
        )
        self.pe_col_ctrl_req = PeControlColSource(
            PeControlColBus.from_prefix(dut, "in_input_1_1"), dut.clk, dut.rst
        )

        self.pe_row_data_resp = PeDataRowSink(
            PeDataRowBus.from_prefix(dut, "out_output_0"), dut.clk, dut.rst
        )
        self.pe_col_data_resp = PeDataColSink(
            PeDataColBus.from_prefix(dut, "out_output_1_0"), dut.clk, dut.rst
        )
        self.pe_col_ctrl_resp = PeControlColSink(
            PeControlColBus.from_prefix(dut, "out_output_1_1"), dut.clk, dut.rst
        )

    async def reset(self):
        self.dut.rst.setimmediatevalue(0)
        await ClockCycles(self.dut.clk, 5)
        self.dut.rst.setimmediatevalue(1)
        await ClockCycles(self.dut.clk, 5)
        self.dut.rst.setimmediatevalue(0)
        await ClockCycles(self.dut.clk, 5)


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def ws_simple(dut):
    """
    Simplest Weight Stationary testcase
    """
    # These value don't affect the output. Just forward the input to output as written in the document.
    rnd_last_idx = random.randint(0, 15)
    rnd_id = random.randint(0, 7)
    rnd_shift = random.randint(0, 10)

    # Generate inputs for test
    activation = [1] * 16
    weight = 1

    # Compute golden output
    golden_pe = PE()
    golden_pe.preload_weight(weight, REG2)
    output_data = []
    for i in range(16):
        output_data.append(golden_pe.compute_ws(activation[i], 0, 0, REG1))

    cocotb.log.info(f"Last index: {rnd_last_idx}")
    cocotb.log.info(f"Random ID: {rnd_id}")
    cocotb.log.info(f"Random Shift: {rnd_shift}")

    cocotb.log.info(f"Input data: {activation}")
    cocotb.log.info(f"Weight data: {weight}")
    cocotb.log.info(f"Expected Output data: {output_data}")

    # Start test
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Preload weight data
    req = PeControlColTransaction(
        payload_Some_0_id=rnd_id,
        payload_Some_0_last=random.randint(0, 1) > 0.5,
        payload_Some_0_control_dataflow_discriminant=WS,
        payload_Some_0_control_propagate_discriminant=REG2,
        payload_Some_0_control_shift=rnd_shift,
        payload_Some_0_bad_dataflow=False,
    )
    await tb.pe_row_data_req.send(PeDataRowTransaction(payload_Some_0_a=0))
    await tb.pe_col_data_req.send(
        PeDataColTransaction(
            payload_Some_0_b=0,
            payload_Some_0_d=weight,
        )
    )
    await tb.pe_col_ctrl_req.send(req)

    row_data_resp = await tb.pe_row_data_resp.recv()
    col_data_resp = await tb.pe_col_data_resp.recv()
    col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

    assert row_data_resp.payload_Some_0_a == 0
    assert col_data_resp.payload_Some_0_b == 0
    assert col_data_resp.payload_Some_0_d.signed_integer == 0
    assert col_ctrl_resp.payload_Some_0_id == rnd_id
    assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == WS
    assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG2
    assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
    assert col_ctrl_resp.payload_Some_0_bad_dataflow == False

    # Input data and check output data
    for i in range(16):
        await tb.pe_row_data_req.send(
            PeDataRowTransaction(payload_Some_0_a=activation[i])
        )
        await tb.pe_col_data_req.send(
            PeDataColTransaction(payload_Some_0_b=0, payload_Some_0_d=0)
        )
        await tb.pe_col_ctrl_req.send(
            PeControlColTransaction(
                payload_Some_0_id=(rnd_id + 1) % 7,
                payload_Some_0_last=(i == rnd_last_idx),
                payload_Some_0_control_dataflow_discriminant=WS,
                payload_Some_0_control_propagate_discriminant=REG1,
                payload_Some_0_control_shift=rnd_shift,
                payload_Some_0_bad_dataflow=False,
            )
        )

        row_data_resp = await tb.pe_row_data_resp.recv()
        col_data_resp = await tb.pe_col_data_resp.recv()
        col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

        assert row_data_resp.payload_Some_0_a.signed_integer == activation[i]
        assert col_data_resp.payload_Some_0_b.signed_integer == output_data[i]
        assert col_data_resp.payload_Some_0_d == 0
        assert col_ctrl_resp.payload_Some_0_id == (rnd_id + 1) % 7
        assert col_ctrl_resp.payload_Some_0_last == (i == rnd_last_idx)
        assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == WS
        assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG1
        assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
        assert col_ctrl_resp.payload_Some_0_bad_dataflow == False


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def ws_random(dut):
    """
    Weight stationary testcase with random inputs
    """
    # Generate inputs for test
    rnd_last_idx = random.randint(0, 15)
    rnd_id = random.randint(0, 7)
    rnd_shift = random.randint(0, 10)

    activation = [random.randint(-(1 << 7), ((1 << 7) - 1)) for _ in range(16)]
    weight = random.randint(-(1 << 19), ((1 << 19) - 1))

    # Compute golden output
    golden_pe = PE()
    golden_pe.preload_weight(weight, REG2)
    output_data = []
    for i in range(16):
        output_data.append(golden_pe.compute_ws(activation[i], 0, 0, REG1))

    cocotb.log.info("== Weight Stationary testcase1 ==")
    cocotb.log.info(f"Last index: {rnd_last_idx}")
    cocotb.log.info(f"Random ID: {rnd_id}")
    cocotb.log.info(f"Random Shift: {rnd_shift}")

    cocotb.log.info(f"Input data: {activation}")
    cocotb.log.info(f"Weight data: {weight}")
    cocotb.log.info(f"Expected Output data: {output_data}")

    # Start test
    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    # Preload weight data
    req = PeControlColTransaction(
        payload_Some_0_id=rnd_id,
        payload_Some_0_last=random.randint(0, 1) > 0.5,
        payload_Some_0_control_dataflow_discriminant=WS,
        payload_Some_0_control_propagate_discriminant=REG2,
        payload_Some_0_control_shift=rnd_shift,
        payload_Some_0_bad_dataflow=False,
    )

    await tb.pe_row_data_req.send(PeDataRowTransaction(payload_Some_0_a=0))
    await tb.pe_col_data_req.send(
        PeDataColTransaction(
            payload_Some_0_b=0,
            payload_Some_0_d=weight,
        )
    )
    await tb.pe_col_ctrl_req.send(req)

    row_data_resp = await tb.pe_row_data_resp.recv()
    col_data_resp = await tb.pe_col_data_resp.recv()
    col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

    assert row_data_resp.payload_Some_0_a == 0
    assert col_data_resp.payload_Some_0_b == 0
    assert col_data_resp.payload_Some_0_d.signed_integer == 0
    assert col_ctrl_resp.payload_Some_0_id == rnd_id
    assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == WS
    assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG2
    assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
    assert col_ctrl_resp.payload_Some_0_bad_dataflow == False

    # Input data and check output data
    for i in range(16):
        await tb.pe_row_data_req.send(
            PeDataRowTransaction(payload_Some_0_a=activation[i])
        )
        await tb.pe_col_data_req.send(
            PeDataColTransaction(payload_Some_0_b=0, payload_Some_0_d=0)
        )
        await tb.pe_col_ctrl_req.send(
            PeControlColTransaction(
                payload_Some_0_id=(rnd_id + 1) % 7,
                payload_Some_0_last=(i == rnd_last_idx),
                payload_Some_0_control_dataflow_discriminant=WS,
                payload_Some_0_control_propagate_discriminant=REG1,
                payload_Some_0_control_shift=rnd_shift,
                payload_Some_0_bad_dataflow=False,
            )
        )

        row_data_resp = await tb.pe_row_data_resp.recv()
        col_data_resp = await tb.pe_col_data_resp.recv()
        col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

        assert row_data_resp.payload_Some_0_a.signed_integer == activation[i]
        assert col_data_resp.payload_Some_0_b.signed_integer == output_data[i]
        assert col_data_resp.payload_Some_0_d == 0
        assert col_ctrl_resp.payload_Some_0_id == (rnd_id + 1) % 7
        assert col_ctrl_resp.payload_Some_0_last == (i == rnd_last_idx)
        assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == WS
        assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG1
        assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
        assert col_ctrl_resp.payload_Some_0_bad_dataflow == False


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def os_simple(dut):
    """
    Output stationary testcase with simple configuration. (Simple input, no shift)
    """

    # Generate inputs for test
    rnd_id = random.randint(0, 7)
    rnd_shift = 0

    activation = [1] * 16
    weight = [1] * 16

    # Compute golden output
    golden_pe = PE()
    for act, w in zip(activation, weight):
        output_data = golden_pe.compute_os(act, w, 0, REG1, rnd_shift)

    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    cocotb.log.info("== Output Stationary testcase1 ==")
    cocotb.log.info(f"Random ID: {rnd_id}")
    cocotb.log.info(f"Random Shift: {rnd_shift}")

    cocotb.log.info(f"Input data: {activation}")
    cocotb.log.info(f"Weight data: {weight}")
    cocotb.log.info(f"Expected Output data: {output_data}")

    # Input data
    for i in range(16):
        await tb.pe_row_data_req.send(
            PeDataRowTransaction(payload_Some_0_a=activation[i])
        )
        await tb.pe_col_data_req.send(
            PeDataColTransaction(
                payload_Some_0_b=weight[i],
                payload_Some_0_d=0,
            )
        )
        await tb.pe_col_ctrl_req.send(
            PeControlColTransaction(
                payload_Some_0_id=rnd_id,
                payload_Some_0_last=(i == 15),
                payload_Some_0_control_dataflow_discriminant=OS,
                payload_Some_0_control_propagate_discriminant=REG1,
                payload_Some_0_control_shift=rnd_shift,
                payload_Some_0_bad_dataflow=False,
            )
        )

        row_data_resp = await tb.pe_row_data_resp.recv()
        col_data_resp = await tb.pe_col_data_resp.recv()
        col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

        assert row_data_resp.payload_Some_0_a.signed_integer == activation[i]
        assert col_data_resp.payload_Some_0_b.signed_integer == weight[i]
        assert col_data_resp.payload_Some_0_d.signed_integer == 0
        assert col_ctrl_resp.payload_Some_0_id == rnd_id
        assert col_ctrl_resp.payload_Some_0_last == (i == 15)
        assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == OS
        assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG1
        assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
        assert col_ctrl_resp.payload_Some_0_bad_dataflow == False

    # Check output data
    await tb.pe_row_data_req.send(PeDataRowTransaction(payload_Some_0_a=0))
    await tb.pe_col_data_req.send(
        PeDataColTransaction(payload_Some_0_b=0, payload_Some_0_d=0)
    )
    await tb.pe_col_ctrl_req.send(
        PeControlColTransaction(
            payload_Some_0_id=(rnd_id + 1) % 7,
            payload_Some_0_last=False,
            payload_Some_0_control_dataflow_discriminant=OS,
            payload_Some_0_control_propagate_discriminant=REG2,
            payload_Some_0_control_shift=rnd_shift,
            payload_Some_0_bad_dataflow=False,
        )
    )

    row_data_resp = await tb.pe_row_data_resp.recv()
    col_data_resp = await tb.pe_col_data_resp.recv()
    col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

    assert row_data_resp.payload_Some_0_a == 0
    assert col_data_resp.payload_Some_0_b == 0
    assert col_data_resp.payload_Some_0_d.signed_integer == output_data
    assert col_ctrl_resp.payload_Some_0_id == (rnd_id + 1) % 7
    assert col_ctrl_resp.payload_Some_0_last == False
    assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == OS
    assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG2
    assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
    assert col_ctrl_resp.payload_Some_0_bad_dataflow == False


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def os_random_shift(dut):
    """
    Output stationary testcase with random shift
    """

    # Generate inputs for test
    rnd_id = random.randint(0, 7)
    rnd_shift = 1

    activation = [i for i in range(16)]
    weight = [-i for i in range(16)]

    # Compute golden output
    golden_pe = PE()
    for act, w in zip(activation, weight):
        output_data = golden_pe.compute_os(act, w, 0, REG1, rnd_shift)

    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    cocotb.log.info("== Output Stationary testcase1 ==")
    cocotb.log.info(f"Random ID: {rnd_id}")
    cocotb.log.info(f"Random Shift: {rnd_shift}")

    cocotb.log.info(f"Input data: {activation}")
    cocotb.log.info(f"Weight data: {weight}")
    cocotb.log.info(f"Expected Output data: {output_data}")

    # Input data
    for i in range(16):
        await tb.pe_row_data_req.send(
            PeDataRowTransaction(payload_Some_0_a=activation[i])
        )
        await tb.pe_col_data_req.send(
            PeDataColTransaction(
                payload_Some_0_b=weight[i],
                payload_Some_0_d=0,
            )
        )
        await tb.pe_col_ctrl_req.send(
            PeControlColTransaction(
                payload_Some_0_id=rnd_id,
                payload_Some_0_last=(i == 15),
                payload_Some_0_control_dataflow_discriminant=OS,
                payload_Some_0_control_propagate_discriminant=REG1,
                payload_Some_0_control_shift=rnd_shift,
                payload_Some_0_bad_dataflow=False,
            )
        )

        row_data_resp = await tb.pe_row_data_resp.recv()
        col_data_resp = await tb.pe_col_data_resp.recv()
        col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

        assert row_data_resp.payload_Some_0_a.signed_integer == activation[i]
        assert col_data_resp.payload_Some_0_b.signed_integer == weight[i]
        assert col_data_resp.payload_Some_0_d.signed_integer == 0
        assert col_ctrl_resp.payload_Some_0_id == rnd_id
        assert col_ctrl_resp.payload_Some_0_last == (i == 15)
        assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == OS
        assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG1
        assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
        assert col_ctrl_resp.payload_Some_0_bad_dataflow == False

    # Check output data
    await tb.pe_row_data_req.send(PeDataRowTransaction(payload_Some_0_a=0))
    await tb.pe_col_data_req.send(
        PeDataColTransaction(payload_Some_0_b=0, payload_Some_0_d=0)
    )
    await tb.pe_col_ctrl_req.send(
        PeControlColTransaction(
            payload_Some_0_id=(rnd_id + 1) % 7,
            payload_Some_0_last=False,
            payload_Some_0_control_dataflow_discriminant=OS,
            payload_Some_0_control_propagate_discriminant=REG2,
            payload_Some_0_control_shift=rnd_shift,
            payload_Some_0_bad_dataflow=False,
        )
    )

    row_data_resp = await tb.pe_row_data_resp.recv()
    col_data_resp = await tb.pe_col_data_resp.recv()
    col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

    assert row_data_resp.payload_Some_0_a == 0
    assert col_data_resp.payload_Some_0_b == 0
    assert col_data_resp.payload_Some_0_d.signed_integer == output_data
    assert col_ctrl_resp.payload_Some_0_id == (rnd_id + 1) % 7
    assert col_ctrl_resp.payload_Some_0_last == False
    assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == OS
    assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG2
    assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
    assert col_ctrl_resp.payload_Some_0_bad_dataflow == False


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def os_random_inp_and_shift(dut):
    """
    Output stationary testcase with random inputs
    """

    # Generate inputs for test
    rnd_id = random.randint(0, 7)
    rnd_shift = 0

    activation = [random.randint(-(1 << 7), ((1 << 7) - 1)) for _ in range(16)]
    weight = [random.randint(-(1 << 19), ((1 << 19) - 1)) for _ in range(16)]

    # Compute golden output
    golden_pe = PE()
    for act, w in zip(activation, weight):
        output_data = golden_pe.compute_os(act, w, 0, REG1, rnd_shift)

    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    cocotb.log.info("== Output Stationary testcase1 ==")
    cocotb.log.info(f"Random ID: {rnd_id}")
    cocotb.log.info(f"Random Shift: {rnd_shift}")

    cocotb.log.info(f"Input data: {activation}")
    cocotb.log.info(f"Weight data: {weight}")
    cocotb.log.info(f"Expected Output data: {output_data}")

    # Input data
    for i in range(16):
        await tb.pe_row_data_req.send(
            PeDataRowTransaction(payload_Some_0_a=activation[i])
        )
        await tb.pe_col_data_req.send(
            PeDataColTransaction(
                payload_Some_0_b=weight[i],
                payload_Some_0_d=0,
            )
        )
        await tb.pe_col_ctrl_req.send(
            PeControlColTransaction(
                payload_Some_0_id=rnd_id,
                payload_Some_0_last=(i == 15),
                payload_Some_0_control_dataflow_discriminant=OS,
                payload_Some_0_control_propagate_discriminant=REG1,
                payload_Some_0_control_shift=rnd_shift,
                payload_Some_0_bad_dataflow=False,
            )
        )

        row_data_resp = await tb.pe_row_data_resp.recv()
        col_data_resp = await tb.pe_col_data_resp.recv()
        col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

        assert row_data_resp.payload_Some_0_a.signed_integer == activation[i]
        assert col_data_resp.payload_Some_0_b.signed_integer == weight[i]
        assert col_data_resp.payload_Some_0_d.signed_integer == 0
        assert col_ctrl_resp.payload_Some_0_id == rnd_id
        assert col_ctrl_resp.payload_Some_0_last == (i == 15)
        assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == OS
        assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG1
        assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
        assert col_ctrl_resp.payload_Some_0_bad_dataflow == False

    # Check output data
    await tb.pe_row_data_req.send(PeDataRowTransaction(payload_Some_0_a=0))
    await tb.pe_col_data_req.send(
        PeDataColTransaction(payload_Some_0_b=0, payload_Some_0_d=0)
    )
    await tb.pe_col_ctrl_req.send(
        PeControlColTransaction(
            payload_Some_0_id=(rnd_id + 1) % 7,
            payload_Some_0_last=False,
            payload_Some_0_control_dataflow_discriminant=OS,
            payload_Some_0_control_propagate_discriminant=REG2,
            payload_Some_0_control_shift=rnd_shift,
            payload_Some_0_bad_dataflow=False,
        )
    )

    row_data_resp = await tb.pe_row_data_resp.recv()
    col_data_resp = await tb.pe_col_data_resp.recv()
    col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

    assert row_data_resp.payload_Some_0_a == 0
    assert col_data_resp.payload_Some_0_b == 0
    assert col_data_resp.payload_Some_0_d.signed_integer == output_data
    assert col_ctrl_resp.payload_Some_0_id == (rnd_id + 1) % 7
    assert col_ctrl_resp.payload_Some_0_last == False
    assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == OS
    assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG2
    assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
    assert col_ctrl_resp.payload_Some_0_bad_dataflow == False


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def os_random_inp(dut):
    """
    Output stationary testcase with random inputs and random shift
    """

    # Generate inputs for test
    rnd_id = random.randint(0, 7)
    rnd_shift = 0

    activation = [random.randint(-(1 << 7), ((1 << 7) - 1)) for _ in range(16)]
    weight = [random.randint(-(1 << 19), ((1 << 19) - 1)) for _ in range(16)]

    # Compute golden output
    golden_pe = PE()
    for act, w in zip(activation, weight):
        output_data = golden_pe.compute_os(act, w, 0, REG1, rnd_shift)

    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    cocotb.log.info("== Output Stationary testcase1 ==")
    cocotb.log.info(f"Random ID: {rnd_id}")
    cocotb.log.info(f"Random Shift: {rnd_shift}")

    cocotb.log.info(f"Input data: {activation}")
    cocotb.log.info(f"Weight data: {weight}")
    cocotb.log.info(f"Expected Output data: {output_data}")

    # Input data
    for i in range(16):
        await tb.pe_row_data_req.send(
            PeDataRowTransaction(payload_Some_0_a=activation[i])
        )
        await tb.pe_col_data_req.send(
            PeDataColTransaction(
                payload_Some_0_b=weight[i],
                payload_Some_0_d=0,
            )
        )
        await tb.pe_col_ctrl_req.send(
            PeControlColTransaction(
                payload_Some_0_id=rnd_id,
                payload_Some_0_last=(i == 15),
                payload_Some_0_control_dataflow_discriminant=OS,
                payload_Some_0_control_propagate_discriminant=REG1,
                payload_Some_0_control_shift=rnd_shift,
                payload_Some_0_bad_dataflow=False,
            )
        )

        row_data_resp = await tb.pe_row_data_resp.recv()
        col_data_resp = await tb.pe_col_data_resp.recv()
        col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

        assert row_data_resp.payload_Some_0_a.signed_integer == activation[i]
        assert col_data_resp.payload_Some_0_b.signed_integer == weight[i]
        assert col_data_resp.payload_Some_0_d.signed_integer == 0
        assert col_ctrl_resp.payload_Some_0_id == rnd_id
        assert col_ctrl_resp.payload_Some_0_last == (i == 15)
        assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == OS
        assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG1
        assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
        assert col_ctrl_resp.payload_Some_0_bad_dataflow == False

    # Check output data
    await tb.pe_row_data_req.send(PeDataRowTransaction(payload_Some_0_a=0))
    await tb.pe_col_data_req.send(
        PeDataColTransaction(payload_Some_0_b=0, payload_Some_0_d=0)
    )
    await tb.pe_col_ctrl_req.send(
        PeControlColTransaction(
            payload_Some_0_id=(rnd_id + 1) % 7,
            payload_Some_0_last=False,
            payload_Some_0_control_dataflow_discriminant=OS,
            payload_Some_0_control_propagate_discriminant=REG2,
            payload_Some_0_control_shift=rnd_shift,
            payload_Some_0_bad_dataflow=False,
        )
    )

    row_data_resp = await tb.pe_row_data_resp.recv()
    col_data_resp = await tb.pe_col_data_resp.recv()
    col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

    assert row_data_resp.payload_Some_0_a == 0
    assert col_data_resp.payload_Some_0_b == 0
    assert col_data_resp.payload_Some_0_d.signed_integer == output_data
    assert col_ctrl_resp.payload_Some_0_id == (rnd_id + 1) % 7
    assert col_ctrl_resp.payload_Some_0_last == False
    assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == OS
    assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG2
    assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
    assert col_ctrl_resp.payload_Some_0_bad_dataflow == False


@cocotb.test(timeout_time=10, timeout_unit="ms")
async def os_random_inp_and_shift(dut):
    """
    Output stationary testcase with random inputs and random shift
    """

    # Generate inputs for test
    rnd_id = random.randint(0, 7)
    rnd_shift = random.randint(1, 3)

    activation = [random.randint(-(1 << 7), ((1 << 7) - 1)) for _ in range(16)]
    weight = [random.randint(-(1 << 19), ((1 << 19) - 1)) for _ in range(16)]

    # Compute golden output
    golden_pe = PE()
    for act, w in zip(activation, weight):
        output_data = golden_pe.compute_os(act, w, 0, REG1, rnd_shift)

    tb = TB(dut)

    await tb.reset()
    await ClockCycles(dut.clk, 10)

    cocotb.log.info("== Output Stationary testcase1 ==")
    cocotb.log.info(f"Random ID: {rnd_id}")
    cocotb.log.info(f"Random Shift: {rnd_shift}")

    cocotb.log.info(f"Input data: {activation}")
    cocotb.log.info(f"Weight data: {weight}")
    cocotb.log.info(f"Expected Output data: {output_data}")

    # Input data
    for i in range(16):
        await tb.pe_row_data_req.send(
            PeDataRowTransaction(payload_Some_0_a=activation[i])
        )
        await tb.pe_col_data_req.send(
            PeDataColTransaction(
                payload_Some_0_b=weight[i],
                payload_Some_0_d=0,
            )
        )
        await tb.pe_col_ctrl_req.send(
            PeControlColTransaction(
                payload_Some_0_id=rnd_id,
                payload_Some_0_last=(i == 15),
                payload_Some_0_control_dataflow_discriminant=OS,
                payload_Some_0_control_propagate_discriminant=REG1,
                payload_Some_0_control_shift=rnd_shift,
                payload_Some_0_bad_dataflow=False,
            )
        )

        row_data_resp = await tb.pe_row_data_resp.recv()
        col_data_resp = await tb.pe_col_data_resp.recv()
        col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

        assert row_data_resp.payload_Some_0_a.signed_integer == activation[i]
        assert col_data_resp.payload_Some_0_b.signed_integer == weight[i]
        assert col_data_resp.payload_Some_0_d.signed_integer == 0
        assert col_ctrl_resp.payload_Some_0_id == rnd_id
        assert col_ctrl_resp.payload_Some_0_last == (i == 15)
        assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == OS
        assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG1
        assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
        assert col_ctrl_resp.payload_Some_0_bad_dataflow == False

    # Check output data
    await tb.pe_row_data_req.send(PeDataRowTransaction(payload_Some_0_a=0))
    await tb.pe_col_data_req.send(
        PeDataColTransaction(payload_Some_0_b=0, payload_Some_0_d=0)
    )
    await tb.pe_col_ctrl_req.send(
        PeControlColTransaction(
            payload_Some_0_id=(rnd_id + 1) % 7,
            payload_Some_0_last=False,
            payload_Some_0_control_dataflow_discriminant=OS,
            payload_Some_0_control_propagate_discriminant=REG2,
            payload_Some_0_control_shift=rnd_shift,
            payload_Some_0_bad_dataflow=False,
        )
    )

    row_data_resp = await tb.pe_row_data_resp.recv()
    col_data_resp = await tb.pe_col_data_resp.recv()
    col_ctrl_resp = await tb.pe_col_ctrl_resp.recv()

    assert row_data_resp.payload_Some_0_a == 0
    assert col_data_resp.payload_Some_0_b == 0
    assert col_data_resp.payload_Some_0_d.signed_integer == output_data
    assert col_ctrl_resp.payload_Some_0_id == (rnd_id + 1) % 7
    assert col_ctrl_resp.payload_Some_0_last == False
    assert col_ctrl_resp.payload_Some_0_control_dataflow_discriminant == OS
    assert col_ctrl_resp.payload_Some_0_control_propagate_discriminant == REG2
    assert col_ctrl_resp.payload_Some_0_control_shift == rnd_shift
    assert col_ctrl_resp.payload_Some_0_bad_dataflow == False
