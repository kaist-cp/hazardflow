#!/usr/bin/env python3

import os
import sys
import subprocess
from pathlib import Path
from rich import print
from rich.console import Console

from trace import *
from cpi import *
from constants import *


console = Console()


def run_tests(total_tests):
    count = 0

    for tb in bench_dir.iterdir():
        tb_filename = tb.name
        if tb.is_file() and not tb.suffix in [".dump", ".trace"]:
            count += 1
            vcd_option = f"-v{log_dir}/{tb_filename}.vcd" if waves_flag else ""
            txt_file = log_dir / f"{tb_filename}.txt"

            console.print(f"Extract trace from benchmark ({count}/{total_tests}): {tb_filename} .. ", end="")
            result = subprocess.run(
                f"{emulator} {vcd_option} +max-cycles=100000 {tb}",
                stdout=open(txt_file, "w"),
                stderr=subprocess.STDOUT,
                shell=True
            )

            console.print("DONE")


if __name__ ==  "__main__":
    # Current file absolute directory path
    curr_dir = Path(__file__).resolve().parent
    log_dir = curr_dir / "output"
    emulator = curr_dir / "emulator-debug"
    bench_dir = curr_dir / "program/bench"

    # Check if the emulator exists
    if not emulator.is_file():
        logger.error(f"{emulator} does not exist.")
        logger.error("Please run `python3 scripts/cpu/build.py` first.")
        sys.exit(1)

    # Flags
    trace_flag = False
    cpi_flag = False
    waves_flag = False

    # Parse arguments
    for arg in sys.argv[1:]:
        if arg == "trace":
            trace_flag = True
        elif arg == "cpi":
            cpi_flag = True
        elif arg == "--waves":
            waves_flag = True

    # Ensure log directory exists
    log_dir.mkdir(parents=True, exist_ok=True)

    # Running tests
    if trace_flag:
        logger.info("Running benchmark trace tests")
        run_tests(9)
        check_trace()
    elif cpi_flag:
        logger.info("Running benchmark cpi tests")
        run_tests(9)
        calculate_cpi_hf("branch_prediction")
