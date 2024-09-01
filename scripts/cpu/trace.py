#!/usr/bin/env python3

from constants import *
from parse import compile


def check_trace():
    """
    Check the trace
    Compare data in register file every cycle.
    """
    logger.info("Trace check start")

    orig_trace_dir = f"{cpu_script_dir}/program/bench"
    hf_trace_dir = f"{cpu_script_dir}/output"

    log_template = compile("Reg[{}]: [{}] -> [{}]\n")
    hf_reg_template = compile("[{}] rf[{}]: {}\n")

    failed = False
    for bench in BENCHES:
        logger.info(f"[Check RF Trace] {bench} START")

        # File paths
        orig_trace_log = f"{orig_trace_dir}/{bench}.trace"
        hf_raw_log = f"{hf_trace_dir}/{bench}.txt"
        hf_trace_log = f"{hf_trace_dir}/{bench}.trace"

        # Initialize register file
        hf_rf = {}
        [hf_rf.setdefault(str(i), "00000000") for i in range(32)]

        with open(hf_raw_log, "r") as hf, open(hf_trace_log, "w") as hf_parsed:
            for line in hf:
                if "rf" in line:
                    parsed = hf_reg_template.parse(line)
                    _tick = parsed[0]
                    addr = parsed[1]
                    data = parsed[2]

                    if hf_rf[addr] != data:
                        hf_parsed.write(f"Reg[{addr}]: [{hf_rf[addr]}] -> [{data}]\n")
                        hf_rf[addr] = data

        with open(orig_trace_log, "r") as orig, open(hf_trace_log, "r") as hf:
            lines = 0

            while True:
                if lines > 10000:
                    break

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

                lines += 1
            logger.info(f"[Check RF Trace] {bench} END")

    if failed:
        exit(1)

if __name__ == "__main__":
    check_trace()
