#!/usr/bin/env python3

import subprocess

from setup import *
from utils import *
from constants import *


def build_core():
    """
    Generate Verilog and Simulator
    """
    logger.info(f"Start generating Verilog code and building Emulator")
    subprocess.run(
        [
            "cargo",
            "run",
            "--release",
            "--",
            "--target",
            "core",
            "--wire-cache",
            "--deadcode",
            "--merge",
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        cwd=hazardflow_dir,
    )
    logger.info(f"[HAZARDFLOW] Verilog code compiled at {hazardflow_dir}/build/core")

    subprocess.run(f"cp {hazardflow_dir}/build/core/*.v {sodor_dir}/vsrc", shell=True)
    subprocess.run(["make", "emulator-debug"], cwd=f"{sodor_dir}/emulator/rv32_5stage_hf")

    subprocess.run(f"mkdir -p {cpu_script_dir}/openroad/vsrc", shell=True)
    subprocess.run(f"cp {sodor_dir}/hf_verilog/Core.v {cpu_script_dir}/openroad/vsrc", shell=True)
    subprocess.run(f"cp {sodor_dir}/vsrc/core_top.v {cpu_script_dir}/openroad/vsrc", shell=True)
    subprocess.run(f"cp {sodor_dir}/vsrc/CoreWrapper.v {cpu_script_dir}/openroad/vsrc", shell=True)
    subprocess.run(f"cp {sodor_dir}/vsrc/CSRFileWrapper.v {cpu_script_dir}/openroad/vsrc", shell=True)

    subprocess.run(f"cp {sodor_dir}/emulator/rv32_5stage_hf/emulator-debug {cpu_script_dir}", shell=True)
    logger.info(f"Emulator generated at {cpu_script_dir}")


if __name__ == "__main__":
    setup()
    build_core()
