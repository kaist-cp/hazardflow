#!/usr/bin/env python3

import sys
import re
from parse import compile
import math

from constants import *


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


def calculate_cpi(arg):
    if arg == "bp":
        logger.info("Comparing CPI with branch prediction")
    else:
        logger.info("Comparing CPI with baseline")

    tracer = SodorCpiCalculator()

    hf_retire_template = compile("[{}] retire=[1] pc=[{}]{}\n")

    count = 0
    count_passed = 0
    count_failed = 0

    for bench in BENCHES:
        start_benchmark = False
        logger.info(f"Start Calculating CPI of {bench}")
        tracer.reset()
        file = f"{cpu_script_dir}/output/{bench}.txt"
        with open(file, "r") as f:
            for line in f:
                if "retire=[1]" in line:
                    parsed = hf_retire_template.parse(line)
                    pc = parsed[1]
                    if pc == "80000000":
                        start_benchmark = True
                    if not start_benchmark:
                        continue
                    tracer.retire()
                elif "retire=[0]" in line:
                    if not start_benchmark:
                        continue
                    tracer.bubble()
                else:
                    pass
        cpi = tracer.cpi()

        ratio = cpi / BASELINE_CPI[bench]
        logger.info(f"CPI result of benchmark {bench}: {cpi} ({ratio:.2f} times of baseline CPI {BASELINE_CPI[bench]})")

        if arg == "bp":
            if math.isclose(cpi, BRANCH_PREDICTION_CPI[bench], abs_tol=0.01):
                count_passed += 1
            else:
                logger.error(f"CPI result is not expected (expected: {BRANCH_PREDICTION_CPI[bench]})")
                count_failed += 1
        else:
            if math.isclose(cpi, BASELINE_CPI[bench], abs_tol=0.01):
                count_passed += 1
            else:
                logger.error(f"CPI result is not expected (expected: {BASELINE_CPI[bench]})")
                count_failed += 1

    logger.info(f"Number of success tests: {count_passed} / {len(BENCHES)}")

    if count_failed > 0:
        logger.error(f"You can check the log file for failed test cases in `{cpu_script_dir}/output` directory.")
        sys.exit(1)


if __name__ == "__main__":
    calculate_cpi(sys.argv[1])
