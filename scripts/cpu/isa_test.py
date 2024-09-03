#!/usr/bin/env python3

import os
import sys
import subprocess
from pathlib import Path
from rich import print
from rich.console import Console

from constants import *


console = Console()


def run_tests(test_dir, total_tests):
    count = 0
    count_passed = 0
    count_failed = 0

    for tb in test_dir.iterdir():
        tb_filename = tb.name
        if tb.is_file() and not tb.suffix == ".dump":
            count += 1
            vcd_option = f"-v{log_dir}/{tb_filename}.vcd" if waves_flag else ""
            txt_file = log_dir / f"{tb_filename}.txt"

            console.print(f"Test ({count}/{total_tests}): {tb_filename} .. ", end="")
            result = subprocess.run(
                f"{emulator} {vcd_option} +max-cycles=100000 {tb}",
                stdout=open(txt_file, "w"),
                stderr=subprocess.STDOUT,
                shell=True,
            )

            if result.returncode == 0:
                console.print("[green]PASSED[/green]")
                count_passed += 1
            else:
                console.print("[red]FAILED[/red]")
                count_failed += 1

    logger.info(f"Number of success tests: {count_passed} / {total_tests}")

    if count_failed > 0:
        logger.error(f"You can check the log file for failed test cases in `{cpu_script_dir}/output` directory.")
        sys.exit(1)


if __name__ == "__main__":
    # Setup
    curr_dir = Path(__file__).resolve().parent
    log_dir = curr_dir / "output"
    emulator = curr_dir / "emulator-debug"

    # Check if the emulator exists
    if not emulator.is_file():
        logger.error(f"{emulator} does not exist.")
        logger.error("Please run `python3 scripts/cpu/build.py` first.")
        sys.exit(1)

    # Flags
    base_flag = False
    mext_flag = False
    waves_flag = False

    # Parse arguments
    for arg in sys.argv[1:]:
        if arg == "base":
            base_flag = True
        elif arg == "mext":
            mext_flag = True
        elif arg == "--waves":
            waves_flag = True

    # Ensure log directory exists
    log_dir.mkdir(parents=True, exist_ok=True)

    # Running tests
    if base_flag:
        logger.info("Running base ISA tests")
        base_dir = curr_dir / "program/isa/base"
        run_tests(base_dir, 43)

    elif mext_flag:
        logger.info("Running M-extension ISA tests")
        mext_dir = curr_dir / "program/isa/mext"
        run_tests(mext_dir, 8)
