#!/usr/bin/env python3

import sys
import re
from parse import compile
import math

from constants import *

# Match on committed instructions
CHISEL_INST_RE = re.compile(r"[^\[]*\[1\].*DASM\(([0-9A-Fa-f]+)\)")


class SodorCpiCalculator:
    # Initialize your new counters to 0 here
    def __init__(self):
        self.inst_count = 0
        self.cycles = 0

    def reset(self):
        self.inst_count = 0
        self.cycles = 0

    # Increment your counters as appropriate here
    def retire(self):
        self.inst_count += 1
        self.cycles += 1

    def bubble(self):
        self.cycles += 1

    def cpi(self) -> float:
        return float(self.cycles) / self.inst_count


def calculate_cpi_hf(arg):
    tracer = SodorCpiCalculator()
    hf_retire_template = compile("[{}] retire: [{}], pc: [{}]")

    failed = False
    for bench in BENCHES:
        start_benchmark = False
        logger.info(f"Start Calculating CPI of {bench}")
        tracer.reset()
        file = f"{cpu_script_dir}/output/{bench}.txt"
        with open(file, "r") as f:
            for line in f:
                line = line.strip()
                if "retire" in line:
                    parsed = hf_retire_template.parse(line)
                    pc = parsed[2]
                    if pc == "80000000":
                        start_benchmark = True
                    if not start_benchmark:
                        continue
                    retired = int(parsed[1])
                    if retired:
                        tracer.retire()
                    else:
                        tracer.bubble()
        cpi = tracer.cpi()

        ratio = cpi / BASELINE_CPI[bench]
        logger.info(f"CPI result of benchmark {bench}: {cpi} ({ratio:.2f} times of baseline CPI {BASELINE_CPI[bench]})")

        if arg == "branch_prediction":
            if not math.isclose(cpi, BRANCH_PREDICTION_CPI[bench], abs_tol=0.01):
                logger.error(f"CPI result is not expected (expected: {BRANCH_PREDICTION_CPI[bench]})")
        elif arg == "baseline":
            if not math.isclose(cpi, BASELINE_CPI[bench], abs_tol=0.01):
                logger.error(f"CPI result is not expected (expected: {BASELINE_CPI[bench]})")

    if failed:
        exit(1)


if __name__ == "__main__":
    calculate_cpi_hf(sys.argv[1])
