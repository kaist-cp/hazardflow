#!/usr/bin/env python3

import os
from os.path import dirname, join
import pathlib
from enum import Enum
import logging
from rich.logging import RichHandler

BENCHES = [
    "aes",
    "coremark",
    "gemm",
    "radix",
]
# Cached CPI values for baseline
BASELINE_CPI = {
    "aes": 1.1996944565691838,
    "coremark": 1.522181254945536,
    "gemm": 1.5291416003622484,
    "radix": 1.2878887138814936,
}
# Cached CPI values for branch prediction
BRANCH_PREDICTION_CPI = {
    "aes": 1.0727827413379822,
    "coremark": 1.1978880983610902,
    "gemm": 1.1815364622382167,
    "radix": 1.0882910684208666,
}

FORMAT = "%(message)s"  # Logger format
# Set logger level
logging.basicConfig(
    level="NOTSET", format=FORMAT, datefmt="[%X]", handlers=[RichHandler()]
)  # set level=20 or logging.INFO to turn off debug
logger = logging.getLogger("rich")

hazardflow_dir = hazardflow_dir = pathlib.Path(__file__).absolute().parent.parent.parent
cpu_script_dir = hazardflow_dir / "scripts" / "cpu"

openroad_flow = os.environ.get("OPENROAD_FLOW")

sodor_dir = hazardflow_dir / "riscv-sodor"

sodor_src_dir = sodor_dir / "src"
chisel_core_src_dir = sodor_src_dir / "rv32_5stage"
hf_core_src_dir = sodor_src_dir / "rv32_5stage_hf"

sodor_emulator_dir = sodor_dir / "emulator"
common_core_emulator_dir = sodor_emulator_dir / "common"
chisel_core_emulator_dir = sodor_emulator_dir / "rv32_5stage"
hf_core_emulator_dir = sodor_emulator_dir / "rv32_5stage_hf"

SBT_BUILD_TXT = b'lazy val rv32_5stage_hf = (project in file("src/rv32_5stage_hf")).\n\tsettings(commonSettings: _*).\n\tsettings(chipSettings: _*).\n\tdependsOn(common)'


class CoreType(Enum):
    # Sodor written in Chisel
    CHISEL = 1
    # Sodor written in Hazardflow
    HAZARDFLOW = 2


help = """TODO"""
