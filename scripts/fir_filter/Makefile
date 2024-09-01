# Makefile

# defaults
SIM ?= icarus
TOPLEVEL_LANG ?= verilog

VERILOG_SOURCES += $(PWD)/fir.v

# TOPLEVEL is the name of the toplevel module in your Verilog or VHDL file
TOPLEVEL = fir

# MODULE is the basename of the Python test file
MODULE = test_fir

# include cocotb's make rules to take care of the simulator setup
include $(shell cocotb-config --makefiles)/Makefile.sim

clean::
	@rm -rf iverilog_dump.v
	@rm -rf $(TOPLEVEL).vcd
