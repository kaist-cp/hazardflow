#!/bin/bash

trap 'echo "Interrupt received, exiting..."; exit 1;' SIGINT

set +e

# Current file absolute directory path
CURR_DIR=$(cd `dirname $0` && pwd)
LOG_DIR="$CURR_DIR/output"
EMULATOR="$CURR_DIR/emulator-debug"

if [ ! -f $EMULATOR ]; then
    echo "$EMULATOR does not exist."
    echo "Please run \`python3 scripts/cpu/main.py build\` first."
    exit 1
fi

BASE_FLAG=0
BASE_DIR="$CURR_DIR/program/isa/base"
MEXT_FLAG=0
MEXT_DIR="$CURR_DIR/program/isa/mext"
WAVES_FLAG=0

for ARG in "$@"; do
    if [ "$ARG" == "base" ]; then
        BASE_FLAG=1
    elif [ "$ARG" == "mext" ]; then
        MEXT_FLAG=1
    elif [ "$ARG" == "--waves" ]; then
        WAVES_FLAG=1
    fi
done

mkdir -p $LOG_DIR

COUNT=0
COUNT_PASSED=0
COUNT_FAILED=0

if [ $BASE_FLAG -eq 1 ]; then
    echo "Running base ISA tests"

    for TB in "$BASE_DIR"/*; do
        TB_FILENAME=$(basename $TB)
        if [ $WAVES_FLAG -eq 1 ]; then
            VCD_OPTION="-v$LOG_DIR/$TB_FILENAME.vcd"
        else
            VCD_OPTION=""
        fi
        TXT_FILE="$LOG_DIR/$TB_FILENAME.txt"

        if [ -f $TB ] && [[ $TB != *.dump ]]; then
            ((COUNT++))

            echo -n "Test ($COUNT/43): $TB_FILENAME .. "
            $EMULATOR $VCD_OPTION $TB > $TXT_FILE 2>&1

            if [ $? -eq 0 ]; then
                echo "PASSED"
                ((COUNT_PASSED++))
            else
                echo "FAILED"
                ((COUNT_FAILED++))
            fi
        fi
    done
    echo "Number of success tests: $COUNT_PASSED / 43"

    if [ $COUNT_FAILED -ne 0 ]; then
        echo "You can check the log file for failed test cases in \`output\` folder."
        exit 1
    fi
elif [ $MEXT_FLAG -eq 1 ]; then
    echo "Running M-extension ISA tests"

    for TB in "$MEXT_DIR"/*; do
        TB_FILENAME=$(basename $TB)
        if [ $WAVES_FLAG -eq 1 ]; then
            VCD_OPTION="-v$LOG_DIR/$TB_FILENAME.vcd"
        else
            VCD_OPTION=""
        fi
        TXT_FILE="$LOG_DIR/$TB_FILENAME.txt"

        if [ -f $TB ] && [[ $TB != *.dump ]]; then
            ((COUNT++))

            echo -n "Test ($COUNT/8): $TB_FILENAME .. "
            $EMULATOR $VCD_OPTION $TB > $TXT_FILE 2>&1

            if [ $? -eq 0 ]; then
                echo "PASSED"
                ((COUNT_PASSED++))
            else
                echo "FAILED"
                ((COUNT_FAILED++))
            fi
        fi
    done
    echo "Number of success tests: $COUNT_PASSED / 8"

    if [ $COUNT_FAILED -ne 0 ]; then
        echo "You can check the log file for failed test cases in \`output\` folder."
        exit 1
    fi
fi
