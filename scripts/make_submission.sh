#!/usr/bin/env bash

# Clears the previous submissions.
rm -rf hw2.zip

# Creates new submissions.
zip hw2.zip -j hazardflow-designs/src/cpu/fetch.rs hazardflow-designs/src/cpu/decode.rs hazardflow-designs/src/cpu/exe.rs hazardflow-designs/src/cpu/branch_predictor/bht.rs hazardflow-designs/src/cpu/branch_predictor/btb.rs
