//! Reservation station.

use super::*;
use crate::gemmini::isa::*;
use crate::gemmini::local_addr::*;
use crate::hpanic;

const BLOCK_ROWS: usize = TILE_ROWS * MESH_ROWS;
const BLOCK_COLS: usize = TILE_COLS * MESH_COLS;

const CL_BLOCK_COLS: usize = clog2(BLOCK_COLS);

// Every cycle, at most two instructions of a single "type" (ld/st/ex) can be completed: one through the `completed` port, and the other if it is a "complete-on-issue" instruction.
const MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE: usize = 2;

/// Reservation station issue type.
#[derive(Debug, Clone, Copy)]
pub struct RsIssue {
    /// Command.
    pub cmd: GemminiCmd,
    /// Rob ID.
    pub rob_id: U<{ clog2(RS_ENTRIES) }>,
}

/// Interfaces issued from reservation stations.
#[derive(Debug, Interface)]
pub struct RsIssues {
    /// Issue from LDQ.
    pub ld: Vr<RsIssue>,
    /// Issue from EXQ.
    pub ex: Vr<RsIssue>,
    /// Issue from STQ.
    pub st: Vr<RsIssue>,
}

/// Number of completed instructions. It will be send to `LoopConv` and `LoopMatmul` modules.
#[derive(Debug, Interface)]
pub struct RsCompleted {
    /// Number of completed LD instructions to be sent to `LoopConv`.
    pub conv_ld: Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
    /// Number of completed EX instructions to be sent to `LoopConv`.
    pub conv_ex: Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
    /// Number of completed ST instructions to be sent to `LoopConv`.
    pub conv_st: Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,

    /// Number of completed LD instructions to be sent to `LoopMatmul`.
    pub matmul_ld: Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
    /// Number of completed EX instructions to be sent to `LoopMatmul`.
    pub matmul_ex: Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
    /// Number of completed ST instructions to be sent to `LoopMatmul`.
    pub matmul_st: Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
}

/// Queue type.
#[derive(Debug, Clone, Copy)]
pub enum Q {
    /// Load queue.
    Ld,
    /// Execute queue.
    Ex,
    /// Store queue.
    St,
}

/// Represents the SRAM addresses accessed by the operation.
#[derive(Debug, Clone, Copy)]
struct Op {
    /// Start address.
    start: LocalAddr,

    /// End address.
    end: LocalAddr,

    /// Indicates whether an address overflow occurs from `start` to `end`.
    ///
    /// - If this is `true`, the accessed SRAM addresses are `[start, MAX_ADDR)` and `[0, end)`.
    /// - If this is `false`, the accessed SRAM addresses are `[start, end)`.
    wraps_around: bool,
}

impl Op {
    /// Checks whether an SRAM address overlap occurred.
    fn overlaps(self, other: Op) -> bool {
        // TODO: `is_garbage` check might not really be necessary.
        ((other.start.le(self.start) && (self.start.lt(other.end) || other.wraps_around))
            || (self.start.le(other.start) && (other.start.lt(self.end) || self.wraps_around)))
            && !(self.start.is_garbage() || other.start.is_garbage())
    }
}

/// Represents dependencies between entries in the queue.
#[derive(Debug, Default, Clone, Copy)]
pub struct Deps {
    /// Dependencies between entries in LD queue.
    pub ld: Array<bool, RS_ENTRIES_LD>,
    /// Dependencies between entries in EX queue.
    pub ex: Array<bool, RS_ENTRIES_EX>,
    /// Dependencies between entries in ST queue.
    pub st: Array<bool, RS_ENTRIES_ST>,
}

impl Deps {
    /// Represents the dependencies are resolved or not.
    pub fn resolved(self) -> bool {
        self.ld == 0.into_u() && self.ex == 0.into_u() && self.st == 0.into_u()
    }

    /// Sets `idx`-th element of LD dependency to `elt`.
    pub fn set_ld(self, idx: U<{ clog2(RS_ENTRIES_LD) }>, elt: bool) -> Self {
        Self { ld: self.ld.set(idx, elt), ..self }
    }

    /// Sets `idx`-th element of EX dependency to `elt`.
    pub fn set_ex(self, idx: U<{ clog2(RS_ENTRIES_EX) }>, elt: bool) -> Self {
        Self { ex: self.ex.set(idx, elt), ..self }
    }

    /// Sets `idx`-th element of ST dependency to `elt`.
    pub fn set_st(self, idx: U<{ clog2(RS_ENTRIES_ST) }>, elt: bool) -> Self {
        Self { st: self.st.set(idx, elt), ..self }
    }
}

/// Entry in the queue.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    /// Queue type.
    pub q: Q,

    /// Indicates whether the command modifies the config.
    is_config: bool,

    /// SRAM addresses accessed by operand A.
    opa: HOption<Op>,
    /// Indicates whether operand A is the destination operand.
    opa_is_dst: bool,
    /// SRAM addresses accessed by operand B.
    opb: HOption<Op>,

    /// Entry is issued or not.
    pub issued: bool,

    /// Indicates that this command is completed immediately when issued.
    complete_on_issue: bool,

    /// Command.
    cmd: GemminiCmd,

    /// Dependencies between entries in the queue.
    pub deps: Deps,
}

/// Returns the decoded command.
///
/// It returns the entry and whether it is norm or not.
fn decode_cmd(cmd: GemminiCmd, config: Config) -> (Entry, bool) {
    let funct = cmd.cmd.inst.funct;
    let funct_is_compute = matches!(funct, Funct::ComputeAndStayCmd | Funct::ComputeAndFlipCmd);
    let config_cmd_type = ConfigCmd::from(cmd.cmd.rs1.clip_const::<2>(0));

    let q = if matches!(funct, Funct::LoadCmd | Funct::Load2Cmd | Funct::Load3Cmd)
        || matches!((funct, config_cmd_type), (Funct::ConfigCmd, ConfigCmd::Load))
    {
        Q::Ld
    } else if matches!(funct, Funct::PreloadCmd)
        || funct_is_compute
        || matches!((funct, config_cmd_type), (Funct::ConfigCmd, ConfigCmd::Ex))
    {
        Q::Ex
    } else if matches!(funct, Funct::StoreCmd)
        || (matches!(funct, Funct::ConfigCmd) && matches!(config_cmd_type, ConfigCmd::Store | ConfigCmd::Norm))
    {
        Q::St
    } else {
        hpanic!("This funct should not come here")
    };

    // Normalization commands are a subset of store commands, so they still go in the ST queue.
    let is_norm = matches!((funct, config_cmd_type), (Funct::ConfigCmd, ConfigCmd::Norm));
    let is_config = matches!(funct, Funct::ConfigCmd);

    let op1_start = LocalAddr::from(cmd.cmd.rs1);
    let op1 = if matches!(funct, Funct::PreloadCmd) {
        // TODO: check `b_transpose` here if WS mode is enabled.
        let preload_rows = cmd.cmd.rs1.clip_const::<{ clog2(BLOCK_ROWS + 1) }>(48);
        let (end, wraps_around) = op1_start.add_with_overflow(preload_rows.resize());

        Some(Op { start: op1_start, end, wraps_around })
    } else if funct_is_compute {
        let rows = cmd.cmd.rs1.clip_const::<{ clog2(BLOCK_ROWS + 1) }>(48);
        let cols = cmd.cmd.rs1.clip_const::<{ clog2(BLOCK_COLS + 1) }>(32);
        let compute_rows = if config.a_transpose { cols } else { rows } * config.a_stride;
        let (end, wraps_around) = op1_start.add_with_overflow(compute_rows.resize());

        Some(Op { start: op1_start, end, wraps_around })
    } else {
        None
    };

    let op2_start = LocalAddr::from(cmd.cmd.rs2);
    let op2 = if funct_is_compute {
        let compute_rows = cmd.cmd.rs2.clip_const::<{ clog2(BLOCK_ROWS + 1) }>(48);
        let (end, wraps_around) = op2_start.add_with_overflow(compute_rows.resize());

        Some(Op { start: op2_start, end, wraps_around })
    } else if config.pooling_is_enabled && (funct_is_compute || matches!(funct, Funct::StoreCmd)) {
        // If pooling is enabled, then we assume that this command simply mvouts everything in this accumulator bank from
        // start to the end of the bank. TODO: This won't work when `ACC_BANKS != 2`.
        let acc_bank = op2_start.acc_bank();
        let next_bank_addr = LocalAddr {
            is_acc_addr: true,
            data: (acc_bank + 1.into_u()).resize() << ACC_BANK_ROW_BITS,
            ..LocalAddr::from(0.into_u())
        };

        Some(Op { start: op2_start, end: next_bank_addr, wraps_around: next_bank_addr.acc_bank() == 0.into_u() })
    } else if matches!(funct, Funct::StoreCmd) {
        let mvout_cols = cmd.cmd.rs2.clip_const::<{ clog2(MVOUT_COLS_BITS) }>(32);
        let mvout_rows = cmd.cmd.rs2.clip_const::<{ clog2(MVOUT_ROWS_BITS) }>(48);

        let mvout_mats = (mvout_cols >> CL_BLOCK_COLS)
            + if (mvout_cols & BLOCK_COLS.into_u()) != 0.into_u() { 1.into_u() } else { 0.into_u() };
        let total_mvout_rows = ((mvout_mats - 1.into_u()) * BLOCK_COLS.into_u::<5>()) + mvout_rows.resize();
        let (end, wraps_around) = op2_start.add_with_overflow(total_mvout_rows.resize());

        Some(Op { start: op2_start, end, wraps_around: config.pooling_is_enabled || wraps_around })
    } else {
        None
    };

    let dst_start = LocalAddr::from(cmd.cmd.rs2.clip_const::<32>(0).resize());
    let dst = if matches!(funct, Funct::PreloadCmd) {
        let preload_rows = cmd.cmd.rs2.clip_const::<{ clog2(BLOCK_ROWS + 1) }>(48) * config.c_stride;
        let (end, wraps_around) = dst_start.add_with_overflow(preload_rows.resize());

        Some(Op { start: dst_start, end, wraps_around })
    } else if matches!(funct, Funct::LoadCmd | Funct::Load2Cmd | Funct::Load3Cmd) {
        let id: U<{ clog2(LOAD_STATES) }> = if matches!(funct, Funct::Load2Cmd) {
            1.into_u()
        } else if matches!(funct, Funct::Load3Cmd) {
            2.into_u()
        } else {
            0.into_u()
        };
        let block_stride = config.ld_block_strides[id];
        let pixel_repeats = config.ld_pixel_repeats[id];

        let mvin_cols = cmd.cmd.rs2.clip_const::<MVIN_COLS_BITS>(32);
        let mvin_rows = cmd.cmd.rs2.clip_const::<MVIN_ROWS_BITS>(48);

        let mvin_mats =
            (mvin_cols >> CL_BLOCK_COLS) + (mvin_cols.clip_const::<CL_BLOCK_COLS>(0) != 0.into_u()).into_u();
        let total_mvin_rows = (mvin_mats - 1.into_u()) * block_stride + mvin_rows.resize();

        let start = if dst_start.is_acc_addr {
            dst_start
        } else if dst_start.full_sp_addr() > (SP_ROWS / 2).into_u() {
            dst_start.floor_sub(pixel_repeats.resize(), (SP_ROWS / 2).into_u()).0
        } else {
            dst_start.floor_sub(pixel_repeats.resize(), 0.into_u()).0
        };

        let (end, wraps_around) = start.add_with_overflow(total_mvin_rows.resize());

        Some(Op { start, end, wraps_around })
    } else {
        None
    };

    let new_entry = Entry {
        q,
        is_config,
        opa: if dst.is_some() { dst } else { op1.or(op2) },
        opa_is_dst: dst.is_some(),
        opb: if dst.is_some() { op1.or(op2) } else { op2 },
        issued: false,
        complete_on_issue: is_config && !matches!(q, Q::Ex),
        cmd,
        deps: Deps::default(),
    };

    (new_entry, is_norm)
}

/// Updates the config.
fn update_config(new_entry: Entry, is_norm: bool, config: Config) -> Config {
    // If command is not config, return early.
    if !new_entry.is_config {
        return config;
    }

    match new_entry.q {
        Q::Ex => {
            let set_only_strides = new_entry.cmd.cmd.rs1[7];

            Config {
                a_stride: new_entry.cmd.cmd.rs1.clip_const::<16>(16).resize(),
                c_stride: new_entry.cmd.cmd.rs2.clip_const::<16>(48).resize(),
                a_transpose: if !set_only_strides { new_entry.cmd.cmd.rs1[8] } else { config.a_transpose },
                ..config
            }
        }
        Q::Ld => {
            let id = new_entry.cmd.cmd.rs1.clip_const::<2>(3);
            let block_stride = new_entry.cmd.cmd.rs1.clip_const::<16>(16);
            let repeat_pixels = new_entry.cmd.cmd.rs1.clip_const::<PIXEL_REPEATS_BITS>(8);
            let repeat_pixels = if repeat_pixels < 1.into_u() { 1.into_u() } else { repeat_pixels };

            Config {
                ld_block_strides: config.ld_block_strides.set(id, block_stride.resize()),
                ld_pixel_repeats: config.ld_pixel_repeats.set(id, repeat_pixels - 1.into_u()),
                ..config
            }
        }
        Q::St => {
            if is_norm {
                config
            } else {
                let pool_stride = new_entry.cmd.cmd.rs1.clip_const::<2>(4);
                Config { pooling_is_enabled: pool_stride != 0.into_u(), ..config }
            }
        }
    }
}

/// Represents the reservation station entries.
#[derive(Debug, Default, Clone, Copy)]
struct Entries {
    /// LD entries.
    entries_ld: Array<HOption<Entry>, RS_ENTRIES_LD>,
    /// EX entries.
    entries_ex: Array<HOption<Entry>, RS_ENTRIES_EX>,
    /// ST entries.
    entries_st: Array<HOption<Entry>, RS_ENTRIES_ST>,
}

impl Entries {
    /// Computes dependencies between entries in the queue.
    fn get_deps(self, entry: Entry) -> Deps {
        let not_config = !entry.is_config;
        let entry_opa = entry.opa.unwrap();
        let entry_opb = entry.opb.unwrap();

        let ld = self.entries_ld.map(|e| {
            e.is_some_and(|e| match entry.q {
                Q::Ld => !e.issued,
                Q::Ex => not_config && e.opa.is_some_and(|opa| entry_opa.overlaps(opa) || entry_opb.overlaps(opa)),
                Q::St => not_config && e.opa.is_some_and(|opa| entry_opa.overlaps(opa)),
            })
        });

        let ex = self.entries_ex.map(|e| {
            e.is_some_and(|e| match entry.q {
                Q::Ld => {
                    not_config
                        && (e.opa.is_some_and(|opa| entry_opa.overlaps(opa))
                            || e.opb.is_some_and(|opb| entry_opa.overlaps(opb)))
                }
                Q::Ex => !e.issued,
                Q::St => not_config && e.opa_is_dst && e.opa.is_some_and(|opa| entry_opa.overlaps(opa)),
            })
        });

        let st = self.entries_st.map(|e| {
            e.is_some_and(|e| match entry.q {
                Q::Ld => not_config && e.opa.is_some_and(|opa| entry_opa.overlaps(opa)),
                Q::Ex => not_config && entry.opa_is_dst && e.opa.is_some_and(|opa| entry_opa.overlaps(opa)),
                Q::St => !e.issued,
            })
        });

        Deps { ld, ex, st }
    }

    /// Tries to allocates a new entry.
    ///
    /// It returns whether the allocation succeeded or not, and updates the entries.
    fn try_alloc(self, entry: Entry) -> (bool, Self) {
        let alloc_id_ld = self.entries_ld.find_idx(|e| e.is_none());
        let alloc_id_ex = self.entries_ex.find_idx(|e| e.is_none());
        let alloc_id_st = self.entries_st.find_idx(|e| e.is_none());

        let is_allocated_ld = matches!(entry.q, Q::Ld) && alloc_id_ld.is_some();
        let is_allocated_ex = matches!(entry.q, Q::Ex) && alloc_id_ex.is_some();
        let is_allocated_st = matches!(entry.q, Q::St) && alloc_id_st.is_some();

        let is_allocated = is_allocated_ld || is_allocated_ex || is_allocated_st;

        let entries_ld_next = self.entries_ld.set_cond(is_allocated_ld, alloc_id_ld.unwrap(), Some(entry));
        let entries_ex_next = self.entries_ex.set_cond(is_allocated_ex, alloc_id_ex.unwrap(), Some(entry));
        let entries_st_next = self.entries_st.set_cond(is_allocated_st, alloc_id_st.unwrap(), Some(entry));

        let entries_next =
            Entries { entries_ld: entries_ld_next, entries_ex: entries_ex_next, entries_st: entries_st_next };

        (is_allocated, entries_next)
    }

    /// Tries to issue entry.
    ///
    /// It returns the command which has `q` type and ready to be issued, and updates the entries.
    fn try_issue(self, q: Q) -> (HOption<(RsIssue, bool)>, Self) {
        let issued_id = match q {
            Q::Ld => self
                .entries_ld
                .find_idx(|e| e.is_some_and(|e| e.deps.resolved() && !e.issued))
                .map(|id| id.resize::<{ clog2(RS_MAX_PER_TYPE) }>()),
            Q::Ex => self
                .entries_ex
                .find_idx(|e| e.is_some_and(|e| e.deps.resolved() && !e.issued))
                .map(|id| id.resize::<{ clog2(RS_MAX_PER_TYPE) }>()),
            Q::St => self
                .entries_st
                .find_idx(|e| e.is_some_and(|e| e.deps.resolved() && !e.issued))
                .map(|id| id.resize::<{ clog2(RS_MAX_PER_TYPE) }>()),
        };

        let issued_entry = issued_id.map(|id| {
            match q {
                Q::Ld => self.entries_ld[id],
                Q::Ex => self.entries_ex[id],
                Q::St => self.entries_st[id],
            }
            .unwrap() // `unwrap()` always success because of the logic of finding `issue_id`.
        });

        let issued = issued_id.zip(issued_entry).map(|(id, entry)| {
            let global_issue_id = id.append((q as usize).into_u::<2>());
            (RsIssue { cmd: entry.cmd, rob_id: global_issue_id }, entry.complete_on_issue)
        });

        let entries_next = if let Some((id, entry)) = issued_id.zip(issued_entry) {
            let entries_ld_next = self.entries_ld.set_cond(
                matches!(q, Q::Ld),
                id.resize::<{ clog2(RS_ENTRIES_LD) }>(),
                if entry.complete_on_issue { None } else { Some(Entry { issued: true, ..entry }) },
            );

            let entries_ex_next = self.entries_ex.set_cond(
                matches!(q, Q::Ex),
                id.resize::<{ clog2(RS_ENTRIES_EX) }>(),
                if entry.complete_on_issue { None } else { Some(Entry { issued: true, ..entry }) },
            );

            let entries_st_next = self.entries_st.set_cond(
                matches!(q, Q::St),
                id.resize::<{ clog2(RS_ENTRIES_ST) }>(),
                if entry.complete_on_issue { None } else { Some(Entry { issued: true, ..entry }) },
            );

            let entries_ld_next = entries_ld_next.map(|e| {
                e.map(|e| match q {
                    Q::Ld => Entry { deps: e.deps.set_ld(id.resize::<{ clog2(RS_ENTRIES_LD) }>(), false), ..e },
                    Q::Ex => {
                        if entry.complete_on_issue {
                            Entry { deps: e.deps.set_ex(id.resize::<{ clog2(RS_ENTRIES_EX) }>(), false), ..e }
                        } else {
                            e
                        }
                    }
                    Q::St => {
                        if entry.complete_on_issue {
                            Entry { deps: e.deps.set_st(id.resize::<{ clog2(RS_ENTRIES_ST) }>(), false), ..e }
                        } else {
                            e
                        }
                    }
                })
            });

            let entries_ex_next = entries_ex_next.map(|e| {
                e.map(|e| match q {
                    Q::Ld => {
                        if entry.complete_on_issue {
                            Entry { deps: e.deps.set_ld(id.resize::<{ clog2(RS_ENTRIES_LD) }>(), false), ..e }
                        } else {
                            e
                        }
                    }
                    Q::Ex => Entry { deps: e.deps.set_ex(id.resize::<{ clog2(RS_ENTRIES_EX) }>(), false), ..e },
                    Q::St => {
                        if entry.complete_on_issue {
                            Entry { deps: e.deps.set_st(id.resize::<{ clog2(RS_ENTRIES_ST) }>(), false), ..e }
                        } else {
                            e
                        }
                    }
                })
            });

            let entries_st_next = entries_st_next.map(|e| {
                e.map(|e| match q {
                    Q::Ld => {
                        if entry.complete_on_issue {
                            Entry { deps: e.deps.set_ld(id.resize::<{ clog2(RS_ENTRIES_LD) }>(), false), ..e }
                        } else {
                            e
                        }
                    }
                    Q::Ex => {
                        if entry.complete_on_issue {
                            Entry { deps: e.deps.set_ex(id.resize::<{ clog2(RS_ENTRIES_EX) }>(), false), ..e }
                        } else {
                            e
                        }
                    }
                    Q::St => Entry { deps: e.deps.set_st(id.resize::<{ clog2(RS_ENTRIES_ST) }>(), false), ..e },
                })
            });

            Entries { entries_ld: entries_ld_next, entries_ex: entries_ex_next, entries_st: entries_st_next }
        } else {
            self
        };

        (issued, entries_next)
    }

    /// Returns completed entry and updates the entries.
    fn compute_completed(self, q: Q, id: U<{ clog2(RS_MAX_PER_TYPE) }>) -> (Entry, Self) {
        let completed_entry = match q {
            Q::Ld => self.entries_ld[id.resize::<{ clog2(RS_ENTRIES_LD) }>()],
            Q::Ex => self.entries_ex[id.resize::<{ clog2(RS_ENTRIES_EX) }>()],
            Q::St => self.entries_st[id.resize::<{ clog2(RS_ENTRIES_ST) }>()],
        }
        .unwrap(); // TODO: Why this `unwrap()` always success?

        let entries_ld_next =
            self.entries_ld.set_cond(matches!(q, Q::Ld), id.resize::<{ clog2(RS_ENTRIES_LD) }>(), None).map(|e| {
                e.map(|e| Entry {
                    deps: match q {
                        Q::Ld => e.deps.set_ld(id.resize::<{ clog2(RS_ENTRIES_LD) }>(), false),
                        Q::Ex => e.deps.set_ex(id.resize::<{ clog2(RS_ENTRIES_EX) }>(), false),
                        Q::St => e.deps.set_st(id.resize::<{ clog2(RS_ENTRIES_ST) }>(), false),
                    },
                    ..e
                })
            });

        let entries_ex_next =
            self.entries_ex.set_cond(matches!(q, Q::Ex), id.resize::<{ clog2(RS_ENTRIES_EX) }>(), None).map(|e| {
                e.map(|e| Entry {
                    deps: match q {
                        Q::Ld => e.deps.set_ld(id.resize::<{ clog2(RS_ENTRIES_LD) }>(), false),
                        Q::Ex => e.deps.set_ex(id.resize::<{ clog2(RS_ENTRIES_EX) }>(), false),
                        Q::St => e.deps.set_st(id.resize::<{ clog2(RS_ENTRIES_ST) }>(), false),
                    },
                    ..e
                })
            });

        let entries_st_next =
            self.entries_st.set_cond(matches!(q, Q::St), id.resize::<{ clog2(RS_ENTRIES_ST) }>(), None).map(|e| {
                e.map(|e| Entry {
                    deps: match q {
                        Q::Ld => e.deps.set_ld(id.resize::<{ clog2(RS_ENTRIES_LD) }>(), false),
                        Q::Ex => e.deps.set_ex(id.resize::<{ clog2(RS_ENTRIES_EX) }>(), false),
                        Q::St => e.deps.set_st(id.resize::<{ clog2(RS_ENTRIES_ST) }>(), false),
                    },
                    ..e
                })
            });

        let entries_next =
            Entries { entries_ld: entries_ld_next, entries_ex: entries_ex_next, entries_st: entries_st_next };

        (completed_entry, entries_next)
    }
}

/// Config values set by programmer.
#[derive(Debug, Default, Clone, Copy)]
struct Config {
    a_stride: U<A_STRIDE_BITS>,
    c_stride: U<C_STRIDE_BITS>,
    a_transpose: bool,
    ld_block_strides: Array<U<BLOCK_STRIDE_BITS>, LOAD_STATES>,
    pooling_is_enabled: bool,
    ld_pixel_repeats: Array<U<PIXEL_REPEATS_BITS>, LOAD_STATES>,
}

/// Internal queues logic.
///
/// It returns (1) issued command, (2) completed id to conv, and (3) completed id to matmul for each queue type.
#[allow(clippy::type_complexity)]
fn queues(
    alloc: Vr<Entry>,
    completed_ld: Valid<U<{ clog2(RS_MAX_PER_TYPE) }>>,
    completed_ex: Valid<U<{ clog2(RS_MAX_PER_TYPE) }>>,
    completed_st: Valid<U<{ clog2(RS_MAX_PER_TYPE) }>>,
) -> (
    (
        Vr<RsIssue>,
        Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
        Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
    ),
    (
        Vr<RsIssue>,
        Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
        Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
    ),
    (
        Vr<RsIssue>,
        Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
        Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
    ),
    Valid<bool>,
) {
    unsafe {
        (alloc, completed_ld, completed_ex, completed_st).fsm::<(
            (
                Vr<RsIssue>,
                Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
                Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
            ),
            (
                Vr<RsIssue>,
                Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
                Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
            ),
            (
                Vr<RsIssue>,
                Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
                Valid<U<{ clog2(MAX_INSTRUCTIONS_COMPLETED_PER_TYPE_PER_CYCLE + 1) }>>,
            ),
            Valid<bool>,
        ), Entries>(
            Entries::default(),
            |(alloc, completed_ld, completed_ex, completed_st),
             ((er_ld, ..), (er_ex, ..), (er_st, ..), ()),
             entries| {
                let new_entry = alloc.map(|new_entry| Entry { deps: entries.get_deps(new_entry), ..new_entry });

                let (is_allocated, entries_next) =
                    if let Some(new_entry) = new_entry { entries.try_alloc(new_entry) } else { (false, entries) };

                let ir = (Ready::new(is_allocated, ()), (), (), ());

                let (issued_ld, conv_ld_issue_completed, matmul_ld_issue_completed, entries_next) = if er_ld.ready {
                    let (issued, entries_next) = entries_next.try_issue(Q::Ld);
                    let complete_on_issue = issued.is_some_and(|p| p.1);
                    let from_conv_fsm = issued.is_some_and(|p| p.0.cmd.from_conv_fsm);
                    let from_matmul_fsm = issued.is_some_and(|p| p.0.cmd.from_matmul_fsm);

                    (
                        issued.map(|p| p.0),
                        complete_on_issue && from_conv_fsm,
                        complete_on_issue && from_matmul_fsm,
                        entries_next,
                    )
                } else {
                    (None, false, false, entries_next)
                };
                let (issued_ex, conv_ex_issue_completed, matmul_ex_issue_completed, entries_next) = if er_ex.ready {
                    let (issued, entries_next) = entries_next.try_issue(Q::Ex);
                    let complete_on_issue = issued.is_some_and(|p| p.1);
                    let from_conv_fsm = issued.is_some_and(|p| p.0.cmd.from_conv_fsm);
                    let from_matmul_fsm = issued.is_some_and(|p| p.0.cmd.from_matmul_fsm);

                    (
                        issued.map(|p| p.0),
                        complete_on_issue && from_conv_fsm,
                        complete_on_issue && from_matmul_fsm,
                        entries_next,
                    )
                } else {
                    (None, false, false, entries_next)
                };
                let (issued_st, conv_st_issue_completed, matmul_st_issue_completed, entries_next) = if er_st.ready {
                    let (issued, entries_next) = entries_next.try_issue(Q::St);
                    let complete_on_issue = issued.is_some_and(|p| p.1);
                    let from_conv_fsm = issued.is_some_and(|p| p.0.cmd.from_conv_fsm);
                    let from_matmul_fsm = issued.is_some_and(|p| p.0.cmd.from_matmul_fsm);

                    (
                        issued.map(|p| p.0),
                        complete_on_issue && from_conv_fsm,
                        complete_on_issue && from_matmul_fsm,
                        entries_next,
                    )
                } else {
                    (None, false, false, entries_next)
                };

                let (conv_ld_completed, matmul_ld_completed, entries_next) = if let Some(id) = completed_ld {
                    let (entry, entries_next) = entries_next.compute_completed(Q::Ld, id);
                    (entry.cmd.from_conv_fsm, entry.cmd.from_matmul_fsm, entries_next)
                } else {
                    (false, false, entries_next)
                };
                let (conv_ex_completed, matmul_ex_completed, entries_next) = if let Some(id) = completed_ex {
                    let (entry, entries_next) = entries_next.compute_completed(Q::Ex, id);
                    (entry.cmd.from_conv_fsm, entry.cmd.from_matmul_fsm, entries_next)
                } else {
                    (false, false, entries_next)
                };
                let (conv_st_completed, matmul_st_completed, entries_next) = if let Some(id) = completed_st {
                    let (entry, entries_next) = entries_next.compute_completed(Q::St, id);
                    (entry.cmd.from_conv_fsm, entry.cmd.from_matmul_fsm, entries_next)
                } else {
                    (false, false, entries_next)
                };

                let conv_ld_completed = conv_ld_issue_completed.into_u::<1>() + conv_ld_completed.into_u::<1>();
                let conv_ex_completed = conv_ex_issue_completed.into_u::<1>() + conv_ex_completed.into_u::<1>();
                let conv_st_completed = conv_st_issue_completed.into_u::<1>() + conv_st_completed.into_u::<1>();

                let matmul_ld_completed = matmul_ld_issue_completed.into_u::<1>() + matmul_ld_completed.into_u::<1>();
                let matmul_ex_completed = matmul_ex_issue_completed.into_u::<1>() + matmul_ex_completed.into_u::<1>();
                let matmul_st_completed = matmul_st_issue_completed.into_u::<1>() + matmul_st_completed.into_u::<1>();

                let ep = (
                    (issued_ld, Some(conv_ld_completed), Some(matmul_ld_completed)),
                    (issued_ex, Some(conv_ex_completed), Some(matmul_ex_completed)),
                    (issued_st, Some(conv_st_completed), Some(matmul_st_completed)),
                    Some(
                        entries.entries_ld.any(|e| e.is_some())
                            || entries.entries_ex.any(|e| e.is_some())
                            || entries.entries_st.any(|e| e.is_some()),
                    ),
                );

                (ep, ir, entries_next)
            },
        )
    }
}

/// Reservation Station
///
/// Due to Gemmini's decoupled access-execute architecture, instructions in the `Load``, `Store`, and `Execute` may operate concurrently and out-of-order with respect to instructions in other modules.
/// This module detects hazards between instructions in `Load`, `Store`, and `Execute`
/// The instructions in the Reservation Station are only issued to their respective modules once they have no dependencies on instructions in other modules.
///
/// Note:
/// Instructions that are destined for the same modules are issued in-order.
/// Reservation Station does not check hazards between instructions within the same module(`Load`, `Store`, and `Execute`)
/// Each module is obligated to handle it's own dependencies and hazards internally, assuming that it receives it's own instructions in program-order.
///
/// # Inputs
///
/// - `alloc`
///     + Allocated instructions from the `LoopMatmul` module.
/// - `completed`
///     + Completed and arbitered instructions from the `Load`, `Store`, and `Execute` modules.
///     + Note that this signal comes from the egress of the `Load`, `Store`, and `Execute` modules.
///     + This implementation preassumes that `module_split` will be applied to this module.
///     + For more information, see the `mod.rs` file in the `crate::gemmini`.
///
/// # Outputs
///
/// - `RsIssues`
///    + Instructions to be issued to the `Load`, `Execute`, and `Store` modules.
/// - `RsCompleted`
///    + Completed instruction IDs from the Reservation Station.
///    + This is sent to `LoopMatmul` and `LoopConv` modules.
/// - `busy`
///    + Reservation station is busy or not.
#[synthesize]
pub fn reservation_station(
    alloc: Vr<GemminiCmd>,
    completed: Valid<U<{ clog2(RS_ENTRIES) }>>,
) -> (RsIssues, RsCompleted, Valid<bool>) {
    let alloc = alloc.fsm_map::<Entry, Config>(Config::default(), |ip, s| {
        let (new_entry, is_norm) = decode_cmd(ip, s);
        let s_next = update_config(new_entry, is_norm, s);

        (new_entry, s_next)
    });

    let [completed_ld, completed_ex, completed_st] = completed
        .map::<(U<{ clog2(RS_MAX_PER_TYPE) }>, BoundedU<3>)>(|p| {
            let q = p.clip_const::<2>(CL_RS_MAX_PER_TYPE);
            let sel = BoundedU::new(q);
            (p.clip_const::<{ clog2(RS_MAX_PER_TYPE) }>(0), sel)
        })
        .branch();

    let ((issue_ld, conv_ld, matmul_ld), (issue_ex, conv_ex, matmul_ex), (issue_st, conv_st, matmul_st), busy) =
        (alloc, completed_ld, completed_ex, completed_st).comb(move |(i1, i2, i3, i4)| queues(i1, i2, i3, i4));

    let rs_issues = RsIssues { ld: issue_ld, ex: issue_ex, st: issue_st };
    let rs_completed = RsCompleted { conv_ld, conv_ex, conv_st, matmul_ld, matmul_ex, matmul_st };

    (rs_issues, rs_completed, busy)
}
