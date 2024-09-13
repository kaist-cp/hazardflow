export DESIGN_NAME = Core
export PLATFORM    = nangate45
export CURR        = $(dir $(DESIGN_CONFIG))

export VERILOG_FILES = $(CURR)../vsrc/Core.v \
                       $(CURR)../vsrc/CoreWrapper.v \
                       $(CURR)../vsrc/core_top.v


export SDC_FILE      = $(CURR)/constraint.sdc

export CORE_UTILIZATION ?= 50
export PLACE_DENSITY_LB_ADDON = 0.20
export TNS_END_PERCENT        = 100
