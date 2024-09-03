#!/usr/bin/env python3

import os
import subprocess
from constants import *
from utils import *
from pathlib import Path


def setup_rust():
    subprocess.run(
        ["rustup", "component", "add", "rust-src", "rustc-dev", "llvm-tools-preview"],
        cwd=hazardflow_dir,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    subprocess.run(
        ["cargo", "build", "-p", "hazardflow-macro"],
        cwd=hazardflow_dir,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def setup():
    """
    Move and copy files to the proper locations.
    """
    logger.info("Setting up the container")

    # 0. Configure directory for hazardflow

    # Clone sodor repository.
    subprocess.run(f"rm -rf {sodor_dir}", shell=True)
    subprocess.run(
        [
            "git",
            "clone",
            "--depth",
            "1",
            "--branch",
            "sodor-old-fix",
            "https://github.com/minseongg/riscv-sodor.git",
            sodor_dir,
        ]
    )
    subprocess.run(
        ["git", "submodule", "update", "--init", "--recursive"], cwd=sodor_dir
    )
    subprocess.run(["./configure"], cwd=sodor_dir)

    # Prepare emulator and src directory for hazardflow.
    if not os.path.exists(hf_core_emulator_dir):
        subprocess.run(
            f"cp -r {chisel_core_emulator_dir} {hf_core_emulator_dir}",
            shell=True,
            check=True,
        )
        subprocess.run(
            [
                "sed",
                "-i",
                "s/rv32_5stage/rv32_5stage_hf/g",
                hf_core_emulator_dir / "Makefile",
            ],
            check=True,
        )
    if not os.path.exists(hf_core_src_dir):
        subprocess.run(
            f"cp -r {chisel_core_src_dir} {hf_core_src_dir}", shell=True, check=True
        )

    # 1. Cleanup
    for dir in [chisel_core_emulator_dir, hf_core_emulator_dir]:
        subprocess.run(["make", "clean"], cwd=dir)
        create_dir(dir / "output")

    # Copy sodor files to rv32_5stage
    subprocess.run(
        f"cp regfile.scala {chisel_core_src_dir}/regfile.scala",
        shell=True,
        cwd=cpu_script_dir / "sodor_files",
    )

    subprocess.run(
        f"cp core.scala {hf_core_src_dir}/core.scala",
        shell=True,
        cwd=cpu_script_dir / "sodor_files",
    )
    core_file_path = f"{sodor_dir}/vsrc/CoreWrapper.v".replace("/", "\/")
    subprocess.run(
        [
            "sed",
            "-i",
            f"s/COREWRAPPERPATH/{core_file_path}/",
            f"{hf_core_src_dir}/core.scala",
        ]
    )
    subprocess.run(
        f"cp hf_top.scala {hf_core_src_dir}/top.scala",
        shell=True,
        cwd=cpu_script_dir / "sodor_files",
    )

    # Copy wrappers of CPU
    subprocess.run(
        f"cp -a {cpu_script_dir}/wrappers/*.v {sodor_dir}/vsrc", shell=True, check=True
    )

    append_txt_if_keyword_doesnt_exist(
        sodor_dir / "build.sbt", "rv32_5stage_hf", SBT_BUILD_TXT
    )

    subprocess.run(
        f"sed -i 1s/^/#include\ \<stdexcept\>\\\\n/ {sodor_dir}/riscv-isa-sim/fesvr/dtm.cc",
        shell=True,
        check=True,
    )
    # mute false alarm from Verilator
    emulator_common_makefile = common_core_emulator_dir / "Makefile.include"

    verilator_version = check_verilator_version()
    if verilator_version >= 5:
        # TODO: Is this really a good approach?
        subprocess.run(
            f"sed -i 's/VERILATOR := verilator --cc --exe /VERILATOR := verilator -Wno-UNOPTFLAT -Wno-STMTDLY -Wno-TIMESCALEMOD --no-timing --cc --exe/' {emulator_common_makefile}",
            shell=True,
        )
    elif verilator_version >= 4.03:
        subprocess.run(
            f"sed -i 's/VERILATOR := verilator --cc --exe /VERILATOR := verilator -Wno-UNOPTFLAT -Wno-STMTDLY -Wno-TIMESCALEMOD --cc --exe/' {emulator_common_makefile}",
            shell=True,
        )
    else:
        subprocess.run(
            f"sed -i 's/VERILATOR := verilator --cc --exe /VERILATOR := verilator -Wno-UNOPTFLAT --cc --exe/' {emulator_common_makefile}",
            shell=True,
        )
    # For fast emulator compilation and compatibility
    subprocess.run(
        f"sed -i 's/CXXFLAGS += -O1 -std=c++11 -g/CXXFLAGS += -O0 -std=c++17/' {emulator_common_makefile}",
        shell=True,
        check=True,
    )

    # remove CPath and DPath
    subprocess.run(["rm", "-f", "cpath.scala", "dpath.scala"], cwd=hf_core_src_dir)

    setup_rust()
    logger.info("Setting is Done")
