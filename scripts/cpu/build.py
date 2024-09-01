#!/usr/bin/env python3

import subprocess

from setup import *
from utils import *
from constants import *


def build_core(core: CoreType):
    """
    Generate Verilog and Simulator
    """
    if core == CoreType.CHISEL:
        logger.info(f"Start generating Verilog code and building Emulator")
        subprocess.run(
            ["make", "emulator-debug"],
            cwd=f"{sodor_dir}/emulator/rv32_5stage",
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        logger.info(f"Emulator generated at {sodor_dir}/emulator/rv32_5stage")
    elif core == CoreType.HAZARDFLOW:
        logger.info(f"Start generating Verilog code and building Emulator")
        subprocess.run(
            ["cargo", "run", "--release", "--", "--system-task", "--deadcode", "--wire-cache", "--target", "cpu"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            cwd=hazardflow_dir,
        )
        logger.info(
            f"[HAZARDFLOW] Verilog code compiled at {hazardflow_dir}/build/core"
        )

        subprocess.run(
            f"cp {hazardflow_dir}/build/core/*.v {sodor_dir}/vsrc", shell=True
        )
        subprocess.run(
            ["make", "emulator-debug"], cwd=f"{sodor_dir}/emulator/rv32_5stage_hf"
        )
        subprocess.run(
            f"cp {sodor_dir}/emulator/rv32_5stage_hf/emulator-debug {cpu_script_dir}", shell=True
        )
        logger.info(f"Emulator generated at {cpu_script_dir}")
    else:
        raise Exception("Invalid core type")


if __name__ == "__main__":
    setup()
    build_core(CoreType.HAZARDFLOW)
