# Makefile

TOPLEVEL_LANG = verilog
SIM = icarus

VERILOG_SOURCES += $(PWD)/fir.v
VERILOG_SOURCES += $(PWD)/dump.v

COMPILE_ARGS += -s dump

# TOPLEVEL is the name of the toplevel module in your Verilog or VHDL file
TOPLEVEL = fir

# MODULE is the basename of the Python test file
MODULE = test_fir

# include cocotb's make rules to take care of the simulator setup
include $(shell cocotb-config --makefiles)/Makefile.sim

clean::
	@rm -rf results.xml
	@rm -rf $(TOPLEVEL).vcd
