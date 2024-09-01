#!/bin/bash

trap 'echo "Interrupt received, exiting..."; exit 1;' SIGINT

set +e

# Current file absolute directory path
CURR_DIR=$(cd `dirname $0` && pwd)
LOG_DIR="$CURR_DIR/output"
BENCH_DIR="$CURR_DIR/program/bench"
EMULATOR="$CURR_DIR/emulator-debug"

if [ ! -f $EMULATOR ]; then
    echo "$EMULATOR does not exist."
    echo "Please run \`python3 scripts/cpu/main.py build\` first."
    exit 1
fi

TRACE_FLAG=0
CPI_FLAG=0
WAVES_FLAG=0

for ARG in "$@"; do
    if [ "$ARG" == "trace" ]; then
        TRACE_FLAG=1
    elif [ "$ARG" == "cpi" ]; then
        CPI_FLAG=1
    elif [ "$ARG" == "--waves" ]; then
        WAVES_FLAG=1
    fi
done

mkdir -p $LOG_DIR

if [ $TRACE_FLAG -eq 1 ]; then
    echo "Running benchmark trace tests"

    for TB in "$BENCH_DIR"/*; do
        TB_FILENAME=$(basename $TB)
        if [ $WAVES_FLAG -eq 1 ]; then
            VCD_OPTION="-v$LOG_DIR/$TB_FILENAME.vcd"
        else
            VCD_OPTION=""
        fi
        TXT_FILE="$LOG_DIR/$TB_FILENAME.txt"

        if [ -f $TB ] && [[ $TB != *.dump ]] && [[ $TB != *.trace ]]; then
            ((COUNT++))

            echo -n "Running benchmark ($COUNT/9): $TB_FILENAME .. "
            $EMULATOR $VCD_OPTION +max-cycles=100000 $TB > $TXT_FILE 2>&1
            echo "DONE"
        fi
    done

    python3 $CURR_DIR/trace.py
elif [ $CPI_FLAG -eq 1 ]; then
    echo "Running benchmark cpi tests"

    for TB in "$BENCH_DIR"/*; do
        TB_FILENAME=$(basename $TB)
        if [ $WAVES_FLAG -eq 1 ]; then
            VCD_OPTION="-v$LOG_DIR/$TB_FILENAME.vcd"
        else
            VCD_OPTION=""
        fi
        TXT_FILE="$LOG_DIR/$TB_FILENAME.txt"

        if [ -f $TB ] && [[ $TB != *.dump ]] && [[ $TB != *.trace ]]; then
            ((COUNT++))

            echo -n "Running benchmark ($COUNT/9): $TB_FILENAME .. "
            $EMULATOR $VCD_OPTION +max-cycles=100000 $TB > $TXT_FILE 2>&1
            echo "DONE"
        fi
    done

    python3 $CURR_DIR/cpi.py branch_prediction
fi
