#!/bin/bash

set -e

# Users cannot run this script if conda env is base
# Silently run the command to check if the conda env is base
if [[ $CONDA_DEFAULT_ENV == "base" ]]; then
    echo "Please activate a conda environment before running this script."
    echo "Run \`source /tmp/chipyard/env.sh\` to activate the chipyard conda environment."
    exit 1
fi

# Current file absolute directory path
CURR_DIR=$(cd `dirname $0` && pwd)
LOG_FILE="cocotb_test.log"

# 1. Compile the hazardflow module
cd $CURR_DIR/../../
rm -rf build/$1
cargo r --release -- --target $1 --merge --system-task
cd -

pip3 install -r $CURR_DIR/requirements.txt

# Go to the ./unit-tests/$1 directory, and run `make WAVES=1`
cd $CURR_DIR/unit_tests/$1
make clean
make WAVES=1 | tee $CURR_DIR/$LOG_FILE
echo "To see the waveform, go to the directory: $CURR_DIR/unit_tests/$1 and check \`$1_top.fst\` file with gtkwave."

FAILED_COUNT=$(grep -o "failed" $CURR_DIR/$LOG_FILE | wc -l)
if [ $FAILED_COUNT -eq 0 ]; then
    echo "Unit test succeeded"
else
    echo "Unit test failed"
    exit 1
fi
