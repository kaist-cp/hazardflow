#!/usr/bin/env python3

import sys

from constants import *
from parse import compile


def check_trace():
    """
    Check the register file update trace.
    It does not check in cycle-accurate.
    """
    logger.info("Trace check start")

    orig_trace_dir = f"{cpu_script_dir}/logs/trace"
    hf_trace_dir = f"{cpu_script_dir}/output"

    log_template = compile("Reg[{}]: [{}] -> [{}]\n")
    hf_reg_write_template = compile("[{}] retire=[1] pc=[{}] inst=[{}] write=[r{}={}]\n")

    count = 0
    count_passed = 0
    count_failed = 0

    for bench in BENCHES:
        failed = False
        logger.info(f"[Check RF Trace] {bench} START")

        # File paths
        orig_trace_log = f"{orig_trace_dir}/{bench}.trace"
        hf_raw_log = f"{hf_trace_dir}/{bench}.txt"
        hf_trace_log = f"{hf_trace_dir}/{bench}.trace"

        # Initialize register file
        hf_rf = {}
        [hf_rf.setdefault(i, "00000000") for i in range(32)]

        with open(hf_raw_log, "r") as hf, open(hf_trace_log, "w") as hf_parsed:
            lines = 0

            for line in hf:
                if lines >= 10000:
                    break

                if "write" in line:
                    parsed = hf_reg_write_template.parse(line)
                    _tick = parsed[0]
                    _pc = parsed[1]
                    _inst = parsed[2]
                    addr = int(parsed[3])
                    data = parsed[4]

                    if hf_rf[addr] != data:
                        hf_parsed.write(f"Reg[{addr}]: [{hf_rf[addr]}] -> [{data}]\n")
                        hf_rf[addr] = data
                        lines += 1

        with open(orig_trace_log, "r") as orig, open(hf_trace_log, "r") as hf:
            while True:
                orig_line = orig.readline()
                hf_line = hf.readline()

                if not orig_line and not hf_line:
                    break

                if not orig_line or not hf_line:
                    logger.error("Number of lines are different")
                    failed = True
                    break

                orig_parsed = log_template.parse(orig_line)
                orig_addr = orig_parsed[0]
                orig_old = orig_parsed[1]
                orig_new = orig_parsed[2]

                hf_parsed = log_template.parse(hf_line)
                hf_addr = hf_parsed[0]
                hf_old = hf_parsed[1]
                hf_new = hf_parsed[2]

                if (orig_addr != hf_addr) or (orig_old != hf_old) or (orig_new != hf_new):
                    logger.error(
                        f"({orig_addr}, {hf_addr})\t({orig_old}, {hf_old})\t({orig_new}, {hf_new})"
                    )
                    failed = True
                    break
            logger.info(f"[Check RF Trace] {bench} END")

        if failed:
            count_failed += 1
        else:
            count_passed += 1

    logger.info(f"Number of success tests: {count_passed} / {len(BENCHES)}")

    if count_failed > 0:
        logger.error(f"You can check the log file for failed test cases in `{cpu_script_dir}/output` directory.")
        sys.exit(1)

if __name__ == "__main__":
    check_trace()
