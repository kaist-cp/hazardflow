- `build-verilator.sh`
  + Modified `j` variable from 1 to 8
  + This file should be placed in `chipyard/generators/gemmini/scripts` directory.
- `Makefile`
  + Modified `VERILATOR_THREADS` from 1 to 8. (Line 107)
  + Added `-I#/$(HOME)/chipyard/generators/gemmini/src/main/resources/vsrc \` to `VERILATOR_OPT_FLAGS`. (Line 124)
  + This fild should be placed in `chipyard/sims/verilator` directory.
