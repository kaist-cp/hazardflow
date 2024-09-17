# Writeback stage

The writeback stage mainly do the following things:

1. Write the result back to the destination register.

It can be decomposed into combinators as follows ([code](https://github.com/kaist-cp/hazardflow/blob/main/hazardflow-designs/src/cpu/wb.rs)):

<p align="center">
  <img src="../../figure/cpu-implementation-wb.svg" width=60% />
</p>

## Input and Output

The IO interface type of the writeback stage is as follows:

### Ingress

It takes an ingress interface with type `I<VrH<MemEP, WbR>, { Dep::Demanding }>`.

You can check the explanation of `MemEP` and `WbR` in [here](mem.md#egress).

### Egress

This is the last stage, it does not return any egress interface.

## Behavior

Each combinator do the following things:

**M0** ([`map_resolver_inner`](https://kaist-cp.github.io/hazardflow/docs/hazardflow_designs/std/hazard/struct.I.html#method.map_resolver_inner)):

- Constructs the ingress resolver of the writeback stage.
  + Attaches the bypassed data and register file for resolving data hazards.

**M1** ([`reg_fwd`](https://kaist-cp.github.io/hazardflow/docs/hazardflow_designs/std/hazard/struct.I.html#method.reg_fwd)):

- Creates a pipelined stage before accessing regfile.
- Sends a ready signal which indicates it will be free in the next cycle.

**M2** ([`sink_fsm_map`](https://kaist-cp.github.io/hazardflow/docs/hazardflow_designs/std/hazard/struct.I.html#method.sink_fsm_map)):

- Updates the register file.
- Attaches the register file to the resolver for reading value of source registers.

<!--
<p align="center">
  <img src="../../figure/wb_resolver.drawio.svg" />
</p>

<p align="center">
  <img src="../../figure/wb_reg.drawio.svg" />
</p>

<p align="center">
  <img src="../../figure/reg_file.drawio.svg" />
</p>
-->
