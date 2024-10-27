//! Pure Function.

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Formatter;

use itertools::Itertools;
use rustc_hir::def::DefKind;
use rustc_middle::thir::{self, ExprKind, Thir};
use rustc_middle::ty::{EarlyBinder, FnSig, GenericArgKind, GenericArgsRef, Instance, ParamEnv, TyCtxt};
use rustc_span::Span;

use super::*;
use crate::utils::*;

/// Function Id
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(usize);

impl FunctionId {
    /// Allocates expr to the table and returns the id
    #[allow(clippy::needless_lifetimes)]
    pub fn alloc_function<'tcx>(function: FunctionBuilder<'tcx>) -> Self {
        let function = unsafe { std::mem::transmute(function) };
        FUNCTION_TABLE.with(|table| table.push(function))
    }

    /// Returns expr corresponding to given id
    pub fn into_function<'tcx>(self) -> FunctionBuilder<'tcx> {
        let function = FUNCTION_TABLE.with(|table| table.get(self));
        unsafe { std::mem::transmute(function) }
    }
}

/// Function Table
#[derive(Default)]
pub struct FunctionTable<'tcx> {
    inner: RefCell<Vec<FunctionBuilder<'tcx>>>,
}

impl<'tcx> std::fmt::Debug for FunctionTable<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionTable").field("inner", &self.inner).finish()
    }
}
thread_local! {
    /// Expr Table
    pub(crate) static FUNCTION_TABLE: FunctionTable<'static> = FunctionTable::default();
}

impl FunctionTable<'static> {
    /// Inserts expr into table.
    fn get(&self, id: FunctionId) -> FunctionBuilder<'static> {
        self.inner.borrow().get(id.0).expect("does not have element!").clone()
    }

    /// Returns expr from table by using id.
    fn push(&self, function: FunctionBuilder<'static>) -> FunctionId {
        let id = self.inner.borrow().len();
        self.inner.borrow_mut().push(function);
        FunctionId(id)
    }
}

/// Function Arguement
#[derive(Debug, Clone)]
pub enum PureValue<'tcx> {
    /// Expr
    Expr(ExprId),

    /// Function
    ///
    /// This contains all the necessary information to build the function
    Function(FunctionBuilder<'tcx>),

    /// Misc
    Misc,
}

impl<'tcx> PureValue<'tcx> {
    /// Get the expression id
    pub fn expr(&self) -> Option<ExprId> {
        match self {
            PureValue::Expr(expr) => Some(*expr),
            PureValue::Function(_) => None,
            PureValue::Misc => None,
        }
    }

    /// Get the function builder
    pub fn function(&self) -> Option<FunctionBuilder<'tcx>> {
        match self {
            PureValue::Expr(_) => None,
            PureValue::Function(f) => Some(f.clone()),
            PureValue::Misc => None,
        }
    }
}

/// Normal Function
#[derive(Debug, Clone)]
pub struct Fn<'tcx> {
    /// Function instance
    instance: Instance<'tcx>,

    /// Upvars
    pub upvars: Option<Vec<(Id, PureValue<'tcx>)>>,

    /// Function body
    thir_body: &'tcx rustc_data_structures::steal::Steal<Thir<'tcx>>,
}

impl<'tcx> Fn<'tcx> {
    fn substs(&self) -> GenericArgsRef<'tcx> {
        self.instance.args
    }
}

/// Function AST
#[derive(Debug, Clone)]
pub enum Function<'tcx> {
    /// Function
    Fn(Fn<'tcx>),

    /// Constructor
    Ctor {
        /// Function instance
        instance: Instance<'tcx>,
    },

    /// Magic Function
    Magic {
        /// Expr Magic
        magic: ExprMagic,

        /// Function instance
        instance: Instance<'tcx>,
    },
}

impl<'tcx> Function<'tcx> {
    /// TODO: Documentation
    pub fn is_closure(&self) -> bool {
        match self {
            Function::Fn(f) => f.upvars.is_some(),
            Function::Ctor { .. } => false,
            Function::Magic { .. } => false,
        }
    }
}

/// Function builder
///
/// This struct builds `Expr` ast from `Thir`.
#[derive(Clone)]
pub struct FunctionBuilder<'tcx> {
    /// Ast
    pub ast: Function<'tcx>,

    /// span of the function
    pub span: Span,

    // ** Preprocess Results **//
    /// Pattern Bindings
    pat_bindings: Vec<PatBinding<'tcx>>,

    /// Explicit Returns
    explicit_returns: Vec<Return<'tcx>>,

    /// System Tasks
    pub system_tasks: Vec<SystemTaskInfo<'tcx>>,
}

impl std::fmt::Debug for FunctionBuilder<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionBuilder").field("ast", &self.ast).field("span", &self.span).finish()
    }
}

impl<'tcx> FunctionBuilder<'tcx> {
    /// Create a new function builder for a crate-local function.
    pub fn new_local(instance: Instance<'tcx>, tcx: TyCtxt<'tcx>) -> Self {
        assert!(instance.def_id().is_local());
        let attr = get_hazardflow_attribute(tcx, tcx.local_def_id_to_hir_id(instance.def_id().expect_local()));

        if let Some(attr) = attr {
            let HazardFlowAttr::ExprMagic(magic) = attr else { panic!() };
            FunctionBuilder::new_magic(instance, magic, tcx)
        } else if tcx.is_constructor(instance.def_id()) {
            FunctionBuilder::new_ctor(instance, tcx)
        } else {
            FunctionBuilder::new_fn(instance, None, tcx)
        }
    }

    /// Create a new function builder
    pub fn new_closure(instance: Instance<'tcx>, upvars: Vec<(Id, PureValue<'tcx>)>, tcx: TyCtxt<'tcx>) -> Self {
        Self::new_fn(instance, Some(upvars), tcx)
    }

    /// Create a new function builder
    fn new_magic(instance: Instance<'tcx>, magic: ExprMagic, tcx: TyCtxt<'tcx>) -> Self {
        Self {
            ast: Function::Magic { instance, magic },
            span: get_span(tcx, instance.def_id()),
            pat_bindings: vec![],
            explicit_returns: vec![],
            system_tasks: vec![],
        }
    }

    /// Create a new function builder
    fn new_ctor(instance: Instance<'tcx>, tcx: TyCtxt<'tcx>) -> Self {
        Self {
            ast: Function::Ctor { instance },
            span: get_span(tcx, instance.def_id()),
            pat_bindings: vec![],
            explicit_returns: vec![],
            system_tasks: vec![],
        }
    }

    /// Create a new function builder
    fn new_fn(instance: Instance<'tcx>, upvars: Option<Vec<(Id, PureValue<'tcx>)>>, tcx: TyCtxt<'tcx>) -> Self {
        let f = Function::Fn(Fn { instance, upvars, thir_body: thir_body(tcx, instance.def_id().expect_local()) });

        let span = get_span(tcx, instance.def_id());

        let mut new_builder =
            Self { ast: f, span, pat_bindings: vec![], explicit_returns: vec![], system_tasks: vec![] };

        if let Function::Fn(_) = &new_builder.ast {
            new_builder.preprocess(tcx);
        }

        new_builder
    }

    /// TODO: Documentation
    pub fn expect_fn(&self) -> &Fn<'tcx> {
        match &self.ast {
            Function::Ctor { .. } => panic!(),
            Function::Magic { .. } => panic!(),
            Function::Fn(f) => f,
        }
    }

    /// builds the function with given args as leaf nodes of ast
    pub fn build(
        &self,
        tcx: TyCtxt<'tcx>,
        args: Vec<PureValue<'tcx>>,
        fsm_cache: &mut FsmCache,
    ) -> (ExprId, Vec<SystemTask>) {
        match &self.ast {
            Function::Ctor { instance } => (self.build_ctor(tcx, *instance, fsm_cache, args), vec![]),
            Function::Fn { .. } => self.build_fn(tcx, fsm_cache, args),
            Function::Magic { instance, magic } => {
                (self.build_magic(tcx, *instance, magic.clone(), fsm_cache, args), vec![])
            }
        }
    }

    fn build_ctor(
        &self,
        tcx: TyCtxt<'tcx>,
        instance: Instance<'tcx>,
        fsm_cache: &mut FsmCache,
        args: Vec<PureValue<'tcx>>,
    ) -> ExprId {
        let instance_id = instance.def_id();
        let span = get_span(tcx, instance_id);
        let DefKind::Ctor(of, kind) = tcx.def_kind(instance_id) else { panic!() };

        match (of, kind) {
            (rustc_hir::def::CtorOf::Struct, rustc_hir::def::CtorKind::Fn) => {
                Expr::tuple(args.into_iter().map(|x| x.expr().unwrap()).collect(), span).alloc_with_fsm_cache(fsm_cache)
            }
            (rustc_hir::def::CtorOf::Struct, rustc_hir::def::CtorKind::Const) => todo!(),
            (rustc_hir::def::CtorOf::Variant, rustc_hir::def::CtorKind::Fn) => {
                let enum_def = tcx.parent(tcx.parent(instance_id));
                let enum_ty = tcx.type_of(enum_def).no_bound_vars().unwrap();

                if let rustc_type_ir::TyKind::Adt(def, substs) = enum_ty.kind() {
                    assert!(substs.is_empty());

                    let discriminant_len = clog2(def.variants().len());

                    let mut inner = vec![(
                        None,
                        Expr::X {
                            // XXX: temp value
                            typ: PortDecls::unsigned_bits(discriminant_len),
                            span,
                        }
                        .alloc_with_fsm_cache(fsm_cache),
                    )];

                    for (variant_idx, variant) in def.variants().iter().enumerate() {
                        let expr = if variant.ctor.is_some_and(|(_, ctor)| ctor == instance_id) {
                            inner[0] = (
                                Some("discriminant".to_string()),
                                Expr::unsigned_bits(discriminant_len, variant_idx, span)
                                    .alloc_with_fsm_cache(fsm_cache),
                            );
                            Expr::Struct {
                                inner: variant
                                                        .fields
                                                        .iter()
                                                        // XXX: this should not be
                                                        // clone
                                                        .zip_eq(args.clone())
                                                        .map(|(field, arg)| {
                                                            (
                                                                Some(field.ident(tcx).to_string()),
                                                                arg.expr().unwrap(),
                                                            )
                                                        })
                                                        .collect(),
                                span,
                            }
                            .alloc_with_fsm_cache(fsm_cache)
                        } else {
                            Expr::Struct {
                                inner: variant
                                    .fields
                                    .iter()
                                    .map(|field| {
                                        let ty = tcx.type_of(field.did).instantiate(tcx, substs);
                                        (
                                            Some(field.ident(tcx).to_string()),
                                            Expr::X { typ: PortDecls::from_ty(ty, tcx).unwrap(), span }
                                                .alloc_with_fsm_cache(fsm_cache),
                                        )
                                    })
                                    .collect(),
                                span,
                            }
                            .alloc_with_fsm_cache(fsm_cache)
                        };
                        inner.push((Some(variant.ident(tcx).to_string()), expr))
                    }

                    let expr = Expr::Struct { inner, span }.alloc_with_fsm_cache(fsm_cache);

                    let t_expected = PortDecls::from_ty(enum_ty, tcx).unwrap();
                    let t_generated = expr.into_expr().port_decls();

                    assert_eq!(t_expected, t_generated);
                    expr
                } else {
                    unreachable!()
                }
            }
            (rustc_hir::def::CtorOf::Variant, rustc_hir::def::CtorKind::Const) => todo!(),
        }
    }

    fn build_fn(
        &self,
        tcx: TyCtxt<'tcx>,
        fsm_cache: &mut FsmCache,
        args: Vec<PureValue<'tcx>>,
    ) -> (ExprId, Vec<SystemTask>) {
        let mut thir_cache = ThirCache::default();

        let mut displays = self
            .system_tasks
            .iter()
            .flat_map(|display| self.build_system_task(display, tcx, &mut thir_cache, fsm_cache, &args))
            .collect::<Vec<_>>();

        let return_expr_id = (self.expect_fn().thir_body.borrow().exprs.len() - 1).into();
        let (default_return, mut displays_inner) =
            self.build_expr(tcx, return_expr_id, &mut thir_cache, fsm_cache, &args);

        displays.append(&mut displays_inner);

        log::debug!("Finished Building function: {:?} returns: {:#?}", self.span, self.explicit_returns);

        let (explicit_returns, displays_from_returns): (Vec<_>, Vec<_>) = self
            .explicit_returns
            .iter()
            .map(|ret| self.build_return(ret, tcx, &mut thir_cache, fsm_cache, &args))
            .unzip();
        displays.append(&mut displays_from_returns.concat());

        let return_selected = if explicit_returns.is_empty() {
            default_return
        } else {
            Expr::Cond { cond_expr_pair: explicit_returns, default: default_return, span: self.span }
                .alloc_with_fsm_cache(fsm_cache)
        };

        (return_selected, displays)
    }

    fn build_magic(
        &self,
        tcx: TyCtxt<'tcx>,
        instance: Instance<'tcx>,
        magic: ExprMagic,
        fsm_cache: &mut FsmCache,
        args: Vec<PureValue<'tcx>>,
    ) -> ExprId {
        match magic {
            ExprMagic::ArrayMagic(magic) => {
                self.build_array_magic_fun(tcx, magic, instance, &args, self.span, fsm_cache)
            }
            ExprMagic::IntMagic(magic) => self.build_int_magic_fun(tcx, magic, &args, instance, self.span, fsm_cache),
            ExprMagic::AdtMagic(magic) => self.build_adt_magic_fun(magic, instance, &args, self.span, fsm_cache),
            ExprMagic::X => {
                let typ = match instance.args.first().unwrap().unpack() {
                    GenericArgKind::Lifetime(_) => todo!(),
                    GenericArgKind::Type(ty) => ty,
                    GenericArgKind::Const(_) => todo!(),
                };

                let typ = PortDecls::from_ty(typ, tcx).unwrap();

                Expr::X { typ, span: self.span }.alloc_with_fsm_cache(fsm_cache)
            }
        }
    }

    fn build_adt_magic_fun(
        &self,
        magic: AdtMagic,
        _monomorphized: Instance<'tcx>,
        build_args: &[PureValue<'tcx>],
        span: Span,
        fsm_cache: &mut FsmCache,
    ) -> ExprId {
        match magic {
            AdtMagic::EnumEq => self.build_adt_eq(build_args, span, fsm_cache),
            AdtMagic::EnumNe => Expr::Not { inner: self.build_adt_eq(build_args, span, fsm_cache), span }
                .alloc_with_fsm_cache(fsm_cache),
        }
    }

    fn build_adt_eq(&self, build_args: &[PureValue<'tcx>], span: Span, fsm_cache: &mut FsmCache) -> ExprId {
        let lhs = build_args[0].expr().unwrap();
        let rhs = build_args[1].expr().unwrap();
        let discriminant_eq = Expr::BinaryOp {
            op: BinaryOp::EqArithmetic,
            lhs: lhs.member(0, span).alloc_with_fsm_cache(fsm_cache),
            rhs: rhs.member(0, span).alloc_with_fsm_cache(fsm_cache),
            span,
        }
        .alloc_with_fsm_cache(fsm_cache);
        match lhs.into_expr().port_decls() {
            PortDecls::Struct(inner) => {
                let mut variant_eqs = vec![];

                for variant_idx in 1..inner.len() {
                    let lhs = Expr::Member { inner: lhs, index: variant_idx, span };
                    // NOTE: skip if the variant is empty
                    if lhs.width() == 0 {
                        continue;
                    }
                    let rhs = Expr::Member { inner: rhs, index: variant_idx, span };
                    variant_eqs.push(
                        Expr::BinaryOp {
                            op: BinaryOp::EqArithmetic,
                            lhs: lhs.alloc_with_fsm_cache(fsm_cache),
                            rhs: rhs.alloc_with_fsm_cache(fsm_cache),
                            span,
                        }
                        .alloc_with_fsm_cache(fsm_cache),
                    );
                }

                if variant_eqs.is_empty() {
                    return discriminant_eq;
                }

                let (first, rest) = variant_eqs.split_first().unwrap();

                // TODO: use reduction operator?
                let variants_eq = rest.iter().fold(*first, |acc, elt| {
                    Expr::BinaryOp { op: BinaryOp::Or, lhs: acc, rhs: *elt, span }.alloc_with_fsm_cache(fsm_cache)
                });

                Expr::BinaryOp { op: BinaryOp::And, lhs: discriminant_eq, rhs: variants_eq, span }
                    .alloc_with_fsm_cache(fsm_cache)
            }
            PortDecls::Bits(_) => todo!(),
        }
    }

    fn build_int_magic_fun(
        &self,
        tcx: TyCtxt<'tcx>,
        magic: IntMagic,
        build_args: &[PureValue<'tcx>],
        monomorphized: Instance<'tcx>,
        span: Span,
        fsm_cache: &mut FsmCache,
    ) -> ExprId {
        match magic {
            IntMagic::Convert => {
                assert_eq!(build_args.len(), 1);
                assert!(monomorphized.args.len() <= 1);
                let from_expr = build_args[0].expr().unwrap();

                let ty = monomorphized.ty(tcx, ParamEnv::empty()).fn_sig(tcx).output().skip_binder();

                let PortDecls::Bits(to) = PortDecls::from_ty(ty, tcx).unwrap() else { panic!() };
                Expr::Cast { from: from_expr, to, span }.alloc_with_fsm_cache(fsm_cache)
            }
            IntMagic::Not => Expr::Not { inner: build_args[0].expr().unwrap(), span }.alloc_with_fsm_cache(fsm_cache),
            magic => {
                let op = magic.bin_op();

                assert_eq!(build_args.len(), 2);

                Expr::BinaryOp { op, lhs: build_args[0].expr().unwrap(), rhs: build_args[1].expr().unwrap(), span }
                    .alloc_with_fsm_cache(fsm_cache)
            }
        }
    }

    fn build_array_magic_fun(
        &self,
        tcx: TyCtxt<'tcx>,
        magic: ArrayMagic,
        monomorphized: Instance<'tcx>,
        build_args: &[PureValue<'tcx>],
        span: Span,
        fsm_cache: &mut FsmCache,
    ) -> ExprId {
        match magic {
            ArrayMagic::Range => {
                let n = evaluate_const_generic_arg(tcx, monomorphized.args.first().unwrap()).unwrap();
                let elt_len = clog2(n);
                let inner =
                    (0..n).map(|i| Expr::unsigned_bits(elt_len, i, span).alloc_with_fsm_cache(fsm_cache)).collect();
                Expr::ConcatArray { inner, elt_typ: PortDecls::unsigned_bits(elt_len), span }
                    .alloc_with_fsm_cache(fsm_cache)
            }
            ArrayMagic::Zip => {
                assert_eq!(build_args.len(), 2);
                let n: usize = evaluate_const_generic_arg(tcx, monomorphized.args.get(1).unwrap()).unwrap();
                let (this, other) = (build_args[0].expr().unwrap(), build_args[1].expr().unwrap());
                let typ_inner = vec![this.into_expr().port_decls().divide(n), other.into_expr().port_decls().divide(n)];

                Expr::Zip { inner: vec![this, other], typ_inner, span }.alloc_with_fsm_cache(fsm_cache)
            }
            ArrayMagic::Map => {
                assert_eq!(build_args.len(), 2);

                let function = build_args[1].function().unwrap();

                let inputs = function.sig(tcx).inputs();
                assert_eq!(inputs.len(), 1);

                let n: usize = evaluate_const_generic_arg(tcx, monomorphized.args.get(1).unwrap()).unwrap();

                let typ_elt = PortDecls::from_ty(inputs[0], tcx).unwrap();

                Expr::Map {
                    inner: build_args[0].expr().unwrap(),
                    typ_elt,
                    func_ret_typ: PortDecls::from_ty(function.sig(tcx).output(), tcx).unwrap(),
                    func: FunctionId::alloc_function(function),
                    span,
                    len: n,
                }
                .alloc_with_fsm_cache(fsm_cache)
            }
            ArrayMagic::Chunk => {
                let n = evaluate_const_generic_arg(tcx, monomorphized.args.get(2).unwrap()).unwrap();

                Expr::Chunk { inner: build_args[0].expr().unwrap(), chunk_size: n, span }
                    .alloc_with_fsm_cache(fsm_cache)
            }
            ArrayMagic::Concat => {
                let inner = build_args[0].expr().unwrap();
                let n = evaluate_const_generic_arg(tcx, monomorphized.args.get(1).unwrap()).unwrap();
                let m = evaluate_const_generic_arg(tcx, monomorphized.args.get(2).unwrap()).unwrap();
                let typ_elt = inner.into_expr().port_decls().divide(n).divide(m);

                Expr::Concat { inner, typ_elt, span }.alloc_with_fsm_cache(fsm_cache)
            }
            ArrayMagic::Resize => {
                assert_eq!(build_args.len(), 1);
                assert_eq!(monomorphized.args.len(), 3);
                let from_expr = build_args[0].expr().unwrap();
                let from_width = evaluate_const_generic_arg(tcx, monomorphized.args.get(1).unwrap()).unwrap();
                let to_width = evaluate_const_generic_arg(tcx, monomorphized.args.get(2).unwrap()).unwrap();

                Expr::resize(from_expr, from_width, to_width, fsm_cache, span)
            }
            ArrayMagic::Set => {
                let inner = build_args[0].expr().unwrap();
                let idx = build_args[1].expr().unwrap();
                let n = evaluate_const_generic_arg(tcx, monomorphized.args.get(1).unwrap()).unwrap();
                let index = Expr::cast_bits(idx, PortDecls::unsigned_bits(clog2(n)), fsm_cache, span);
                Expr::Set { inner, index, elt: build_args[2].expr().unwrap(), span }.alloc_with_fsm_cache(fsm_cache)
            }
            ArrayMagic::ClipConst => {
                let GenericArgKind::Type(typ_elt) = monomorphized.args.first().unwrap().unpack() else { panic!() };
                log::debug!("clip_const {monomorphized:?}");
                let typ_elt = PortDecls::from_ty(typ_elt, tcx).unwrap();
                let from = build_args[1].expr().unwrap();
                let size = evaluate_const_generic_arg(tcx, monomorphized.args.get(2).unwrap()).unwrap();
                Expr::Clip { inner: build_args[0].expr().unwrap(), typ_elt, from, size, span }
                    .alloc_with_fsm_cache(fsm_cache)
            }
            ArrayMagic::Fold => {
                let GenericArgKind::Type(typ_elt) = monomorphized.args.first().unwrap().unpack() else { panic!() };
                log::debug!("fold {monomorphized:?}");
                let typ_elt = PortDecls::from_ty(typ_elt, tcx).unwrap();
                let func = build_args[2].function().unwrap();

                Expr::Fold {
                    inner: build_args[0].expr().unwrap(),
                    typ_elt,
                    func: FunctionId::alloc_function(func),
                    init: build_args[1].expr().unwrap(),
                    span,
                }
                .alloc_with_fsm_cache(fsm_cache)
            }
            ArrayMagic::Append => {
                let GenericArgKind::Type(typ_elt) = monomorphized.args.first().unwrap().unpack() else { panic!() };
                let typ_elt = PortDecls::from_ty(typ_elt, tcx).unwrap();
                Expr::Append { lhs: build_args[0].expr().unwrap(), rhs: build_args[1].expr().unwrap(), typ_elt, span }
                    .alloc_with_fsm_cache(fsm_cache)
            }
            ArrayMagic::Array => todo!(),
            ArrayMagic::From => {
                let elt_ty = match monomorphized.args.first().unwrap().unpack() {
                    rustc_middle::ty::GenericArgKind::Lifetime(_) => todo!(),
                    rustc_middle::ty::GenericArgKind::Type(ty) => PortDecls::from_ty(ty, tcx).unwrap(),
                    rustc_middle::ty::GenericArgKind::Const(_) => todo!(),
                };
                let len = evaluate_const_generic_arg(tcx, monomorphized.args.get(1).unwrap()).unwrap();

                let inner = build_args[0].expr().unwrap();

                assert_eq!(inner.into_expr().width(), elt_ty.multiple(len).width());

                match (inner.into_expr().port_decls(), elt_ty) {
                    (PortDecls::Struct(_), PortDecls::Struct(_)) => inner,
                    (PortDecls::Bits(_), PortDecls::Bits(_)) => inner,
                    _ => panic!(),
                }
            }
            ArrayMagic::Index => {
                let inner = build_args[0].expr().unwrap();
                let idx = build_args[1].expr().unwrap();

                let inner_len = match monomorphized.args.len() {
                    2 => evaluate_const_generic_arg(tcx, monomorphized.args.get(1).unwrap()).unwrap(),
                    3 => evaluate_const_generic_arg(tcx, monomorphized.args.get(2).unwrap()).unwrap(),
                    _ => panic!(),
                };

                let GenericArgKind::Type(ty) = monomorphized.args.first().unwrap().unpack() else { panic!() };

                let index = Expr::resize(idx, idx.into_expr().width(), clog2(inner_len), fsm_cache, span);

                Expr::Get { inner, typ_elt: PortDecls::from_ty(ty, tcx).unwrap(), index, span }
                    .alloc_with_fsm_cache(fsm_cache)
            }
            ArrayMagic::BitOr => Expr::BinaryOp {
                op: BinaryOp::Or,
                lhs: build_args[0].expr().unwrap(),
                rhs: build_args[1].expr().unwrap(),
                span,
            }
            .alloc_with_fsm_cache(fsm_cache),
            ArrayMagic::BitAnd => Expr::BinaryOp {
                op: BinaryOp::And,
                lhs: build_args[0].expr().unwrap(),
                rhs: build_args[1].expr().unwrap(),
                span,
            }
            .alloc_with_fsm_cache(fsm_cache),
            ArrayMagic::Repeat => {
                assert_eq!(monomorphized.args.len(), 2, "{:?}", monomorphized.args);

                let count = evaluate_const_generic_arg(tcx, monomorphized.args.get(1).unwrap()).unwrap();

                let inner = build_args[0].expr().unwrap();

                Expr::Repeat { inner, count, span }.alloc_with_fsm_cache(fsm_cache)
            }
            ArrayMagic::Eq => Expr::BinaryOp {
                op: BinaryOp::EqArithmetic,
                lhs: build_args[0].expr().unwrap(),
                rhs: build_args[1].expr().unwrap(),
                span,
            }
            .alloc_with_fsm_cache(fsm_cache),
            ArrayMagic::Ne => Expr::BinaryOp {
                op: BinaryOp::NeArithmetic,
                lhs: build_args[0].expr().unwrap(),
                rhs: build_args[1].expr().unwrap(),
                span,
            }
            .alloc_with_fsm_cache(fsm_cache),
            ArrayMagic::BitXor => Expr::BinaryOp {
                op: BinaryOp::Xor,
                lhs: build_args[0].expr().unwrap(),
                rhs: build_args[1].expr().unwrap(),
                span,
            }
            .alloc_with_fsm_cache(fsm_cache),
            ArrayMagic::SetRange => {
                let typ_elt = match monomorphized.args.first().unwrap().unpack() {
                    rustc_middle::ty::GenericArgKind::Lifetime(_) => todo!(),
                    rustc_middle::ty::GenericArgKind::Type(ty) => PortDecls::from_ty(ty, tcx).unwrap(),
                    rustc_middle::ty::GenericArgKind::Const(_) => todo!(),
                };
                Expr::SetRange {
                    inner: build_args[0].expr().unwrap(),
                    typ_elt,
                    index: build_args[1].expr().unwrap(),
                    elts: build_args[2].expr().unwrap(),
                    span,
                }
                .alloc_with_fsm_cache(fsm_cache)
            }
        }
    }

    fn build_return(
        &self,
        ret: &Return<'tcx>,
        tcx: TyCtxt<'tcx>,
        thir_cache: &mut ThirCache,
        fsm_cache: &mut FsmCache,
        args: &[PureValue<'tcx>],
        // acc: ExprId,
    ) -> ((ExprId, ExprId), Vec<SystemTask>) {
        let path_cond = &ret.path_cond;
        let (cond, displays_in_path) = self.build_path_cond(path_cond, tcx, thir_cache, fsm_cache, args);
        let cond = cond.unwrap_or(Expr::unsigned_bits(1, 1, self.span).alloc_with_fsm_cache(fsm_cache));

        let (value, displays_in_value) = self.build_expr(tcx, ret.value, thir_cache, fsm_cache, args);
        ((cond, value), [displays_in_path, displays_in_value].concat())
    }

    fn build_system_task(
        &self,
        task: &SystemTaskInfo<'tcx>,
        tcx: TyCtxt<'tcx>,
        thir_cache: &mut ThirCache,
        fsm_cache: &mut FsmCache,
        args: &[PureValue<'tcx>],
    ) -> Vec<SystemTask> {
        let mut result = vec![];

        let path_cond = &task.path_cond;
        let (path_cond, mut tasks_in_path) = self.build_path_cond(path_cond, tcx, thir_cache, fsm_cache, args);
        result.append(&mut tasks_in_path);

        let (kind, mut tasks) = match task.kind {
            SystemTaskInfoKind::Display => (SystemTaskKind::Display, vec![]),
            SystemTaskInfoKind::Assert { cond } => {
                let (cond, mut tasks) = self.build_expr(tcx, cond, thir_cache, fsm_cache, args);

                for task in tasks.iter_mut() {
                    let zipped = match (task.path_cond, path_cond) {
                        (None, None) => None,
                        (None, Some(cond)) | (Some(cond), None) => Some(cond),
                        (Some(l), Some(r)) => Some(
                            Expr::BinaryOp { op: BinaryOp::And, lhs: l, rhs: r, span: self.span }
                                .alloc_with_fsm_cache(fsm_cache),
                        ),
                    };
                    // Concat path cond to the display
                    task.path_cond = zipped;
                }

                (SystemTaskKind::Assert { cond }, tasks)
            }
        };
        result.append(&mut tasks);

        let (args, mut tasks) = self.build_expr(tcx, task.arg, thir_cache, fsm_cache, args);

        for task in tasks.iter_mut() {
            let zipped = match (task.path_cond, path_cond) {
                (None, None) => None,
                (None, Some(cond)) | (Some(cond), None) => Some(cond),
                (Some(l), Some(r)) => Some(
                    Expr::BinaryOp { op: BinaryOp::And, lhs: l, rhs: r, span: self.span }
                        .alloc_with_fsm_cache(fsm_cache),
                ),
            };
            // Concat path cond to the display
            task.path_cond = zipped;
        }
        result.append(&mut tasks);

        let PortDecls::Struct(inner) = args.into_expr().port_decls() else { panic!() };

        let task = SystemTask {
            kind,
            fstring: task.fstring.clone(),
            path_cond,
            args: inner
                .iter()
                .enumerate()
                .map(|(i, _)| args.member(i, args.into_expr().span()).alloc_with_fsm_cache(fsm_cache))
                .collect(),
            span: task.span,
        };
        result.push(task);

        // tasks.into_iter().chain(tasks_in_path).chain(std::iter::once(task)).collect()
        result
    }

    fn build_path_cond(
        &self,
        path_cond: &[Condition<'tcx>],
        tcx: TyCtxt<'tcx>,
        thir_cache: &mut ThirCache,
        fsm_cache: &mut FsmCache,
        args: &[PureValue<'tcx>],
    ) -> (Option<ExprId>, Vec<SystemTask>) {
        let mut condition_exprs = vec![];
        let mut displays = vec![];

        for condition in path_cond.iter() {
            let (cond_expr, mut displays_in_path) =
                self.build_condition_expr(condition, tcx, thir_cache, fsm_cache, args);
            displays.append(&mut displays_in_path);
            assert_eq!(cond_expr.into_expr().width(), 1);
            condition_exprs.push(cond_expr);
        }
        // TODO: use reduction operator
        (
            condition_exprs.into_iter().reduce(|l, r| {
                Expr::BinaryOp { op: BinaryOp::And, lhs: l, rhs: r, span: self.span }.alloc_with_fsm_cache(fsm_cache)
            }),
            displays,
        )
    }

    fn build_condition_expr(
        &self,
        condition: &Condition<'tcx>,
        tcx: TyCtxt<'tcx>,
        thir_cache: &mut ThirCache,
        fsm_cache: &mut FsmCache,
        args: &[PureValue<'tcx>],
    ) -> (ExprId, Vec<SystemTask>) {
        match condition {
            Condition::Expr(cond_expr_id) => self.build_expr(tcx, *cond_expr_id, thir_cache, fsm_cache, args),
            Condition::Matches(pat, cond_expr_id) => {
                let (match_arg, displays) = self.build_expr(tcx, *cond_expr_id, thir_cache, fsm_cache, args);
                (
                    gen_match_cond(tcx, pat.as_ref(), match_arg, fsm_cache)
                        .unwrap_or(Expr::unsigned_bits(1, 1, self.span).alloc_with_fsm_cache(fsm_cache)),
                    displays,
                )
            }
            Condition::Not(cond) => {
                let (inner, displays) = self.build_condition_expr(cond, tcx, thir_cache, fsm_cache, args);
                (Expr::Not { inner, span: self.span }.alloc_with_fsm_cache(fsm_cache), displays)
            }
        }
    }

    /// Returns signature of the function
    pub fn sig(&self, tcx: TyCtxt<'tcx>) -> FnSig<'tcx> {
        match &self.ast {
            Function::Fn(f) => match f.thir_body.borrow().body_type {
                thir::BodyTy::Fn(fn_sig) => {
                    normalize_alias_ty(tcx, EarlyBinder::bind(fn_sig).instantiate(tcx, f.substs()))
                }
                thir::BodyTy::Const(_) => panic!(),
            },
            Function::Magic { instance, .. } | Function::Ctor { instance } => {
                instance.ty(tcx, ParamEnv::empty()).fn_sig(tcx).no_bound_vars().unwrap()
            }
        }
    }

    fn build_expr(
        &self,
        tcx: TyCtxt<'tcx>,
        expr_id: thir::ExprId,
        thir_cache: &mut ThirCache,
        fsm_cache: &mut FsmCache,
        args: &[PureValue<'tcx>],
    ) -> (ExprId, Vec<SystemTask>) {
        ExprBuilder {
            tcx,
            expr_id,
            thir_body: self.expect_fn().thir_body,
            thir_cache,
            fsm_cache,
            substs: self.expect_fn().substs(),
            args,
            upvars: self.expect_fn().upvars.as_deref(),
            pat_bindings: &self.pat_bindings,
            path_ctx: Default::default(),
            tasks_inner: vec![],
        }
        .build()
    }

    fn preprocess(&mut self, tcx: TyCtxt<'tcx>) {
        let mut preprocess_ctx = PreprocessCtx::new(tcx);
        if let Some(expr) = self.expect_fn().thir_body.borrow().exprs.iter().last() {
            self.preprocess_expr(expr, &mut preprocess_ctx)
        } else {
            panic!()
        }
        assert!(preprocess_ctx.is_clean());
    }

    fn preprocess_block(&mut self, block: &thir::Block, ctx: &mut PreprocessCtx<'tcx>) {
        // synthesize!("preprocessing block: {:#?}", block);
        for stmt in block.stmts.iter() {
            self.preprocess_stmt(&self.expect_fn().thir_body.borrow()[*stmt], ctx)
        }

        if let Some(expr) = block.expr.as_ref() {
            self.preprocess_expr(&self.expect_fn().thir_body.borrow()[*expr], ctx)
        }
    }

    fn preprocess_expr(&mut self, expr: &thir::Expr<'tcx>, ctx: &mut PreprocessCtx<'tcx>) {
        let body = &self.expect_fn().thir_body.borrow();
        match &expr.kind {
            ExprKind::Scope { value, .. } => self.preprocess_expr(&body[*value], ctx),
            ExprKind::Box { .. } => panic!(),
            ExprKind::If { cond, then, else_opt, .. } => {
                self.preprocess_expr(&body[*cond], ctx);

                let then_cond = Condition::expr(*cond);

                ctx.push_cond(then_cond.clone());
                self.preprocess_expr(&body[*then], ctx);
                ctx.pop_cond();

                if let Some(els) = else_opt {
                    ctx.push_cond(then_cond.not());
                    self.preprocess_expr(&body[*els], ctx);
                    ctx.pop_cond();
                }
            }
            ExprKind::Call { fun, args, .. } => {
                self.preprocess_expr(&body[*fun], ctx);

                for arg in args.iter() {
                    self.preprocess_expr(&body[*arg], ctx)
                }

                let (func_def_id, _substs) = match self.expect_fn().thir_body.borrow()[*fun].ty.kind() {
                    rustc_type_ir::TyKind::FnDef(id, args) => (*id, *args),
                    _ => panic!(),
                };

                let Some(local) = func_def_id.as_local() else {
                    return;
                };

                let Some(attr) = get_hazardflow_attribute(ctx.tcx, ctx.tcx.local_def_id_to_hir_id(local)) else {
                    return;
                };

                let HazardFlowAttr::SystemTask(task) = attr else {
                    return;
                };

                let task = match task {
                    SystemTaskMagic::Display => {
                        let (fstring, span) = get_string_from_thir_id(body.borrow(), args[0]);

                        let arg_id = skip_exprs(body, args[1]);
                        log::debug!("{:#?}", &body[arg_id]);

                        assert!(matches!(&body[arg_id].ty.kind(), rustc_type_ir::TyKind::Tuple(_)));

                        SystemTaskInfo {
                            kind: SystemTaskInfoKind::Display,
                            path_cond: ctx.path_conds(),
                            // TODO: revisit. maybe we need to use `to_string`
                            fstring,
                            arg: arg_id,
                            span,
                        }
                    }
                    SystemTaskMagic::Assert => {
                        let cond_id = skip_exprs(body, args[0]);
                        let (fstring, span) = get_string_from_thir_id(body.borrow(), args[1]);
                        let arg_id = skip_exprs(body, args[2]);
                        log::debug!("{:#?}", &body[arg_id]);
                        assert!(matches!(&body[arg_id].ty.kind(), rustc_type_ir::TyKind::Tuple(_)));

                        SystemTaskInfo {
                            kind: SystemTaskInfoKind::Assert { cond: cond_id },
                            path_cond: ctx.path_conds(),
                            fstring,
                            arg: arg_id,
                            span,
                        }
                    }
                };

                self.system_tasks.push(task)
            }
            ExprKind::Deref { arg } => {
                self.preprocess_expr(&body[*arg], ctx);
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                self.preprocess_expr(&body[*lhs], ctx);
                self.preprocess_expr(&body[*rhs], ctx);
            }
            ExprKind::LogicalOp { lhs, rhs, .. } => {
                self.preprocess_expr(&body[*lhs], ctx);
                self.preprocess_expr(&body[*rhs], ctx);
            }
            ExprKind::Unary { arg, .. } => self.preprocess_expr(&body[*arg], ctx),
            ExprKind::Cast { source } => self.preprocess_expr(&body[*source], ctx),
            ExprKind::Use { source } => self.preprocess_expr(&body[*source], ctx),
            ExprKind::NeverToAny { source } => self.preprocess_expr(&body[*source], ctx),
            // XXX: We come here when panic. ignore for now
            ExprKind::PointerCoercion { .. } => {}
            ExprKind::Loop { .. } => todo!(),
            ExprKind::Let { expr, pat } => {
                self.preprocess_expr(&body[*expr], ctx);

                self.pat_bindings.push(PatBinding { id: *expr, patterns: vec![pat.as_ref().clone()] })
            }
            ExprKind::Match { scrutinee, arms, .. } => {
                let mut patterns = vec![];
                for arm in arms.iter() {
                    let arm: &thir::Arm<'_> = &body[*arm];
                    if let Some(guard) = &arm.guard {
                        match guard {
                            thir::Guard::If(expr) => {
                                self.preprocess_expr(&body[*expr], ctx);

                                ctx.push_cond(Condition::expr(*expr));
                            }
                            thir::Guard::IfLet(..) => todo!(),
                        }
                    }

                    patterns.push(arm.pattern.as_ref().clone());

                    ctx.push_cond(Condition::matches(*scrutinee, arm.pattern.as_ref().clone()));

                    self.preprocess_expr(&body[arm.body], ctx);

                    ctx.pop_cond();

                    if arm.guard.is_some() {
                        ctx.pop_cond();
                    }
                }

                self.pat_bindings.push(PatBinding { id: *scrutinee, patterns })
            }
            ExprKind::Block { block } => self.preprocess_block(&body[*block], ctx),
            ExprKind::Assign { .. } => todo!(),
            ExprKind::AssignOp { .. } => todo!(),
            ExprKind::Field { lhs, .. } => self.preprocess_expr(&body[*lhs], ctx),
            ExprKind::Index { lhs, index } => {
                self.preprocess_expr(&body[*lhs], ctx);
                self.preprocess_expr(&body[*index], ctx);
            }
            ExprKind::VarRef { .. } => {}
            ExprKind::UpvarRef { .. } => {}
            ExprKind::Borrow { arg, .. } => self.preprocess_expr(&body[*arg], ctx),
            ExprKind::AddressOf { .. } => todo!(),
            ExprKind::Break { .. } => todo!(),
            ExprKind::Continue { .. } => todo!(),
            ExprKind::Return { value } => {
                let value = value.unwrap();
                self.explicit_returns.push(Return { value, path_cond: ctx.path_conds() });

                self.preprocess_expr(&body[value], ctx)
            }
            ExprKind::ConstBlock { .. } => todo!(),
            ExprKind::Repeat { value, .. } => self.preprocess_expr(&body[*value], ctx),
            ExprKind::Array { fields } => {
                for field in fields.iter() {
                    self.preprocess_expr(&body[*field], ctx)
                }
            }
            ExprKind::Tuple { fields } => {
                for field in fields.iter() {
                    self.preprocess_expr(&body[*field], ctx)
                }
            }
            ExprKind::Adt(adt_expr) => {
                for field_expr in adt_expr.fields.iter() {
                    self.preprocess_expr(&body[field_expr.expr], ctx)
                }

                if let Some(base) = &adt_expr.base {
                    self.preprocess_expr(&body[base.base], ctx)
                }
            }
            ExprKind::PlaceTypeAscription { .. } => todo!(),
            ExprKind::ValueTypeAscription { .. } => todo!(),
            ExprKind::Closure(closure_expr) => {
                for upvar in closure_expr.upvars.iter() {
                    self.preprocess_expr(&body[*upvar], ctx)
                }
            }
            ExprKind::Literal { .. } => {}
            ExprKind::NonHirLiteral { .. } => todo!(),
            ExprKind::ZstLiteral { .. } => {}
            ExprKind::NamedConst { .. } => {}
            ExprKind::ConstParam { .. } => {}
            ExprKind::StaticRef { .. } => todo!(),
            ExprKind::InlineAsm(_) => todo!(),
            ExprKind::OffsetOf { .. } => todo!(),
            ExprKind::ThreadLocalRef(_) => todo!(),
            ExprKind::Yield { .. } => todo!(),
            ExprKind::Become { .. } => todo!(),
        }
    }

    fn preprocess_stmt(&mut self, stmt: &thir::Stmt<'tcx>, ctx: &mut PreprocessCtx<'tcx>) {
        match &stmt.kind {
            thir::StmtKind::Expr { expr, .. } => self.preprocess_expr(&self.expect_fn().thir_body.borrow()[*expr], ctx),
            thir::StmtKind::Let { pattern, initializer, else_block, .. } => {
                self.preprocess_expr(&self.expect_fn().thir_body.borrow()[initializer.unwrap()], ctx);

                if let Some(block_id) = &else_block {
                    let else_cond = Condition::matches(initializer.unwrap(), pattern.as_ref().clone()).not();
                    ctx.push_cond(else_cond);
                    self.preprocess_block(&self.expect_fn().thir_body.borrow()[*block_id], ctx);
                    ctx.pop_cond()
                }
            }
        }
    }
}

/// Per-fsm cache.
///
/// This prevents the same expression from being allocated multiple times.
#[derive(Debug, Default)]
pub struct FsmCache {
    inner: HashMap<Expr, ExprId>,
    hit: usize,
}

impl FsmCache {
    /// Allocates an expression.
    ///
    /// If the expression has already been allocated, returns the existing id.
    /// Othersise, allocates a new id and returns it.
    ///
    /// While constructing Expr ast, ExprId should only be created by this method, except for
    /// creating constants
    pub fn alloc(&mut self, expr: Expr) -> ExprId {
        if let Some(id) = self.inner.get(&expr) {
            self.hit += 1;
            return *id;
        }
        let id = ExprId::alloc_expr(expr.clone());
        self.inner.insert(expr, id);
        id
    }

    /// Print statistics.
    pub fn stats(&self) -> String {
        format!(
            "\n\tTotal trials: {}\n\tNumber of exprs allocated: {}\n\tcache hit: {}",
            self.inner.len() + self.hit,
            self.inner.len(),
            self.hit
        )
    }

    /// Clear the cache.
    pub fn clear(&mut self) {
        self.inner.clear();
        self.hit = 0;
    }
}

/// Per-function cache.
///
/// This prevents multiple allocations for same `thir::ExprId` in same function.
#[derive(Debug, Default)]
pub struct ThirCache {
    inner: HashMap<thir::ExprId, ExprId>,
    hit: usize,
}

impl ThirCache {
    /// Return the cached id for the given `thir::ExprId`.
    pub fn get(&mut self, thir_id: thir::ExprId) -> Option<ExprId> {
        if let Some(expr_id) = self.inner.get(&thir_id) {
            self.hit += 1;
            Some(*expr_id)
        } else {
            None
        }
    }

    /// Insert the mapping from `thir::ExprId` to `ExprId`.
    pub fn insert(&mut self, thir_id: thir::ExprId, expr_id: ExprId) -> Option<ExprId> {
        self.inner.insert(thir_id, expr_id)
    }

    /// Print statistics.
    pub fn stats(&self) -> String {
        format!(
            "\n\tTotal trials: {}\n\tNumber of exprs allocated: {}\n\tcache hit: {}",
            self.inner.len() + self.hit,
            self.inner.len(),
            self.hit
        )
    }
}

#[derive(Debug, Clone)]
enum Condition<'tcx> {
    Expr(thir::ExprId),

    Matches(Box<thir::Pat<'tcx>>, thir::ExprId),

    Not(Box<Condition<'tcx>>),
}

impl<'tcx> Condition<'tcx> {
    fn expr(id: thir::ExprId) -> Self {
        Self::Expr(id)
    }

    fn not(self) -> Self {
        Condition::Not(Box::new(self))
    }

    fn matches(id: thir::ExprId, pat: thir::Pat<'tcx>) -> Self {
        Condition::Matches(Box::new(pat), id)
    }
}

struct PreprocessCtx<'tcx> {
    inner: Vec<Condition<'tcx>>,
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> std::fmt::Debug for PreprocessCtx<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreprocessCtx").field("inner", &self.inner).finish()
    }
}

impl<'tcx> PreprocessCtx<'tcx> {
    fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self { inner: vec![], tcx }
    }

    fn push_cond(&mut self, cond: Condition<'tcx>) {
        self.inner.push(cond);
    }

    fn pop_cond(&mut self) {
        self.inner.pop();
    }

    fn path_conds(&self) -> Vec<Condition<'tcx>> {
        self.inner.clone()
    }

    fn is_clean(&self) -> bool {
        self.inner.is_empty()
    }
}

#[derive(Debug, Clone)]
struct Return<'tcx> {
    value: thir::ExprId,

    path_cond: Vec<Condition<'tcx>>,
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct SystemTaskInfo<'tcx> {
    kind: SystemTaskInfoKind,
    path_cond: Vec<Condition<'tcx>>,
    fstring: String,
    arg: thir::ExprId,
    span: Span,
}

#[derive(Debug, Clone)]
enum SystemTaskInfoKind {
    Display,
    Assert { cond: thir::ExprId },
}

/// A task to synthesize a display function.
#[derive(Debug, Clone)]
pub struct SystemTask {
    /// Kind
    pub kind: SystemTaskKind,
    /// Format string.
    pub fstring: String,
    /// Path conditions.
    pub path_cond: Option<ExprId>,
    /// Arguments.
    pub args: Vec<ExprId>,
    /// Span.
    pub span: Span,
}

impl SystemTask {
    /// Add path condition.
    pub fn add_path_cond(&mut self, path_cond: ExprId, fsm_cache: &mut FsmCache) {
        let new_path_cond = match self.path_cond {
            Some(old_path_cond) => {
                Expr::BinaryOp { op: BinaryOp::And, lhs: old_path_cond, rhs: path_cond, span: self.span }
                    .alloc_with_fsm_cache(fsm_cache)
            }
            None => path_cond,
        };
        self.path_cond = Some(new_path_cond);
    }
}

/// System task kind.
#[derive(Debug, Clone)]
pub enum SystemTaskKind {
    /// Display
    Display,
    /// Assert
    Assert {
        /// Condition
        cond: ExprId,
    },
}
