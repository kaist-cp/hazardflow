# CPU Core (5-Stage Pipelined)

We will use the 5-Stage pipelined Sodor CPU as an implementation example for the HazardFlow HDL.
The Sodor CPU is an educational, open-source processor developed in the [RISC-V](https://riscv.org/) project.

## Pipelined Design

Pipelined design can improve the overall processor performance with the trade-off of adding design complexity.

- The overall performance is improved by breaking down the critical path into multiple stages, while multiple instructions are processing **simultaneously** at different stages.
* The design complexity comes from the necessity of **hazard** from the later stages to the earlier stages to make sure the execution result is correct. 

### Dataflow Overview

<p align="center">
  <img src="../figure/sodor.drawio.svg" />
</p>

**Payload:**

* Each stage calculates its payload every clock cycle.
* Payloads flow horizontally from left to right through the stages.
* Payload will be passed to the next stage in the next clock cycle.
* Payload might get dropped (not passing to the next stage) because of hazards from later stages.
* Payload might get stalled (stay in the same stage) because of hazards from later stages.
* Payload might receive data from later stages' hazards and get updated before passing to the next stage.
* Payload sending to the next stage contains the necessary information for the next stage to calculate its payload, resolver, and update its state.

**Resolver:**

* Each stage calculates its resolver every clock cycle.
* Resolver flows horizontally from right to left through the stages.
* Resolver passes to the earlier stages within the same clock cycle.
* Resolvers from later stages contains the necessary information for the previous stages to construct their payloads and resolvers.

**State:**

* Each stage keeps its state in some registers (A.K.A latches).
* The state in each stage might be coming from 3 different sources:
  * The payload from the previous stage (decode, execution, memory).
  * The resolver from the earlier stage (fetch).
  * The stage maintains its state (register file in write-back stage).
* The state can be used to calculate the payload or resolver within each stage.
* State might get extracted out of the registers and not pass to the next stage as a payload (get dropped) because of hazards from later stages.

### 5-Stage Overview

* Fetch: Retrieve the next instruction to be executed from the instruction memory.
* Decode: Decode the fetched instruction.
* Execution: Perform the operation specified by the instruction.
* Memory: Access data memory or CSR if the instruction involves memory operations or CSR operation.
* Write-Back: Write the result of the execution, memory access, or CSR access back to the register file.

### Pipelining (w/o Hazard)

The ideal pipelined design works like the following figure if there is no dependency between the instructions.

<p align="center">
  <img src="../figure/ideal-5-stage.drawio.svg" />
</p>

### Pipelining (w/ Hazard)

However, most of the time, there are some dependencies between instructions.
The later stages need to send back resolvers to the earlier stages to make sure the execution result is correct.
The resolvers might cause the early stages to stall or extract the payload out of their latches and drop their payloads (kill).

#### Branch Misprediciton

The fetch stage will try to fetch the next instruction as early as possible even if the next `pc_sel` has not yet been calculated in the execution stage.
We consider it a right prediction when the calculated `pc_sel` in the execution stage is PLUS 4; otherwise, it is a misprediction.
We need to discard the mispredicted instructions in the fetch stage and decode stage if there is a misprediction.

```
I1: be x1, x2, target
I2: ADD x5, x6, x7
I3: LW x5, 8(x6)

target:
# Instructions to execute if x1 == x2
I4: SUB x5, x6, x7
...
```

<p align="center">
  <img src="../figure/branch_mis.drawio.svg" />
</p>

* At `Cycle 3`, the execution stage will pass its resolver `exe_r` to the decode stage.
* `exe_r` contains the control information to kill the mispredicted instruction in fetch stage and decode stage.
* The decode stage will extract the data from its latch and drop its payload in the next cycle when it receives the resolver from the execution stage.
* The decode stage will calculate its resolver, including the correct `pc_sel` and the kill signal, and send it to the fetch stage.
* The fetch stage will extract the data from its latch and drop its payload in the next cycle when it receives the resolver from the decode stage.
* The fetch stage will fetch the correct `pc` with the correct `pc_sel` in the next clock cycle.


#### The `FENCE.I` Instruction

RISC-V does not guarantee that stores to instruction memory `imem` will be made visible to instruction fetches until a `FENCE.I` instruction is executed (The `FENCE.I` instruction reaches the memory stage).

```
I1: SW x2, 0(x1)    # Store the value in x2 to instruction memory at address 0(x1)
I2: FENCE.I
I3: ....
```

<p align="center">
  <img src="../figure/fencei.drawio.svg" />
</p>

* `I1` is writing data to the instruction memory `imem` at the address `0(x1)`.
* `I3` is fetched from instruction memory at the address `0(x1)`.
* The fetch stage **CANNOT** sees the changes to the instruction memory **until** the `FENCE.I` instruction reaches the memory stage.
* At cycle 5, the `FENCE.I` instruction reaches the memory stage.
* At cycle 5, the fetch stage can see the newest changes in the instruction memory and fetches `I3`.
* The `pc_sel` should stay the same at cycle 3 and cycle 4.

#### Pipeline Kill

There are 2 cases we need to kill the entire pipeline (the fetch stage, the decode stage, and the execution stage) and disregard stalls.
* When the processor returns from an exception.
  * The CSR's response contains the `eret` signal.
* When the processor encounters an illegal instruction.

Dataflow when the processor encounters an illegal instruction:
* An unsupported instruction or illegal is decoded in the decode stage.
* The exception information is passed as a payload to the execution stage.
* The execution stage will pass the exception information to the memory stage.
* The memory stage will make a CSR request and write the exception information to the CSR.
* The memory stage will send out the resolver to execution stage in the same clock cycle containing:
  * The `pipeline_kill` signal to kill all the earlier stage.
  * The address of the exception handler.
* The execution stage receives the resolver from memory stage.
  * It extracts the payload from its latch and drops the payload in the next clock cycle.
  * It updates the `pc_sel` to the exception handler.
  * It sends out the resolver containing the updated `pc_sel` to decode stage in the same cycle.
* The decode stage receives the resolver from execution stage.
  * It extracts the payload from its latch and drops the payload in the next clock cycle.
  * It sends out the resolver containing the updated `pc_sel` and the kill signal `if_kill` to fetch stage in the same cycle.
* The fetch stage receives the resolver from decode stage.
  * It extracts the payload from its latch and drops the payload in the next clock cycle.
  * It will fetch the exception handler in the next clock cycle.

```
I1: some illegal instruction
I2: SW x2, 0(x1)
I3: ADD x5, x6, x7
I4: LW x5, 8(x6)
```

<p align="center">
  <img src="../figure/pipeline_kill.drawio.svg" />
</p>

#### Load-Use Stall

When read-after-write (RAW) dependency happens, we need to stall the instruction in decode stage until the instruction in execution stage reaches the memory stage.

```
I1: ADD x3, x4, x5
I2: LW x6, 8(x5)
I3: MV x1, x6
```

* `I2` is reading the memory address `8(x5)`, then it will write the result to `x6`.
* `I3` needs to read the data in `x6`.
* The `x6` can only be updated when `I2` reaches the memory stage.
* We need to stall `I3` at the decode stage until `I2` reaches the memory stage (Adding a bubble between `I2` and `I3`).
* After `I2` gets the result from memory response, `I3` can be decoded with [data bypassing](#data-bypassing-in-decode-stage).

<p align="center">
  <img src="../figure/load_use_stall.drawio.svg" />
</p>

#### Data Cache Miss

If there is a data cache miss, the processor must go to the lower memory hierarchy to search the data, which will take multiple cycles.
* The instruction in execution stage needs to be stalled, since the memory stage is taking multiple cycles to get the memory response.
* The instruction in decode stage needs to be stalled for 2 reasons:
  * It is a pipelined design and the execution stage is stalled.
  * Data could be [bypassed](#data-bypassing-in-decode-stage) from the memory stage.
* The instruction in the fetch stage will be stalled since the `false` ready signal from the memory module will go all the way down to the fetch stage's egress resolver. (Minseong please have a look about this sentence)

```
I1: LW x5, 8(x3)
I2: ADD x3, x4, x6
I3: MV x7, x3
```

<p align="center">
  <img src="../figure/dcache_miss.drawio.svg" />
</p>

#### Data Bypassing in Decode Stage

We must know certain registers' values in the decode stage then we can pass the instruction to the later stages for other processing.
The most straightforward method is to read the registers' value from the register file.
However, we do not need to always wait for the older instructions to reach the last write-back stage where writing the result to the register file happens.

* Data bypassing from execution stage:
  ```
  I1: ADD x3, x4, x5
  I2: LW x6, 0(x3)
  ```
  * When `I1` is in the execution stage, `I2` is still in decode stage.
  * The execution stage can bypass the result of `x3` to `I2` in the decode stage.

<p align="center">
  <img src="../figure/exe_bypass.drawio.svg" />
</p>

* Data bypassing from the memory stage:
  ```
  I1: LW x3, 0(x4)
  I2: LW x6, 0(x3)
  ```
  * When `I1` is in the execution stage, `I2` is still in decode stage, and [load-use stall](#load-use-stall) will happen.
  * When `I1` reaches the memory stage, `I2` is still in the decode stage.
  * When `I1` gets the value of `x3` from memory, the memory stage can bypass the value of `x3` to the `I2` in decode stage.

<p align="center">
  <img src="../figure/mem_bypass.drawio.svg" />
</p>

* Data bypassing from the write-back stage:
  ```
  I1: LW x3, 0(x4)
  I2: ADD x5, x6, x7
  I3: SUB x8, x9, x1
  I4: LW x6, 0(x3)
  ```
  * When `I1` reaches write-back stage, `I4` is in decode stage.
  * Write-back stage can bypass the value of `x3` to `I4` in decode stage.

<p align="center">
  <img src="../figure/wb_bypass.drawio.svg" />
</p>

#### Decode Stall by CSR 

The CSR could write its response to certain registers. If we need to decode the value of those registers in the decode stage, then we need to stall the instruction in the decode stage until the response coming back from the CSR in the memory stage.

```
I1: li t1, 2
I2: csrr t0, mcause
I3: bne t1, t0, 0x80000210
```

* The branch instruction `I3` needs to read the value of registers `t0` and `t1` in the decode stage.
* The value of `t0` is coming from the CSR, which is located in the memory stage. 
* The branch instruction need to be stalled when the CSR instruction is in the execute stage. 
* The value of `t0` will be bypassed from memory stage.

<p align="center">
  <img src="../figure/csr_stall.drawio.svg" />
</p>

* At cycle 4, the execution resolver stalls the payload in decode stage.
* At cycle 5, the value of `t0` will be bypassed to the decode stage from memory stage.

## Specification

In this section, we will explain the specification of each stage.

### Fetch Stage

The fetch stage must calculate the next program counter `pc` and drop the invalid `pc`.
* The egress payload of this stage is the next instruction's data and address from the instruction memory's response `mem_resp`.
* The egress resolver of this stage indicates if the current `pc` should be killed and the program counter selector `pc_sel`.
* This is the first stage, there is no ingress interface.

<p align="center">
  <img src="../figure/fetch_stage.drawio.svg" />
</p>

**Calculate the Next Program Counter**

* We can get the program counter selector `pc_sel` from the later stages as part of the egress resolver to the fetch stage.
* We use the current `pc` and `pc_sel` to calculate the next `pc`.
* The `pc_sel` specifies the next `pc` in 3 cases.
  * The current `pc` + 4.
  * A specific target.
  * Stays the same as the current `pc`.

<p align="center">
  <img src="../figure/nextpc.drawio.svg" />
</p>

**Store The Current PC and Extract The Invalid PC**

* We need to store the current `pc` for two reasons:
  * We must pass the current `pc` as a resolver to previous combinators for calculating the next `pc`.
  * We want to extract the current `pc` from the register and drop the payload if certain hazards happen.
* Whether the current `pc` should be killed will be passed to the fetch stage as an egress resolver.

<p align="center">
  <img src="../figure/store_extract_pc.drawio.svg" />
</p>

**Request Instruction Memory and Discard Invalid PC Response**

* We construct instruction memory request from the `pc`.
* The instruction memory is provided as a black box module, we can use `comb` to attach this module.
* We can assume that the `imem` can provide the response in the same cycle.
* We filter out the response if certain hazards happen and send it out as the egress payload of the fetch stage.

<p align="center">
  <img src="../figure/req_imem.drawio.svg" />
</p>

### Decode Stage

The decode stage decodes the instruction data from the fetch stage,
calculates the payload passing to the execution stage,
and also calculates the resolver to the fetch stage containing program counter selector `pc_sel`,
and information indicating if the current `pc` should be killed `if_kill`.

* The ingress payload is the instruction memory response containing the instruction's data and address `mem_resp`.
* The ingress resolver is `pc_sel` and `if_kill`.
* The egress payload contains the information of the decoded instruction `DecEP`.
* The egress resolver contains the resolvers from later stage.

<p align="center">
  <img src="../figure/decode.drawio.svg" />
</p>

**Calculate Ingress Resolver for Fetch Stage**

* Calculate the resolver from decode stage to fetch stage.
* Resolvers from later stages will be used here to calculate `pc_sel` and `if_kill`

<p align="center">
  <img src="../figure/decode_gen_resolver.drawio.svg" />
</p>

**Store The Instruction Memory Response and Decode The Instruction**

* We store the `imem` response into the latch.
* We decode the current `imem` response and calculate the instruction.

<p align="center">
  <img src="../figure/store_decode.drawio.svg" />
</p>

**Stall the Payload and Pass Back The Instruction**

* We need to stall the payload if certain certain hazards happen.
* We need to pass back the decoded instruction to previous combinators for calculating resolver.

<p align="center">
  <img src="../figure/stall_pass_back.drawio.svg" />
</p>

**Calculate the Egress Payload for Execution Stage**

* Calculate the payload for execution stage.
* Drop the payload if certain hazards happen.

<p align="center">
  <img src="../figure/decode_ep.drawio.svg" />
</p>

### Execution Stage

The execution stage executes instruction from the decode stage,
calculates the payload passing to the memory stage,
and also calculates the resolver passing to the decode stage.

* The ingress payload is decode stage's egress payload `DecEP`.
* The ingress resolver is the resolvers from execute stage and later stages `(exe_r, mem_r, wb_r)`.
* The egress payload should contain necessary information for the memory stage `exe_ep`.
* The egress resolver contains the resolver from memory stage and write-back stage `(mem_r, wb_r)`.

<p align="center">
  <img src="../figure/exe_stage.drawio.svg" />
</p>

**Calculate Ingress Resolver for Decode Stage**

* Calculate the execution stage resolver and pass it with `mem_r` and `wb_r` to the decode stage.

<p align="center">
  <img src="../figure/exe_resolver.drawio.svg" />
</p>

**Store the Decode Stage Egress Payload**

* Store the `dec_ep` into register for passing back to previous combinators for calculating `exe_r`.

<p align="center">
  <img src="../figure/exe_latch.drawio.svg" />
</p>

**Execute The Instruction**

* Execute the instruction and pass the `alu_out` to the next combinator.

<p align="center">
  <img src="../figure/exe_inst.drawio.svg" />
</p>

**Stall The Payload and Pass Back The Result of ALU**

* We need to stall the payload if certain certain hazards happen.
* We need to pass back the result of ALU `alu_out` to previous combinators for calculating resolver.

<p align="center">
  <img src="../figure/stall_exe.drawio.svg" />
</p>

**Calculate The Payload For Memory Stage**

* Calculate the payload for the memory stage.
* Drop the payload if certain hazards happen.

<p align="center">
  <img src="../figure/exe_ep.drawio.svg" />
</p>

### Memory Stage

The memory stage sends the requests to the memory `dmem` module and the CSR module according to the instruction types. 
When the memory stage gets the response from `dmem` or CSR, it will calculate the egress payload of the memory stage and send the payload to the write-back stage.
Also, the memory stage will calculate its resolver from the response from `dmem` and CSR and send it back to the execution stage.

* The ingress payload is execution stage's egress payload `ExeEP`.
* The ingress resolver is the resolvers from memory stage and the write-back stages `(mem_r, wb_r)`.
* The egress payload should contain necessary information for the write-back stage `mem_ep`.
* The egress resolver contains the resolver from the write-back stage `wb_r`.

**Store The Execution Stage Egress Payload and Filter out Unnecessary Information**

* Store the `exe_ep` into the latch to create a pipeline
* Filter out the unnecessary information and pass the resolver to execution stage.

<p align="center">
  <img src="../figure/mem_latch.drawio.svg" />
</p>

**Calculate The Resolver For Execution Stage**

* Clear up the resolver from later combinators.
* Calculate the resolver for the execution stage.

<p align="center">
  <img src="../figure/mem_resolver.drawio.svg" />
</p>

**Split The Ingress Interface For Different Memory Request**

* We split the one ingress interface into three egress interfaces for requesting different module (memory or CSR).
* We need to calculate the branch selector for splitting interface.
* We only select one of the egress interfaces to transfer the payload, also combines all the egress interfaces' resolvers into the ingress resolver. 

Calculate the branch selector:

<p align="center">
  <img src="../figure/branch_selector.drawio.svg" />
</p>

Split the ingress interface into three egress interfaces

<p align="center">
  <img src="../figure/branch.drawio.svg" />
</p>

**Calculate The Memory Request and Request The Memory Module**

* The memory module's egress resolver ready signal is `false` until the memory module gets the data from memory.
* The memory module might take multiple cycles to find the desired data if data cache miss happens.
* If data cache miss happens, certain instructions in other stages need to be stalled.

<p align="center">
  <img src="../figure/mem_module.drawio.svg" />
</p>

**Calculate The CSR Request and Request The CSR Module**

* In RISC-V, handling exceptions and interrupts involves setting up the exception vectors and managing the states when exceptions occur. Here are relevant instructions and registers:
* CSR provides the exception and interrupt handler `evec`.
* CSR provides the state of the program to continue execution after handling the exception `eret`.

<p align="center">
  <img src="../figure/csr_module.drawio.svg" />
</p>

**Pass Back The Execution Egress Payload To Earlier Combinators**

* The earlier combinator needs the egress payload `exe_ep` for calculating the memory stage's resolver.
* We pass the `exe_ep` back in each branch.

<p align="center">
  <img src="../figure/mem_pass_exe_p.drawio.svg" />
</p>

**Formatting The Ingress Interfaces For Merge**

* To merge the three branches, we need to format the ingress interfaces.
* For the fields which is not in the ingress payload, we simply set it as `None`.

<p align="center">
  <img src="../figure/mem_before_merge.drawio.svg" />
</p>

**Merge The Branches**

* This combinator will select one from the ingress interfaces to deliver the ingress payload to the egress payload and also leave the inner of the egress resolver untouched to the ingress interfaces. 

<p align="center">
  <img src="../figure/mem_merge.drawio.svg" />
</p>

**Calculate The Egress Payload of Memory Stage**

* Calculate the egress payload of memory stage and pass it to the write-back stage

<p align="center">
  <img src="../figure/mem_payload.drawio.svg" />
</p>

### Writeback Stage

This is the last stage of the pipelined design.
In this stage we need to write the data back to the register file.

* The ingress payload is the egress payload of the memory stage `mem_ep`.
* The ingress resolver is the resolver from write-back stage itself `wb_r`.
* This is the last stage of the pipelined design, there is no egress interface.

**Calculate The Write-Back Stage Resolver**

The resolver contains:
* The write-back register used in the decode stage data bypassing.
* The whole register file used in the decode stage to decode operands.
* The retire flag for writing back to the CSR module.

<p align="center">
  <img src="../figure/wb_resolver.drawio.svg" />
</p>

**Store The Execution Stage Egress Payload Pass Back For Calculating**

* Store the execution stage egress payload to create a latch.
* Pass the execution stage egress payload to previous combinator for calculating the write-back stage payload.

<p align="center">
  <img src="../figure/wb_reg.drawio.svg" />
</p>

**Register File**

* We update the register file every clock cycle.
* Send back the whole register file as a resolver for the decode stage and retire flag for CSR module.

<p align="center">
  <img src="../figure/reg_file.drawio.svg" />
</p>
