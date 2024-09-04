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
    "ellpack",
    "gemm-block",
    "gemm",
    "kmp",
    "nw",
    "queue",
    "radix",
]
# Cached CPI values for baseline
BASELINE_CPI = {
    "aes": 1.2000873267110577,
    "coremark": 1.5222068826183581,
    "ellpack": 1.3759583636465387,
    "gemm-block": 1.5248672242888168,
    "gemm": 1.5291745730550284,
    "kmp": 1.496593118287688,
    "nw": 1.3411507976321861,
    "queue": 1.3322280857423061,
    "radix": 1.2879425703930862,
}
# Cached CPI values for branch prediction
BRANCH_PREDICTION_CPI = {
    "aes": 1.073133514986376,
    "coremark": 1.1979357129607546,
    "ellpack": 1.057687344059193,
    "gemm-block": 1.1849320790656486,
    "gemm": 1.181595521343597,
    "kmp": 1.0109134952508447,
    "nw": 1.073114438245093,
    "queue": 1.1186424853535,
    "radix": 1.0883295248415745,
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
