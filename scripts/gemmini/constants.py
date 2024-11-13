import pathlib
import os

# Paths
CHIPYARD_PATH = pathlib.Path(os.environ["CONDA_PREFIX"]).absolute().parent
GEMMINI_PATH = CHIPYARD_PATH / "generators" / "gemmini"
VERILATOR_MAKEFILE_PATH = CHIPYARD_PATH / "sims" / "verilator"
GEMMINI_SRC_PATH = GEMMINI_PATH / "src" / "main"
GEMMINI_VSRC_PATH = GEMMINI_SRC_PATH / "resources" / "vsrc"
GEMMINI_CHISEL_PATH = GEMMINI_SRC_PATH / "scala" / "gemmini"

HAZARDFLOW_PATH = pathlib.Path(__file__).absolute().parent.parent.parent
GEMMINI_SCRIPT_PATH = HAZARDFLOW_PATH / "scripts" / "gemmini"
CHISEL_WRAPPERS_PATH = GEMMINI_SCRIPT_PATH / "chisel_wrappers"
VERILOG_WRAPPERS_PATH = GEMMINI_SCRIPT_PATH / "verilog_wrappers"
VERILATOR_CONFIG_FILES_PATH = GEMMINI_SCRIPT_PATH / "verilator_build_files"

assert CHIPYARD_PATH is not None

# Wrapper Configurations
BUILD_CONFIGS = {
    "empty": {
        "module_names": [],
        "chisel_wrappers": [],
        "verilog_wrappers": [],
    },
    "pe": {
        "module_names": ["pe"],
        "chisel_wrappers": ["PE.scala"],
        "verilog_wrappers": ["PEBlackBox.v"],
    },
    "mesh": {
        "module_names": ["mesh_default"],
        "chisel_wrappers": ["Mesh.scala"],
        "verilog_wrappers": ["MeshBlackBox.v", "PE256Wrapper.v", "PE_256.sv", "MacUnit.sv"],
    },
    "transposer": {
        "module_names": ["transposer_default"],
        "chisel_wrappers": ["Transposer.scala"],
        "verilog_wrappers": ["TransposerBlackBox.v"],
    },
    "mesh_with_delays": {
        "module_names": ["execute_default"],
        "chisel_wrappers": ["ExecuteController.scala"],
        "verilog_wrappers": [
            "ExecuteControllerBlackBox.v",
            "AlwaysOutTransposer.sv",
            "MeshWrapper.v",
            "TransposerWrapper.v",
            "Mesh.sv",
            "Queue_98_mesh_with_delays.sv",
            "TagQueue.sv",
            "Tile.sv",
            "PE_256.sv",
            "ram_combMem_6_mesh_with_delays.sv",
            "MacUnit.sv",
            "PE.sv",
        ],
    },
    # If you want to add custom configurations, add them here
    "custom": {
        "module_names": [],
        "chisel_wrappers": [],
        "verilog_wrappers": [],
    },
}
