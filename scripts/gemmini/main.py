import shutil
import os
import subprocess
import argparse
import time

from constants import *


def setup_rust():
    subprocess.run(
        ["rustup", "component", "add", "rust-src", "rustc-dev", "llvm-tools-preview"],
        cwd=HAZARDFLOW_PATH,
        # stdout=subprocess.DEVNULL,
        # stderr=subprocess.DEVNULL,
    )
    subprocess.run(
        ["cargo", "build", "-p", "hazardflow-macro"],
        cwd=HAZARDFLOW_PATH,
        # stdout=subprocess.DEVNULL,
        # stderr=subprocess.DEVNULL,
    )


def compile_hazardflow_modules(config: str):
    # Install required rust packages and prebuild for hazardflow compiler
    setup_rust()

    # Remove `build` directory
    if os.path.isdir(HAZARDFLOW_PATH / "build"):
        shutil.rmtree(HAZARDFLOW_PATH / "build", ignore_errors=True)

    for module in BUILD_CONFIGS[config]["module_names"]:
        print(f"Compile module {module}")
        subprocess.run(
            [
                "cargo",
                "run",
                "--release",
                "--",
                "--system-task",
                "--merge",
                "--target",
                module,
            ],
            # stdout=subprocess.DEVNULL,
            # stderr=subprocess.DEVNULL,
            cwd=HAZARDFLOW_PATH,
        )


def copy_compiled_hazardflow_files(config: str):
    for module in BUILD_CONFIGS[config]["module_names"]:
        BUILD_PATH = HAZARDFLOW_PATH / "build" / module

        for filename in os.listdir(BUILD_PATH):
            if filename.endswith((".v", ".sv")):
                source_file = os.path.join(BUILD_PATH, filename)
                target_file = os.path.join(GEMMINI_VSRC_PATH, filename)
                shutil.copy(source_file, target_file)


def copy_chisel_wrappers(config: str):
    for wrapper in BUILD_CONFIGS[config]["chisel_wrappers"]:
        if not wrapper.endswith(".scala"):
            raise Exception(f"Invalid file format: {wrapper}")
        source_file = os.path.join(CHISEL_WRAPPERS_PATH, wrapper)
        target_file = os.path.join(GEMMINI_CHISEL_PATH, wrapper)
        shutil.copy(source_file, target_file)


def copy_verilog_wrappers(config: str):
    for wrapper in BUILD_CONFIGS[config]["verilog_wrappers"]:
        if not wrapper.endswith((".v", ".sv")):
            raise Exception(f"Invalid file format: {wrapper}")
        source_file = os.path.join(VERILOG_WRAPPERS_PATH, wrapper)
        target_file = os.path.join(GEMMINI_VSRC_PATH, wrapper)
        shutil.copy(source_file, target_file)


def copy_verilator_configuration_files():
    # Copy `build-verilator.sh`
    shutil.copy(
        VERILATOR_CONFIG_FILES_PATH / "build-verilator.sh",
        GEMMINI_PATH / "scripts" / "build-verilator.sh",
    )
    # Copy Makefile
    shutil.copy(
        VERILATOR_CONFIG_FILES_PATH / "Makefile", VERILATOR_MAKEFILE_PATH / "Makefile"
    )


def compile_testbenches_with_fast_option():
    subprocess.run(
        "CFLAGS=-DFAST ./build.sh",
        shell=True,
        cwd=CHIPYARD_PATH / "generators" / "gemmini" / "software" / "gemmini-rocc-tests",
    )


def reset_gemmini():
    """
    Reset the gemmini repository
    """

    subprocess.run(["git", "reset", "--hard", "v0.7.1"], cwd=GEMMINI_PATH)
    subprocess.run(["git", "clean", "-fdx"], cwd=GEMMINI_PATH)

    os.makedirs(GEMMINI_VSRC_PATH, exist_ok=False)


def setup_gemmini(config: str):
    reset_gemmini()

    for module in BUILD_CONFIGS[config]["module_names"]:
        check_hazardflow_module(module)

    copy_compiled_hazardflow_files(config)
    copy_chisel_wrappers(config)
    copy_verilog_wrappers(config)

    copy_verilator_configuration_files()


def build_verilator_simulation_binary(debug: bool):
    subprocess.run(["bash", GEMMINI_PATH / "scripts" / "setup-paths.sh"], cwd=GEMMINI_PATH)
    if debug:
        subprocess.run(
            ["bash", GEMMINI_PATH / "scripts" / "build-verilator.sh", "--debug"],
            cwd=GEMMINI_PATH,
        )
    else:
        subprocess.run(
            ["bash", GEMMINI_PATH / "scripts" / "build-verilator.sh"],
            cwd=GEMMINI_PATH,
        )


def get_args():
    """
    Get arguments from user
    """
    parser = argparse.ArgumentParser(
        description=help, formatter_class=argparse.RawDescriptionHelpFormatter
    )

    parser.add_argument(
        "--debug", action="store_true", help="Is your purpose debugging?"
    )

    subparsers = parser.add_subparsers(
        dest="cmd", required=True, help="Choose a command"
    )

    build_parser = subparsers.add_parser("build")
    build_parser.add_argument(
        "-c", "--config", required=True, help="Module to build with"
    )
    build_parser.set_defaults(action=lambda: "build")

    run_parser = subparsers.add_parser("run")
    run_parser.add_argument("-b", "--bench", required=True, help="Name of testbench")
    run_parser.set_defaults(action=lambda: "run")

    args = parser.parse_args()
    return args


def run_simulation(tb: str, debug: bool):
    compile_testbenches_with_fast_option()
    if debug:
        subprocess.run(
            ["bash", GEMMINI_PATH / "scripts" / "run-verilator.sh", tb, "--debug"],
            cwd=GEMMINI_PATH,
        )
        with open(GEMMINI_PATH / "waveforms" / "waveform_pruned.vcd", "w") as outfile:
            subprocess.run(
                [
                    "vcd-prune",
                    GEMMINI_PATH / "waveforms" / "waveform.vcd",
                    "-m",
                    "gemmini",
                ],
                stdout=outfile,
            )
        subprocess.run(
            [
                "cp",
                GEMMINI_PATH / "waveforms" / "waveform_pruned.vcd",
                GEMMINI_SCRIPT_PATH / f"{tb}.vcd",
            ]
        )
    else:
        subprocess.run(
            ["bash", GEMMINI_PATH / "scripts" / "run-verilator.sh", tb], cwd=GEMMINI_PATH
        )


def check_hazardflow_module(module: str):
    """
    Check if the module is compiled or not
    """
    if not os.path.exists(HAZARDFLOW_PATH / "build" / module):
        print(f"HazardFlow module {module} is not compiled")
        exit(1)


if __name__ == "__main__":
    """
    Integration Test
    1. Build Integration test verilator binary
        - python3 main.py build -c pe
        - python3 main.py --debug build -c pe
    2. Run Integration test
        - python3 main.py run -b matmul
        - python3 main.py --debug run -b matmul,
    """
    args = get_args()
    if args.cmd == "build":
        compile_hazardflow_modules(args.config)
        # Check if we compiled the hazardflow module
        for module in BUILD_CONFIGS[args.config]["module_names"]:
            check_hazardflow_module(module)

        setup_gemmini(args.config)
        build_verilator_simulation_binary(args.debug)

    elif args.cmd == "run":
        run_simulation(args.bench, args.debug)
