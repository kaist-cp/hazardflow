//! Expr.

use std::cell::RefCell;
use std::cmp::Ordering;

use hashcons::merkle::Merkle;
use rustc_span::Span;

use super::*;
use crate::utils::*;

/// Expr Id
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(usize);

impl ExprId {
    /// Allocates expr to the table and returns the id
    pub fn alloc_expr(expr: Expr) -> Self {
        TABLE.with(|table| table.push(Merkle::new(expr)))
    }

    /// Returns expr corresponding to given id
    pub fn into_expr(self) -> Merkle<Expr> {
        TABLE.with(|table| table.get(self))
    }

    /// Returns Member expr
    pub fn member(self, index: usize, span: Span) -> Expr {
        match self.into_expr().port_decls() {
            PortDecls::Struct(inner) => {
                assert!(index < inner.len(), "{:#?} {index}", inner);
                Expr::Member { inner: self, index, span }
            }
            PortDecls::Bits(_) => todo!(),
        }
    }
}

/// Expr Table
#[derive(Default)]
pub struct Table {
    inner: RefCell<Vec<Merkle<Expr>>>,
}

impl std::fmt::Debug for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Table").field("inner", &self.inner).finish()
    }
}

impl Table {
    /// Inserts expr into table.
    fn get(&self, id: ExprId) -> Merkle<Expr> {
        self.inner.borrow().get(id.0).expect("does not have element!").clone()
    }

    /// Returns expr from table by using id.
    fn push(&self, expr: Merkle<Expr>) -> ExprId {
        let id = self.inner.borrow().len();
        self.inner.borrow_mut().push(expr);
        ExprId(id)
    }
}

thread_local! {
    /// Expr Table
    pub(crate) static TABLE: Table = Table::default();
}

#[doc(hidden)]
pub trait TableStorageElement<'id> {}

/// Exprs.
#[derive(Debug, Clone, Eq)]
pub enum Expr {
    /// Don't-care value
    X {
        /// Value type of the expr
        typ: PortDecls,

        /// Span of the expr
        span: Span,
    },

    /// Constant value
    Constant {
        /// Bitvector constant
        bits: Vec<bool>,

        /// Value type of the expr
        typ: PortDecls,

        /// Span of the expr
        span: Span,
    },

    /// Repeated expr
    Repeat {
        /// The repeated expr
        inner: ExprId,

        /// Repeat count
        count: usize,

        /// Span of the expr
        span: Span,
    },

    /// Variable
    Var {
        /// Name of the variable
        name: Option<String>,

        /// Value type of the expr
        typ: PortDecls,

        /// Span of the expr
        span: Span,
    },

    /// Member of expr
    Member {
        /// The inner expr
        inner: ExprId,

        /// Index of the member
        index: usize,

        /// Span of the expr
        span: Span,
    },

    /// Combine exprs
    Struct {
        /// The inner exprs
        inner: Vec<(Option<String>, ExprId)>,

        /// Span of the expr
        span: Span,
    },

    /// Logical negation: `!inner`
    Not {
        /// The input expr
        inner: ExprId,

        /// Span of the expr
        span: Span,
    },

    /// Binary operation: `op lhs rhs`
    BinaryOp {
        /// Operator
        op: BinaryOp,

        /// Lhs
        lhs: ExprId,

        /// Rhs
        rhs: ExprId,

        /// Span of the expr
        span: Span,
    },

    /// Fold (bitwise)
    Fold {
        /// The inner expr
        inner: ExprId,

        /// Value type of the element expr
        typ_elt: PortDecls,

        /// Fold operator
        func: FunctionId,

        /// Fold initial value
        init: ExprId,

        // /// Fold accumulator
        // acc: ExprId,
        //
        // /// Inner slice
        // inner_slice: ExprId,
        /// Span of the expr
        span: Span,
    },

    /// Tree Fold
    TreeFold {
        /// The inner expr
        inner: ExprId,

        /// acc
        acc: ExprId,

        /// op
        ///
        /// TODO: Use FunctionId
        op: ExprId,

        /// lhs,
        lhs: ExprId,

        /// rhs,
        rhs: ExprId,

        /// Span of the expr
        span: Span,
    },

    /// Mapped expr
    Map {
        /// The inner expr
        inner: ExprId,

        /// Value type of the element expr
        typ_elt: PortDecls,

        /// Length of the inner expr
        len: usize,

        /// Map function
        func: FunctionId,

        /// Value type of the function return
        func_ret_typ: PortDecls,

        /// Span of the expr
        span: Span,
    },

    /// Range (N..M)
    Range {
        /// From
        len: usize,

        /// Value type of the element expr
        typ_elt: PortDecls,

        /// Span of the expr
        span: Span,
    },

    /// Indexing: `inner[index]`
    Get {
        /// The inner expr
        inner: ExprId,

        /// Value type of the element expr
        typ_elt: PortDecls,

        /// Index
        index: ExprId,

        /// Span of the expr
        span: Span,
    },

    /// Primitive casting.
    ///
    /// You can only cast from `Bits` to `Bits`.
    Cast {
        /// The inner expr
        from: ExprId,

        /// The type to cast to
        to: Shape,

        /// Span of the expr
        span: Span,
    },

    /// Clip: `inner[from..to]`
    Clip {
        /// The inner expr
        inner: ExprId,

        /// Value type of the element expr
        typ_elt: PortDecls,

        /// Starting index
        from: ExprId,

        /// Array size
        size: usize,

        /// Span of the expr
        span: Span,
    },

    /// Append
    Append {
        /// Lhs
        lhs: ExprId,

        /// Rhs
        rhs: ExprId,

        /// Value type of the element expr
        typ_elt: PortDecls,

        /// Span of the expr
        span: Span,
    },

    /// Zip exprs.
    Zip {
        /// Inner exprs.
        inner: Vec<ExprId>,

        /// Value type of the element exprs.
        typ_inner: Vec<PortDecls>,

        /// Span of the expr
        span: Span,
    },
    /// Concatenate (2-dimensional to 1-dimensional)
    Concat {
        /// The inner expr
        inner: ExprId,

        /// Value type of the element expr
        typ_elt: PortDecls,

        /// Span of the expr
        span: Span,
    },

    /// Chunk (1-dimensional to 2-dimensional)
    Chunk {
        /// The inner expr
        inner: ExprId,

        /// Chunk size
        chunk_size: usize,

        /// Span of the expr
        span: Span,
    },

    /// Flatten to 1-dimensional
    Repr {
        /// The inner expr
        inner: ExprId,

        /// Span of the expr
        span: Span,
    },

    /// Conditional operator: `if cond begin lhs end else rhs end`
    Cond {
        /// The condition expr
        cond_expr_pair: Vec<(ExprId, ExprId)>,

        /// Output when the condition is false
        default: ExprId,

        /// Span of the expr
        span: Span,
    },

    /// TODO: Documentation
    Set {
        /// The inner expr
        inner: ExprId,

        /// Index of the element
        index: ExprId,

        /// The value after the change
        elt: ExprId,

        /// Span of the expr
        span: Span,
    },

    /// TODO: Documentation
    SetRange {
        /// The inner expr
        inner: ExprId,

        /// Value type of the element expr
        typ_elt: PortDecls,

        /// Index of the element
        index: ExprId,

        /// The value after the change
        elts: ExprId,

        /// Span of the expr
        span: Span,
    },

    /// Case statement for verilog. Should always contain a default value.
    Case {
        /// The case expression
        case_expr: ExprId,

        /// Vec of (case, assignment) pairs; doesn't need to be constant
        case_items: Vec<(ExprId, ExprId)>,

        /// Default expr
        default: Option<ExprId>,

        /// Span of the expr
        span: Span,
    },

    /// `[Expr<V>; N]` to `Expr<[V; N]>`
    ConcatArray {
        /// length N vector of exprs with type elt_typ
        inner: Vec<ExprId>,

        /// elemtent tyoe
        elt_typ: PortDecls,

        /// Span of the expr
        span: Span,
    },
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::X { typ: l_typ, .. }, Self::X { typ: r_typ, .. }) => l_typ == r_typ,
            (Self::Constant { bits: l_bits, typ: l_typ, .. }, Self::Constant { bits: r_bits, typ: r_typ, .. }) => {
                l_bits == r_bits && l_typ == r_typ
            }
            (
                Self::Repeat { inner: l_inner, count: l_count, .. },
                Self::Repeat { inner: r_inner, count: r_count, .. },
            ) => l_inner == r_inner && l_count == r_count,
            (Self::Var { name: l_name, typ: l_typ, .. }, Self::Var { name: r_name, typ: r_typ, .. }) => {
                l_name == r_name && l_typ == r_typ
            }
            (
                Self::Member { inner: l_inner, index: l_index, .. },
                Self::Member { inner: r_inner, index: r_index, .. },
            ) => l_inner == r_inner && l_index == r_index,
            (Self::Struct { inner: l_inner, .. }, Self::Struct { inner: r_inner, .. }) => l_inner == r_inner,
            (Self::Not { inner: l_inner, .. }, Self::Not { inner: r_inner, .. }) => l_inner == r_inner,
            (
                Self::BinaryOp { op: l_op, lhs: l_lhs, rhs: l_rhs, .. },
                Self::BinaryOp { op: r_op, lhs: r_lhs, rhs: r_rhs, .. },
            ) => l_op == r_op && l_lhs == r_lhs && l_rhs == r_rhs,
            (
                Self::Fold { inner: l_inner, typ_elt: l_typ_elt, func: l_func, init: l_init, .. },
                Self::Fold { inner: r_inner, typ_elt: r_typ_elt, func: r_func, init: r_init, .. },
            ) => l_inner == r_inner && l_typ_elt == r_typ_elt && l_func == r_func && l_init == r_init,
            (
                Self::TreeFold { inner: l_inner, acc: l_acc, op: l_op, lhs: l_lhs, rhs: l_rhs, .. },
                Self::TreeFold { inner: r_inner, acc: r_acc, op: r_op, lhs: r_lhs, rhs: r_rhs, .. },
            ) => l_inner == r_inner && l_acc == r_acc && l_op == r_op && l_lhs == r_lhs && l_rhs == r_rhs,
            (
                Self::Map { inner: l_inner, typ_elt: l_typ_elt, func: l_func, .. },
                Self::Map { inner: r_inner, typ_elt: r_typ_elt, func: r_func, .. },
            ) => l_inner == r_inner && l_typ_elt == r_typ_elt && l_func == r_func,
            (
                Self::Range { len: l_len, typ_elt: l_typ_elt, .. },
                Self::Range { len: r_len, typ_elt: r_typ_elt, .. },
            ) => l_len == r_len && l_typ_elt == r_typ_elt,
            (
                Self::Get { inner: l_inner, typ_elt: l_typ_elt, index: l_index, .. },
                Self::Get { inner: r_inner, typ_elt: r_typ_elt, index: r_index, .. },
            ) => l_inner == r_inner && l_typ_elt == r_typ_elt && l_index == r_index,
            (
                Self::Clip { inner: l_inner, typ_elt: l_typ_elt, from: l_from, size: l_size, .. },
                Self::Clip { inner: r_inner, typ_elt: r_typ_elt, from: r_from, size: r_size, .. },
            ) => l_inner == r_inner && l_typ_elt == r_typ_elt && l_from == r_from && l_size == r_size,
            (
                Self::Append { lhs: l_lhs, rhs: l_rhs, typ_elt: l_typ_elt, .. },
                Self::Append { lhs: r_lhs, rhs: r_rhs, typ_elt: r_typ_elt, .. },
            ) => l_lhs == r_lhs && l_rhs == r_rhs && l_typ_elt == r_typ_elt,
            (
                Self::Zip { inner: l_inner, typ_inner: l_typ_inner, .. },
                Self::Zip { inner: r_inner, typ_inner: r_typ_inner, .. },
            ) => l_inner == r_inner && l_typ_inner == r_typ_inner,
            (
                Self::Concat { inner: l_inner, typ_elt: l_typ_elt, .. },
                Self::Concat { inner: r_inner, typ_elt: r_typ_elt, .. },
            ) => l_inner == r_inner && l_typ_elt == r_typ_elt,
            (
                Self::Chunk { inner: l_inner, chunk_size: l_chunk_size, .. },
                Self::Chunk { inner: r_inner, chunk_size: r_chunk_size, .. },
            ) => l_inner == r_inner && l_chunk_size == r_chunk_size,
            (Self::Repr { inner: l_inner, .. }, Self::Repr { inner: r_inner, .. }) => l_inner == r_inner,
            (
                Self::Cond { cond_expr_pair: l_cond_expr_pair, default: l_default, .. },
                Self::Cond { cond_expr_pair: r_cond_expr_pair, default: r_default, .. },
            ) => l_cond_expr_pair == r_cond_expr_pair && l_default == r_default,
            (
                Self::Set { inner: l_inner, index: l_index, elt: l_elt, .. },
                Self::Set { inner: r_inner, index: r_index, elt: r_elt, .. },
            ) => l_inner == r_inner && l_index == r_index && l_elt == r_elt,
            (
                Self::SetRange { inner: l_inner, typ_elt: l_typ_elt, index: l_index, elts: l_elts, .. },
                Self::SetRange { inner: r_inner, typ_elt: r_typ_elt, index: r_index, elts: r_elts, .. },
            ) => l_inner == r_inner && l_typ_elt == r_typ_elt && l_index == r_index && l_elts == r_elts,
            (
                Self::Case { case_expr: l_case_expr, case_items: l_case_items, default: l_default, .. },
                Self::Case { case_expr: r_case_expr, case_items: r_case_items, default: r_default, .. },
            ) => l_case_expr == r_case_expr && l_case_items == r_case_items && l_default == r_default,
            (
                Self::ConcatArray { inner: l_inner, elt_typ: l_elt_typ, .. },
                Self::ConcatArray { inner: r_inner, elt_typ: r_elt_typ, .. },
            ) => l_inner == r_inner && l_elt_typ == r_elt_typ,
            _ => false,
        }
    }
}

impl std::hash::Hash for Expr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Expr::X { typ, .. } => typ.hash(state),
            Expr::Constant { bits, typ, .. } => {
                bits.hash(state);
                typ.hash(state);
            }
            Expr::Repeat { inner, count, .. } => {
                inner.hash(state);
                count.hash(state)
            }
            Expr::Var { name, typ, .. } => {
                name.hash(state);
                typ.hash(state);
            }
            Expr::Member { inner, index, .. } => {
                inner.hash(state);
                index.hash(state);
            }
            Expr::Struct { inner, .. } => inner.hash(state),
            Expr::Not { inner, .. } => inner.hash(state),
            Expr::BinaryOp { op, lhs, rhs, .. } => {
                op.hash(state);
                lhs.hash(state);
                rhs.hash(state);
            }
            Expr::Fold { inner, typ_elt, func, init, .. } => {
                inner.hash(state);
                typ_elt.hash(state);
                func.hash(state);
                init.hash(state);
            }
            Expr::TreeFold { inner, acc, op, lhs, rhs, .. } => {
                inner.hash(state);
                acc.hash(state);
                op.hash(state);
                lhs.hash(state);
                rhs.hash(state);
            }
            Expr::Map { inner, typ_elt, func, .. } => {
                inner.hash(state);
                typ_elt.hash(state);
                func.hash(state);
            }
            Expr::Range { len, typ_elt, .. } => {
                len.hash(state);
                typ_elt.hash(state);
            }
            Expr::Get { inner, typ_elt, index, .. } => {
                inner.hash(state);
                typ_elt.hash(state);
                index.hash(state);
            }
            Expr::Clip { inner, typ_elt, from, size, .. } => {
                inner.hash(state);
                typ_elt.hash(state);
                from.hash(state);
                size.hash(state);
            }
            Expr::Append { lhs, rhs, typ_elt, .. } => {
                lhs.hash(state);
                rhs.hash(state);
                typ_elt.hash(state);
            }
            Expr::Zip { inner, typ_inner, .. } => {
                inner.hash(state);
                typ_inner.hash(state);
            }
            Expr::Concat { inner, typ_elt, .. } => {
                inner.hash(state);
                typ_elt.hash(state);
            }
            Expr::Chunk { inner, chunk_size, .. } => {
                inner.hash(state);
                chunk_size.hash(state);
            }
            Expr::Repr { inner, .. } => {
                inner.hash(state);
            }
            Expr::Cond { cond_expr_pair, default, .. } => {
                cond_expr_pair.hash(state);
                default.hash(state);
            }
            Expr::Set { inner, index, elt, .. } => {
                inner.hash(state);
                index.hash(state);
                elt.hash(state);
            }
            Expr::SetRange { inner, typ_elt, index, elts, .. } => {
                inner.hash(state);
                typ_elt.hash(state);
                index.hash(state);
                elts.hash(state);
            }
            Expr::Case { case_expr, case_items, default, .. } => {
                case_expr.hash(state);
                case_items.hash(state);
                default.hash(state);
            }
            Expr::ConcatArray { inner, elt_typ, .. } => {
                inner.hash(state);
                elt_typ.hash(state);
            }
            Expr::Cast { from, to, .. } => {
                from.hash(state);
                to.hash(state);
            }
        }
    }
}

impl Expr {
    /// Construct a unit expression.
    pub fn unit(span: Span) -> ExprId {
        ExprId::alloc_expr(Self::Struct { inner: vec![], span })
    }

    /// Returns the span of this expression.
    pub fn span(&self) -> Span {
        match self {
            Expr::X { span, .. }
            | Expr::Constant { span, .. }
            | Expr::Repeat { span, .. }
            | Expr::Var { span, .. }
            | Expr::Member { span, .. }
            | Expr::Struct { span, .. }
            | Expr::Not { span, .. }
            | Expr::BinaryOp { span, .. }
            | Expr::Fold { span, .. }
            | Expr::TreeFold { span, .. }
            | Expr::Map { span, .. }
            | Expr::Range { span, .. }
            | Expr::Get { span, .. }
            | Expr::Clip { span, .. }
            | Expr::Append { span, .. }
            | Expr::Zip { span, .. }
            | Expr::Concat { span, .. }
            | Expr::Chunk { span, .. }
            | Expr::Repr { span, .. }
            | Expr::Cond { span, .. }
            | Expr::Set { span, .. }
            | Expr::SetRange { span, .. }
            | Expr::Case { span, .. }
            | Expr::Cast { span, .. }
            | Expr::ConcatArray { span, .. } => *span,
        }
    }

    /// Constructs a input expr.
    pub fn input(name: Option<String>, typ: PortDecls, span: Span) -> Self {
        Self::Var { name, typ, span }
    }

    /// Constructs a unsigned bits.
    pub fn unsigned_bits(len: usize, value: usize, span: Span) -> Self {
        let bits = usize_to_bitvec(len, value);
        Self::Constant { bits, typ: PortDecls::unsigned_bits(len), span }
    }

    /// Constructs a unsigned constant.
    pub fn signed_bits(len: usize, value: usize, span: Span) -> Self {
        let bits = usize_to_bitvec(len, value);
        Self::Constant { bits, typ: PortDecls::signed_bits(len), span }
    }

    /// Allocate an expr with a cache.
    pub fn alloc_with_fsm_cache(self, cache: &mut FsmCache) -> ExprId {
        cache.alloc(self)
    }

    /// Cast from bits to bits
    pub fn cast_bits(from: ExprId, to_typ: PortDecls, cache: &mut FsmCache, span: Span) -> ExprId {
        Expr::Cast { from, to: to_typ.shape(), span }.alloc_with_fsm_cache(cache)
    }

    /// Construct a resize expr.
    pub fn resize(from: ExprId, from_width: usize, to_width: usize, cache: &mut FsmCache, span: Span) -> ExprId {
        let typ_elt = from.into_expr().port_decls().divide(from_width);

        match from_width.cmp(&to_width) {
            Ordering::Less => {
                assert!(from_width < to_width);
                let zero = Expr::unsigned_bits(typ_elt.width(), 0, span).alloc_with_fsm_cache(cache);
                let extra =
                    Expr::Repeat { inner: zero, count: to_width - from_width, span }.alloc_with_fsm_cache(cache);
                Expr::Append { lhs: from, rhs: extra, typ_elt, span }.alloc_with_fsm_cache(cache)
            }
            Ordering::Equal => from,
            Ordering::Greater => {
                let zero = Expr::unsigned_bits(clog2(from_width), 0, span).alloc_with_fsm_cache(cache);
                Expr::Clip { inner: from, typ_elt, from: zero, size: to_width, span }.alloc_with_fsm_cache(cache)
            }
        }
    }

    /// Type of the expr.
    pub fn port_decls(&self) -> PortDecls {
        match self {
            Self::X { typ, .. } => typ.clone(),
            Self::Constant { typ, .. } => typ.clone(),
            Self::Repeat { inner, count, .. } => inner.into_expr().port_decls().multiple(*count),
            Self::Var { typ, .. } => typ.clone(),
            Self::Member { inner, index, .. } => match inner.into_expr().port_decls() {
                PortDecls::Struct(inner) => inner[*index].clone().1,
                PortDecls::Bits(_) => panic!("Cannot index a `PortDecls::Bits`."),
            },
            Self::Struct { inner, .. } => PortDecls::Struct(
                inner.iter().map(|(name, member)| (name.clone(), member.into_expr().port_decls())).collect(),
            ),
            Self::BinaryOp { op, lhs, rhs, .. } => {
                let lhs_typ = lhs.into_expr().port_decls();
                let rhs_typ = rhs.into_expr().port_decls();
                match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Div
                    | BinaryOp::Mod
                    | BinaryOp::Or
                    | BinaryOp::And
                    | BinaryOp::Xor
                    | BinaryOp::Mul => {
                        // Context-determined operations.
                        // According to the IEEE Std 1364-2005(Verilog specification) Section 5.5.1(Rules for expression types),
                        // For nonself-determined (which includes context-determined) operands the result becomes signed onlyewhen both operands are signed.
                        if lhs_typ.is_signed() && rhs_typ.is_signed() {
                            PortDecls::signed_bits(self.width())
                        } else {
                            PortDecls::unsigned_bits(self.width())
                        }
                    }
                    BinaryOp::NeStrict
                    | BinaryOp::Eq
                    | BinaryOp::Less
                    | BinaryOp::Greater
                    | BinaryOp::LessEq
                    | BinaryOp::GreaterEq
                    | BinaryOp::EqArithmetic
                    | BinaryOp::NeArithmetic => {
                        // According to the IEEE Std 1364-2005(Verilog specification) Section 5.5.1(Rules for expression types),
                        // Comparison results are unsigned, regardless of the operands.
                        assert_eq!(lhs_typ.is_signed(), rhs_typ.is_signed());
                        assert_eq!(&lhs_typ, &rhs_typ);
                        PortDecls::unsigned_bits(self.width())
                    }
                    BinaryOp::ShiftRight | BinaryOp::ShiftLeft => {
                        if lhs_typ.is_signed() {
                            PortDecls::signed_bits(self.width())
                        } else {
                            PortDecls::unsigned_bits(self.width())
                        }
                    }
                }
            }
            Self::Chunk { inner, .. } => inner.into_expr().port_decls(),
            Self::Not { inner, .. } => inner.into_expr().port_decls(),
            Self::Fold { init, .. } => init.into_expr().port_decls(),
            Self::TreeFold { lhs, .. } => lhs.into_expr().port_decls(),
            Self::Clip { inner, from: _, size, typ_elt, .. } => {
                inner.into_expr().port_decls().divide(inner.into_expr().width() / typ_elt.width()).multiple(*size)
            }
            Self::Append { lhs, rhs, typ_elt, .. } => {
                let count = (lhs.into_expr().width() + rhs.into_expr().width()) / typ_elt.width();
                typ_elt.multiple(count)
            }
            Self::Get { typ_elt, .. } => typ_elt.clone(),
            // NOTE: signedness of repr result is always unsigned.
            Self::Repr { inner, .. } => PortDecls::unsigned_bits(inner.into_expr().width()),
            Self::Map { func_ret_typ, len, .. } => {
                let count = *len;
                func_ret_typ.multiple(count)
            }
            Self::Zip { inner, .. } => PortDecls::Struct(
                inner
                    .iter()
                    .enumerate()
                    .map(|(idx, expr_id)| (Some(idx.to_string()), expr_id.into_expr().port_decls()))
                    .collect(),
            ),
            Self::Concat { inner, typ_elt, .. } => {
                let count = inner.into_expr().width() / typ_elt.width();
                typ_elt.multiple(count)
            }
            Self::Cond { default, .. } => default.into_expr().port_decls(),
            Self::Set { inner, .. } => inner.into_expr().port_decls(),
            Self::SetRange { inner, .. } => inner.into_expr().port_decls(),
            Self::Case { case_items, default, .. } => {
                if case_items.is_empty() {
                    // If there are no cases, there must be a default case
                    default.as_ref().unwrap().into_expr().port_decls()
                } else {
                    let typ = case_items[0].1.into_expr().port_decls();
                    assert!(case_items.iter().all(|expr| expr.1.into_expr().port_decls() == typ));
                    if let Some(default) = &default {
                        assert_eq!(default.into_expr().port_decls(), typ);
                    }
                    typ
                }
            }
            Self::ConcatArray { inner, elt_typ, .. } => elt_typ.multiple(inner.len()),
            Self::Range { len, typ_elt, .. } => typ_elt.multiple(*len),
            Self::Cast { to, .. } => PortDecls::Bits(to.clone()),
        }
    }

    /// Computes width of the expr.
    // TODO: Memoization?
    pub fn width(&self) -> usize {
        match self {
            Self::X { typ, .. } => typ.width(),
            Self::Constant { bits, .. } => bits.len(),
            Self::BinaryOp { op, lhs, rhs, .. } => match op {
                BinaryOp::And
                | BinaryOp::Or
                | BinaryOp::Xor
                | BinaryOp::Sub
                | BinaryOp::ShiftRight
                | BinaryOp::ShiftLeft => lhs.into_expr().width(),
                BinaryOp::Add => {
                    let lhs_width = lhs.into_expr().width();
                    let rhs_width = rhs.into_expr().width();
                    assert_eq!(lhs_width, rhs_width);
                    lhs_width + 1
                }
                BinaryOp::Mul => lhs.into_expr().width() + rhs.into_expr().width(),
                BinaryOp::Div => lhs.into_expr().width(),
                BinaryOp::Mod => rhs.into_expr().width(),
                BinaryOp::EqArithmetic
                | BinaryOp::NeArithmetic
                | BinaryOp::Less
                | BinaryOp::Greater
                | BinaryOp::LessEq
                | BinaryOp::GreaterEq => {
                    let lhs_width = lhs.into_expr().width();
                    let rhs_width = rhs.into_expr().width();
                    assert_eq!(lhs_width, rhs_width);
                    1
                }
                _ => todo!("Unimplemented width for binary operator {:#?}", op),
            },
            Self::Member { inner, index, .. } => {
                let inner_typ = inner.into_expr().port_decls();
                match inner_typ {
                    PortDecls::Struct(inner) => inner[*index].1.width(),
                    PortDecls::Bits(_) => panic!("Cannot index a `PortDecls::Bits`."),
                }
            }
            Self::Concat { inner, .. } => inner.into_expr().width(),
            Self::Map { inner, typ_elt, func_ret_typ, .. } => {
                let inner_width = inner.into_expr().width();
                assert_eq!(inner_width % typ_elt.width(), 0);
                (inner_width / typ_elt.width()) * func_ret_typ.width()
            }
            Self::Repeat { inner, count, .. } => inner.into_expr().width() * count,
            Self::Var { typ, .. } => typ.width(),
            Self::Not { inner, .. } => inner.into_expr().width(),
            Self::Cond { cond_expr_pair, default, .. } => {
                let cond_expr_widths = cond_expr_pair
                    .iter()
                    .map(|(cond, expr)| (cond.into_expr().width(), expr.into_expr().width()))
                    .collect::<Vec<_>>();
                let default_width = default.into_expr().width();

                for (cond_width, lhs_width) in cond_expr_widths.iter() {
                    assert_eq!(*cond_width, 1);
                    assert_eq!(*lhs_width, default_width);
                }

                default_width
            }
            Self::Chunk { inner, .. } => inner.into_expr().width(),
            Self::Get { typ_elt, .. } => typ_elt.width(),
            Self::Clip { size, typ_elt, .. } => typ_elt.width() * (size),
            Self::Append { lhs, rhs, .. } => lhs.into_expr().width() + rhs.into_expr().width(),
            Self::Zip { inner, .. } => inner.iter().map(|expr_id| expr_id.into_expr().width()).sum(),
            Self::Repr { inner, .. } => inner.into_expr().width(),
            Self::Struct { inner, .. } => inner.iter().map(|(_, inner)| inner.into_expr().width()).sum(),
            Self::Set { inner, .. } => inner.into_expr().width(),
            Self::SetRange { inner, .. } => inner.into_expr().width(),
            Self::Fold { init, .. } => init.into_expr().width(),
            Self::TreeFold { lhs, .. } => lhs.into_expr().width(),
            Self::Case { case_items, default, .. } => {
                if case_items.is_empty() {
                    // If there are no cases, there must be a default case
                    default.as_ref().unwrap().into_expr().width()
                } else {
                    let width = case_items[0].1.into_expr().width();
                    assert!(case_items.iter().all(|expr| expr.1.into_expr().width() == width));
                    if let Some(default) = &default {
                        assert_eq!(default.into_expr().width(), width);
                    }
                    width
                }
            }
            Self::ConcatArray { inner, elt_typ, .. } => elt_typ.width() * inner.len(),
            Self::Range { len, typ_elt, .. } => typ_elt.width() * *len,
            Self::Cast { to, .. } => to.width(),
        }
    }

    pub(crate) fn tuple(args: Vec<ExprId>, span: Span) -> Self {
        Expr::Struct {
            inner: args
                .into_iter()
                .enumerate()
                .map(|(idx, field_expr_id)| (Some(idx.to_string()), field_expr_id))
                .collect(),
            span,
        }
    }
}
