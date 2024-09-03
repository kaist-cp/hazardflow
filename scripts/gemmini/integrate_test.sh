#!/bin/bash

set -e

# Users cannot run this script if conda env is base
# Silently run the command to check if the conda env is base
if [[ $CONDA_DEFAULT_ENV == "base" ]]; then
    echo "Please activate a conda environment before running this script."
    echo "Run \`source ${CHIPYARD}/env.sh\` to activate the chipyard conda environment."
    exit 1
fi

# Current file absolute directory path
CURR_DIR=$(cd `dirname $0` && pwd)

# 1. Compile the hazardflow module
python3 $CURR_DIR/main.py compile -c $1

# 2. Build the gemmini verilator binary
python3 $CURR_DIR/main.py --debug build -c $1

# 3. Run the gemmini verilator binary
python3 $CURR_DIR/main.py --debug run -b matmul

echo "For waveform, go to the directory: ${CHIPYARD}/generators/gemmini/waveforms and check \`waveform_pruned.vcd\` file with gtkwave."
echo "To see the waveform, run \`gtkwave waveform_pruned.vcd\`"
