//! Rustc Utilities

use std::collections::{HashMap, VecDeque};

use hir::def_id::{DefId, LocalDefId};
use hir::HirId;
use rustc_ast::{LitKind, StrStyle};
use rustc_const_eval::interpret::Scalar;
use rustc_hir as hir;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::mir::{BorrowKind, ConstValue};
use rustc_middle::thir::{self, ExprId, ExprKind, Param, Thir};
use rustc_middle::ty::fold::TypeFoldable;
use rustc_middle::ty::{
    Const, EarlyBinder, GenericArg, GenericPredicates, Instance, InstantiatedPredicates, ParamEnv, Ty, TyCtxt,
    UnevaluatedConst, ValTree, VariantDef,
};
use rustc_span::Span;
use rustc_target::abi::{FieldIdx, VariantIdx};
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::utils::clog2;
use crate::*;

/// Find a trait by name
pub fn find_trait_by_name(tcx: TyCtxt<'_>, name: &str) -> Option<DefId> {
    tcx.all_traits().find(|&trait_defid| tcx.item_name(trait_defid).to_string() == name)
}

/// Find all impls of a trait
pub fn find_trait_impls(tcx: TyCtxt<'_>, trait_id: DefId) -> Vec<hir::Impl<'_>> {
    tcx.all_impls(trait_id).map(|id| *tcx.hir_node_by_def_id(id.expect_local()).expect_item().expect_impl()).collect()
}

use once_cell::sync::Lazy;

/// Cache for stolen `Thir`
pub static mut STOLEN_THIRS: Lazy<HashMap<LocalDefId, Thir<'static>>> = Lazy::new(HashMap::new);

/// Copy `Thir` before it is stolen
pub fn copy_thir_before_steal(def: LocalDefId, thir: Thir<'_>) {
    unsafe {
        let thir = std::mem::transmute::<Thir<'_>, Thir<'static>>(thir);
        STOLEN_THIRS.insert(def, thir);
    }
}

/// Retreive `Thir` given `LocalDefId
pub fn thir_body<'tcx>(tcx: TyCtxt<'tcx>, id: LocalDefId) -> &rustc_data_structures::steal::Steal<Thir<'tcx>> {
    assert!(!tcx.is_constructor(id.to_def_id()));

    let stolen = unsafe { std::mem::transmute::<Option<&Thir<'static>>, Option<&Thir<'tcx>>>(STOLEN_THIRS.get(&id)) };
    if let Some(thir) = stolen {
        return tcx.alloc_steal_thir(thir.clone());
    }

    let steal = tcx.thir_body(id).unwrap().0;
    steal
}

/// Retreive function parameter names
pub fn get_param_name(param: &Param<'_>) -> Option<String> {
    param.pat.as_ref().map(|p| p.to_string())
}

/// Normalize an alias type
pub fn normalize_alias_ty<'tcx, T>(tcx: TyCtxt<'tcx>, ty: T) -> T
where T: TypeFoldable<TyCtxt<'tcx>> {
    let infcx = &tcx.infer_ctxt().build();
    let ocx = ObligationCtxt::new(infcx);
    let normalized = ocx.normalize(&ObligationCause::dummy(), ParamEnv::reveal_all(), ty);
    let _unused = ocx.select_all_or_error();
    infcx.resolve_vars_if_possible(normalized)
}

/// Skip exprs that are not used in the module graph.
#[allow(clippy::needless_lifetimes)]
pub fn skip_exprs<'tcx>(body: &Thir<'tcx>, expr_id: ExprId) -> ExprId {
    let expr = &body.exprs[expr_id].kind;
    match expr {
        ExprKind::Call { .. }
        | ExprKind::Field { .. }
        | ExprKind::Index { .. }
        | ExprKind::VarRef { .. }
        | ExprKind::UpvarRef { .. }
        | ExprKind::Array { .. }
        | ExprKind::Tuple { .. }
        | ExprKind::Adt(_)
        | ExprKind::Deref { .. }
        | ExprKind::Closure(_)
        | ExprKind::Match { .. }
        | ExprKind::Literal { .. }
        | ExprKind::If { .. }
        | ExprKind::Binary { .. }
        | ExprKind::Unary { .. }
        | ExprKind::Repeat { .. }
        | ExprKind::Return { .. }
        | ExprKind::Let { .. }
        | ExprKind::LogicalOp { .. }
        | ExprKind::Cast { .. }
        | ExprKind::NamedConst { .. }
        | ExprKind::ConstParam { .. }
        | ExprKind::ZstLiteral { .. } => expr_id,
        ExprKind::Scope { value, .. } => skip_exprs(body, *value),
        ExprKind::Block { block } => {
            let block = &body.blocks[*block];
            match block.expr {
                Some(expr_id) => skip_exprs(body, expr_id),
                None => todo!(),
            }
        }
        ExprKind::Use { source } => skip_exprs(body, *source),
        // NOTE: This should not be skipped, since it has to be translated into `Expr::X`, rather
        // than actually translating the panic function.
        ExprKind::NeverToAny { .. } => expr_id,
        ExprKind::Borrow { arg, borrow_kind } => {
            // WARN: we are skipping the borrows(`&`).
            // - This path should be only allowed for closures, since closures silently borrows the
            // captured variables.
            // - This path should be only allowed for shared borrows(`&`), since mutable borrows
            // should not exist in hazardflow
            assert_eq!(borrow_kind, &BorrowKind::Shared);
            skip_exprs(body, *arg)
        }
        unimplmented => todo!("{unimplmented:?}"),
    }
}

/// Pattern Accessor Node
#[derive(Debug)]
pub enum PatAccessNode {
    /// Field Access
    Field {
        /// Field Index.
        idx: FieldIdx,

        /// Field Name.
        name: String,
    },

    /// Variant Access
    Variant {
        /// Variant Index
        idx: VariantIdx,

        /// Variant Name
        name: String,

        /// Discriminator
        discriminator: u32,
    },

    /// Array Access
    Index(usize),
}

/// Pattern Accessor
#[derive(Debug)]
pub struct PatAccessor {
    inner: VecDeque<PatAccessNode>,
}

impl PatAccessor {
    fn empty() -> Self {
        Self { inner: VecDeque::new() }
    }

    fn prepend(mut self, node: PatAccessNode) -> Self {
        self.inner.push_front(node);
        self
    }

    /// Return true if the accessor is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Return the length of the accessor
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Iterate on the from innermost access
    pub fn iter(&self) -> impl Iterator<Item = &PatAccessNode> {
        self.inner.iter()
    }
}

fn find_localvar_from_pat(tcx: TyCtxt<'_>, pat: &thir::Pat<'_>, local_var_id: thir::LocalVarId) -> Vec<PatAccessor> {
    let mut accessors = vec![];

    match &pat.kind {
        thir::PatKind::Binding { var, subpattern, .. } => {
            assert!(subpattern.is_none());
            if *var == local_var_id {
                accessors.push(PatAccessor::empty())
            }
        }
        thir::PatKind::AscribeUserType { subpattern, .. } => {
            accessors.append(&mut find_localvar_from_pat(tcx, subpattern.as_ref(), local_var_id))
        }
        thir::PatKind::Wild => {}
        thir::PatKind::Variant { adt_def, variant_index, subpatterns, .. } => match adt_def.adt_kind() {
            rustc_middle::ty::AdtKind::Enum => {
                for pat in subpatterns.iter() {
                    for accessor in find_localvar_from_pat(tcx, pat.pattern.as_ref(), local_var_id) {
                        let variant_def: &VariantDef = &adt_def.variants()[*variant_index];

                        let field_def = &variant_def.fields[pat.field];

                        let accessor = accessor
                            .prepend(PatAccessNode::Field { idx: pat.field, name: field_def.name.to_string() })
                            .prepend(PatAccessNode::Variant {
                                idx: *variant_index,
                                name: variant_def.name.to_ident_string(),
                                discriminator: get_variant_discriminator(tcx, variant_def),
                            });
                        accessors.push(accessor)
                    }
                }
            }
            rustc_middle::ty::AdtKind::Struct => todo!(),
            rustc_middle::ty::AdtKind::Union => todo!(),
        },
        thir::PatKind::Leaf { subpatterns } => match pat.ty.kind() {
            rustc_type_ir::TyKind::Adt(adt_def, _) => {
                assert!(adt_def.is_struct());

                for pat in subpatterns.iter() {
                    for accessor in find_localvar_from_pat(tcx, pat.pattern.as_ref(), local_var_id) {
                        {
                            let field_def = &adt_def.variants()[0u32.into()].fields[pat.field];
                            let accessor = accessor.prepend(PatAccessNode::Field {
                                idx: pat.field,
                                name: field_def.name.to_ident_string(),
                            });
                            accessors.push(accessor)
                        }
                    }
                }
            }
            rustc_type_ir::TyKind::Tuple(_) => {
                for (i, pat) in subpatterns.iter().enumerate() {
                    for accessor in find_localvar_from_pat(tcx, pat.pattern.as_ref(), local_var_id) {
                        let accessor = accessor.prepend(PatAccessNode::Field { idx: pat.field, name: i.to_string() });
                        accessors.push(accessor)
                    }
                }
            }
            _ => todo!(),
        },
        thir::PatKind::Deref { .. } => todo!(),
        thir::PatKind::Constant { .. } => {}
        thir::PatKind::Range(_) => todo!(),
        thir::PatKind::Slice { .. } => todo!(),
        thir::PatKind::Array { prefix, slice, suffix } => {
            assert!(slice.is_none());
            assert!(suffix.is_empty());
            for (i, pat) in prefix.iter().enumerate() {
                for accessor in find_localvar_from_pat(tcx, pat.as_ref(), local_var_id) {
                    let accessor = accessor.prepend(PatAccessNode::Index(i));
                    accessors.push(accessor)
                }
            }
        }
        thir::PatKind::Or { pats } => {
            for pat in pats.iter() {
                for accessor in find_localvar_from_pat(tcx, pat, local_var_id) {
                    accessors.push(accessor)
                }
            }
        }
        thir::PatKind::InlineConstant { .. } => todo!(),
        thir::PatKind::Never => todo!(),
        thir::PatKind::Error(_) => todo!(),
    }

    accessors
}

/// Returns the LocalVarId and Initializer given `Stmt`.
fn find_localvar_from_stmt<'tcx>(
    tcx: TyCtxt<'tcx>,
    stmt: &thir::Stmt<'tcx>,
    local_var_id: thir::LocalVarId,
) -> Vec<(thir::Pat<'tcx>, PatAccessor, ExprId)> {
    match &stmt.kind {
        thir::StmtKind::Let { pattern, initializer, .. } => find_localvar_from_pat(tcx, pattern, local_var_id)
            .into_iter()
            .map(|acc| (pattern.as_ref().clone(), acc, initializer.unwrap()))
            .collect(),
        _ => vec![],
    }
}

/// Local Variable
#[derive(Debug)]
pub enum LocalVar<'tcx> {
    /// Function parameter
    Param {
        /// Parameter index
        ///
        /// TODO: we might need more sophisticated datatype in case of destructuring
        arg_idx: usize,

        /// Accessor
        accessor: PatAccessor,

        /// Pattern
        pat: thir::Pat<'tcx>,
    },

    /// Statement
    Stmt {
        /// ExprId of the statement initializer
        ///
        /// Only storing initializer is enough, because any re-assignments on local variables or
        /// unassigned local variables are not allowed in hazardflow
        expr_id: ExprId,

        /// Accessor
        accessor: PatAccessor,

        /// Pattern
        pat: thir::Pat<'tcx>,
    },

    /// Pattern
    PatBinding {
        /// ExprId of the pattern binding
        expr_id: ExprId,

        /// Accessor
        accessor: PatAccessor,

        /// Pattern
        pat: thir::Pat<'tcx>,
    },
}

/// A pair of pattern and thir expression bounded to it.
#[derive(Debug, Clone)]
pub struct PatBinding<'tcx> {
    /// The Bounded Expression.
    pub id: ExprId,

    /// The Pattern.
    pub patterns: Vec<thir::Pat<'tcx>>,
}

/// Returns the local variable given `LocalVarId`.
///
/// A local variable can be either a function parameter or a statement.
///
/// TODO: pat_bindings should not be `Option`
pub fn resolve_var_ref<'tcx>(
    tcx: TyCtxt<'tcx>,
    thir_body: &'tcx rustc_data_structures::steal::Steal<Thir<'tcx>>,
    local_var_id: thir::LocalVarId,
    pat_bindings: Option<&[PatBinding<'tcx>]>,
) -> Vec<LocalVar<'tcx>> {
    let mut param_matched = vec![];
    for (i, param) in thir_body.borrow().params.iter().enumerate() {
        if let Some((pat, accessors)) =
            param.pat.as_ref().map(|pat| (pat.clone(), find_localvar_from_pat(tcx, pat.as_ref(), local_var_id)))
        {
            for accessor in accessors {
                param_matched.push(LocalVar::Param { arg_idx: i, accessor, pat: pat.as_ref().clone() });
            }
        }
    }

    let mut stmt_matched = vec![];
    for stmt in thir_body.borrow().stmts.iter() {
        for (pat, accessor, initializer) in find_localvar_from_stmt(tcx, stmt, local_var_id) {
            stmt_matched.push(LocalVar::Stmt { expr_id: initializer, accessor, pat });
        }
    }

    let mut pat_bindings_matched = vec![];
    if let Some(pat_bindings) = pat_bindings {
        for PatBinding { id, patterns } in pat_bindings {
            for pattern in patterns.iter() {
                for accessor in find_localvar_from_pat(tcx, pattern, local_var_id) {
                    pat_bindings_matched.push(LocalVar::PatBinding { expr_id: *id, accessor, pat: pattern.clone() })
                }
            }
        }
    }

    match (param_matched.is_empty(), stmt_matched.is_empty(), pat_bindings_matched.is_empty()) {
        (false, true, true) => param_matched,
        (true, false, true) => stmt_matched,
        (true, true, false) => pat_bindings_matched,
        _ => panic!(
            "Unexpected local variable binding:\n{param_matched:#?}\n{stmt_matched:#?}\n{pat_bindings_matched:#?}"
        ),
    }
}

/// Thir Extension
pub trait ThirExt<'tcx> {
    /// Pretty Print Thir
    fn print(&self);
}

impl<'tcx> ThirExt<'tcx> for Thir<'tcx> {
    fn print(&self) {
        println!(">> Print Thir");
        println!("body type: {:#?}", self.body_type);

        println!("Blocks:");
        for (i, x) in self.blocks.iter().enumerate() {
            println!("b{i}:{:#?}", x);
        }
        println!("Stmts:");
        for (i, x) in self.stmts.iter().enumerate() {
            println!("s{i}:{:#?}", x);
        }
        println!("Params:");
        for (i, x) in self.params.iter().enumerate() {
            println!("p{i}:{:#?}", x);
        }
        println!("Exprs:");
        for (i, x) in self.exprs.iter().enumerate() {
            println!("e{i}:{:#?}", x);
        }
        println!("Arms:");
        for (i, x) in self.arms.iter().enumerate() {
            println!("a{i}:{:#?}", x);
        }
    }
}

/// Given function and its arguments, checks if the function is a closure call.
///
/// Returns `true` if the call expression is a calling a closure
pub fn is_closure_call_with_id<'tcx>(
    tcx: TyCtxt<'tcx>,
    caller_body: &'tcx rustc_data_structures::steal::Steal<Thir<'tcx>>,
    fun_id: ExprId,
    args: &[ExprId],
) -> bool {
    if args.len() == 2 {
        let fun_expr = &caller_body.borrow()[fun_id];

        is_closure_call(fun_expr, tcx)
    } else {
        false
    }
}

/// Given expression, checks if the function is a closure call.
fn is_closure_call(fun_expr: &thir::Expr<'_>, tcx: TyCtxt<'_>) -> bool {
    match fun_expr.ty.kind() {
        rustc_type_ir::TyKind::FnDef(id, _) => {
            let Some(fn_name) = tcx.opt_item_name(*id).map(|name| name.to_ident_string()) else {
                return false;
            };

            if let Some(parent_trait_id) = tcx.trait_of_item(*id) {
                if tcx.is_fn_trait(parent_trait_id) {
                    matches!(fn_name.as_str(), "call" | "call_once")
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Unwrap `Scalar` type into `usize`
pub fn scalar_to_usize(scalar: Scalar) -> Option<usize> {
    match scalar {
        rustc_const_eval::interpret::Scalar::Int(scalar_int) => {
            scalar.to_bits(scalar_int.size()).ok().map(|value| value.try_into().unwrap())
        }
        rustc_const_eval::interpret::Scalar::Ptr(..) => todo!(),
    }
}

/// Evaluates constant generic arguement
pub fn evaluate_const_generic_arg<'tcx>(tcx: TyCtxt<'tcx>, arg: &GenericArg<'tcx>) -> Option<usize> {
    match normalize_alias_ty(tcx, *arg).unpack() {
        rustc_middle::ty::GenericArgKind::Lifetime(_) => panic!(),
        rustc_middle::ty::GenericArgKind::Type(t) => panic!("{t:?}"),
        rustc_middle::ty::GenericArgKind::Const(value) => eval_const(value, tcx),
    }
}

fn eval_const<'tcx>(value: Const<'tcx>, tcx: TyCtxt<'tcx>) -> Option<usize> {
    if let Some(c) = value.try_eval_bits(tcx, ParamEnv::empty()) {
        return c.try_into().ok();
    }

    match value.kind() {
        rustc_middle::ty::ConstKind::Param(_) => todo!(),
        rustc_middle::ty::ConstKind::Infer(_) => todo!(),
        rustc_middle::ty::ConstKind::Bound(..) => todo!(),
        rustc_middle::ty::ConstKind::Placeholder(_) => todo!(),
        rustc_middle::ty::ConstKind::Unevaluated(uneval) => {
            let mut evaluated = vec![];
            for subst in uneval.args {
                match subst.unpack() {
                    rustc_middle::ty::GenericArgKind::Const(c) => {
                        let c = Const::from_bits(
                            tcx,
                            evaluate_const_generic_arg(tcx, &subst).unwrap() as u128,
                            ParamEnv::empty().and(c.ty()),
                        );
                        evaluated.push(c.into())
                    }
                    _ => evaluated.push(subst),
                }
            }

            let c = UnevaluatedConst { def: uneval.def, args: tcx.mk_args(&evaluated) };
            tcx.const_eval_resolve_for_typeck(ParamEnv::reveal_all(), c, None)
                .ok()
                .and_then(|valtree| {
                    let valtree = valtree?;

                    resolve_valtree(&valtree)
                })
                .map(|value| value.try_into().unwrap())
        }
        rustc_middle::ty::ConstKind::Value(_) => {
            todo!()
        }
        rustc_middle::ty::ConstKind::Error(_) => todo!(),
        rustc_middle::ty::ConstKind::Expr(expr) => match expr {
            rustc_middle::ty::Expr::Binop(op, lhs, rhs) => {
                let lhs = eval_const(lhs, tcx);
                let rhs = eval_const(rhs, tcx);
                let v = lhs.zip(rhs).map(|(lhs, rhs)| match op {
                    rustc_middle::mir::BinOp::Add => lhs + rhs,
                    rustc_middle::mir::BinOp::Sub => lhs - rhs,
                    rustc_middle::mir::BinOp::Mul => lhs * rhs,
                    rustc_middle::mir::BinOp::Div => lhs / rhs,
                    rustc_middle::mir::BinOp::Rem => lhs % rhs,
                    rustc_middle::mir::BinOp::BitXor => todo!(),
                    rustc_middle::mir::BinOp::BitAnd => todo!(),
                    rustc_middle::mir::BinOp::BitOr => todo!(),
                    rustc_middle::mir::BinOp::Shl => todo!(),
                    rustc_middle::mir::BinOp::Shr => todo!(),
                    rustc_middle::mir::BinOp::Eq => todo!(),
                    rustc_middle::mir::BinOp::Lt => todo!(),
                    rustc_middle::mir::BinOp::Le => todo!(),
                    rustc_middle::mir::BinOp::Ne => todo!(),
                    rustc_middle::mir::BinOp::Ge => todo!(),
                    rustc_middle::mir::BinOp::Gt => todo!(),
                    rustc_middle::mir::BinOp::Offset => todo!(),
                    rustc_middle::mir::BinOp::AddUnchecked => todo!(),
                    rustc_middle::mir::BinOp::SubUnchecked => todo!(),
                    rustc_middle::mir::BinOp::MulUnchecked => todo!(),
                    rustc_middle::mir::BinOp::ShlUnchecked => todo!(),
                    rustc_middle::mir::BinOp::ShrUnchecked => todo!(),
                });
                assert!(v.is_some());
                v
            }
            rustc_middle::ty::Expr::UnOp(..) => todo!(),
            rustc_middle::ty::Expr::FunctionCall(c, l) => {
                let rustc_type_ir::TyKind::FnDef(id, _) = c.ty().kind() else { panic!() };

                // HACK: Find proper way without using this hack
                if tcx.item_name(*id).to_string() == "clog2" {
                    assert_eq!(l.len(), 1);
                    let s = l[0];
                    Some(clog2(s.eval_bits(tcx, ParamEnv::reveal_all()).try_into().unwrap()))
                } else if tcx.item_name(*id).to_string() == "max" {
                    assert_eq!(l.len(), 2);
                    let lhs = eval_const(l[0], tcx);
                    let rhs = eval_const(l[1], tcx);
                    let max_val = lhs.zip(rhs).map(|(lhs, rhs)| std::cmp::max(lhs, rhs));
                    assert!(max_val.is_some());
                    max_val
                } else {
                    panic!("{:#?}", tcx.item_name(*id).to_string())
                }
            }
            rustc_middle::ty::Expr::Cast(..) => todo!(),
        },
    }
}

fn resolve_valtree(valtree: &ValTree<'_>) -> Option<u128> {
    match valtree {
        ValTree::Leaf(x) => x.to_bits(x.size()).ok(),
        ValTree::Branch(x) => match x {
            [x] => resolve_valtree(x),
            _ => panic!("We do not know when this case happens"),
        },
    }
}

/// Returns the span of the given `DefId`
pub fn get_span(tcx: TyCtxt<'_>, id: DefId) -> Span {
    tcx.hir().span_if_local(id).unwrap()
}

/// Mapping from generic parameters to their bounds
#[derive(Debug, Clone)]
pub struct GenericMap<'tcx> {
    inner: HashMap<Ty<'tcx>, GenericBound<'tcx>>,
}

impl<'tcx> GenericMap<'tcx> {
    /// Get the bound of the given generic parameter
    pub fn get(&self, ty: Ty<'tcx>) -> Option<&GenericBound<'tcx>> {
        match ty.kind() {
            rustc_type_ir::TyKind::Param(_) => self.inner.get(&ty),
            _ => None,
        }
    }
}

/// Type bounded to a generic parameter
#[derive(Debug, Clone)]
pub enum GenericBound<'tcx> {
    /// Function-like type
    Function {
        /// Input
        input: Ty<'tcx>,

        /// Output
        output: Ty<'tcx>,
    },
    /// Constant
    Const(Const<'tcx>),
}

/// Returns a map from generic parameters to their bounds
pub fn get_generic_map<'tcx>(tcx: TyCtxt<'tcx>, instance: Instance<'tcx>) -> GenericMap<'tcx> {
    // XXX: Resolve `FnOnce` in top level module. Maybe there is a better solution for this.
    let predicates: GenericPredicates<'_> = tcx.explicit_predicates_of(instance.def_id());
    let instantiated_predicates: InstantiatedPredicates<'_> = predicates.instantiate_identity(tcx);

    let mut resolved_types = HashMap::new();
    let mut unresolved_ty_map = HashMap::new();

    for param in tcx.generics_of(instance.def_id()).params.iter() {
        let param = tcx.mk_param_from_def(param);

        match param.unpack() {
            rustc_middle::ty::GenericArgKind::Lifetime(_) => panic!(),
            rustc_middle::ty::GenericArgKind::Type(ty) => {
                unresolved_ty_map.insert(ty, (None, None));
            }
            rustc_middle::ty::GenericArgKind::Const(c) => {
                // TODO: substitute
                resolved_types.insert(c.ty(), GenericBound::Const(c));
            }
        }
    }

    for (predicate, _) in instantiated_predicates.iter() {
        match predicate.kind().skip_binder() {
            rustc_type_ir::ClauseKind::Trait(tr) => {
                if tcx.is_fn_trait(tr.trait_ref.def_id) {
                    let self_ty = tr.self_ty();

                    if let Some(v) = unresolved_ty_map.get_mut(&self_ty) {
                        v.0 = Some(tr.trait_ref.args.type_at(1));
                    };
                }
            }
            rustc_type_ir::ClauseKind::Projection(projection) => {
                let self_ty = projection.self_ty();
                let term = projection.term.ty();
                if let Some(v) = unresolved_ty_map.get_mut(&self_ty) {
                    v.1 = term;
                };
            }
            rustc_type_ir::ClauseKind::ConstEvaluatable(_) => {}
            unimpl => todo!("{unimpl:?}"),
        }
    }

    for (param_ty, v) in unresolved_ty_map.into_iter() {
        if let (Some(input), Some(output)) = v {
            let param = normalize_alias_ty(tcx, EarlyBinder::bind(param_ty).instantiate(tcx, instance.args));
            let input = normalize_alias_ty(tcx, EarlyBinder::bind(input).instantiate(tcx, instance.args));
            let output = normalize_alias_ty(tcx, EarlyBinder::bind(output).instantiate(tcx, instance.args));

            resolved_types.insert(param, GenericBound::Function { input, output });
        }
        // TODO:take care of other cases than function type
    }

    GenericMap { inner: resolved_types }
}

/// TODO: Documentation
#[derive(Debug, Clone, Copy)]
pub enum Id {
    /// TODO: Documentation
    Local(thir::LocalVarId),
    /// TODO: Documentation
    Upvar(HirId),
}

/// TODO: Documentation
#[allow(clippy::needless_lifetimes)]
pub fn get_hir_id<'tcx>(thir_body: &Thir<'tcx>, expr_id: ExprId) -> Id {
    let expr = &thir_body[expr_id];

    match &expr.kind {
        ExprKind::Scope { lint_level, .. } => match lint_level {
            thir::LintLevel::Inherited => todo!(),
            thir::LintLevel::Explicit(id) => Id::Upvar(*id),
        },
        ExprKind::Field { lhs, .. } => get_hir_id(thir_body, *lhs),
        ExprKind::VarRef { id, .. } => Id::Local(*id),
        ExprKind::UpvarRef { var_hir_id, .. } => Id::Local(*var_hir_id),
        ExprKind::Borrow { borrow_kind, arg } => {
            assert_eq!(*borrow_kind, BorrowKind::Shared);
            get_hir_id(thir_body, *arg)
        }
        _ => todo!("{expr:?}"),
    }
}

/// TODO: Documentation
pub fn get_variant_discriminator(tcx: TyCtxt<'_>, variant: &VariantDef) -> u32 {
    match variant.discr {
        rustc_middle::ty::VariantDiscr::Explicit(id) => match tcx.const_eval_poly(id).unwrap() {
            ConstValue::Scalar(scalar) => match scalar {
                rustc_const_eval::interpret::Scalar::Int(scalar_int) => {
                    scalar_int.to_bits(scalar_int.size()).unwrap().try_into().unwrap()
                }
                rustc_const_eval::interpret::Scalar::Ptr(..) => todo!(),
            },
            _ => todo!("{variant:?}"),
        },
        rustc_middle::ty::VariantDiscr::Relative(i) => i,
    }
}

/// Get the string from the Thir id.
pub fn get_string_from_thir_id(body: &Thir<'_>, id: ExprId) -> (String, Span) {
    let ExprKind::Deref { arg, .. } = &body[skip_exprs(body, id)].kind else { panic!() };

    let ExprKind::Literal { lit, .. } = &body[skip_exprs(body, *arg)].kind else { panic!() };

    let LitKind::Str(symbol, str_style) = lit.node else { panic!() };

    assert_eq!(str_style, StrStyle::Cooked);
    let (fstring, span) = (symbol.to_ident_string(), lit.span);
    (fstring, span)
}
