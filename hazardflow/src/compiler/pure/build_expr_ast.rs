//! This module constructs `Expr` ast.

use std::collections::VecDeque;
use std::fmt::Formatter;

use itertools::Itertools;
use rustc_middle::mir::BorrowKind;
use rustc_middle::thir::{self, ExprKind, Thir};
use rustc_middle::ty::{AdtKind, EarlyBinder, GenericArgsRef, Instance, ParamEnv, Ty, TyCtxt};
use rustc_span::Span;
use rustc_target::abi::{FieldIdx, VariantIdx};
use rustc_type_ir::fold::TypeFoldable;

use super::*;
use crate::compiler::prelude::*;
use crate::utils::*;

#[derive(Debug, Default)]
pub(super) struct PathCtx {
    pub(super) inner: VecDeque<ExprId>,
}

/// Builds expr from a `ExprId`.
pub(super) struct ExprBuilder<'tcx, 'function_builder> {
    pub(super) tcx: TyCtxt<'tcx>,
    pub(super) expr_id: thir::ExprId,
    pub(super) thir_body: &'tcx rustc_data_structures::steal::Steal<Thir<'tcx>>,
    /// Translates `thir::ExprId` to `ExprId`.
    ///
    /// This prevents allocating the multiple `Expr`s for same `thir::ExprId` in a function.
    pub(super) thir_cache: &'function_builder mut ThirCache,
    /// Translates `Expr` to `ExprId`.
    ///
    /// This prevents allocating the multiple `Expr`s for same `Expr` in a single `Fsm`.
    pub(super) fsm_cache: &'function_builder mut FsmCache,
    pub(super) substs: GenericArgsRef<'tcx>,
    pub(super) args: &'function_builder [PureValue<'tcx>],

    pub(super) path_ctx: PathCtx,

    pub(super) upvars: Option<&'function_builder [(Id, PureValue<'tcx>)]>,

    pub(super) pat_bindings: &'function_builder [PatBinding<'tcx>],
    pub(super) tasks_inner: Vec<SystemTask>,
}

impl<'tcx> std::fmt::Debug for ExprBuilder<'tcx, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExprBuilder")
            .field("expr_id", &self.expr_id)
            .field("thir_body", &self.thir_body.borrow())
            .finish()
    }
}

impl<'tcx, 'function_builder> ExprBuilder<'tcx, 'function_builder> {
    /// Builds expr from a `ExprId`.
    pub(super) fn build(mut self) -> (ExprId, Vec<SystemTask>) {
        (self.build_impl(self.expr_id), self.tasks_inner)
    }

    fn is_closure(&self) -> bool {
        self.upvars.is_some()
    }

    fn monomorphise<T>(&mut self, t: T) -> T
    where T: TypeFoldable<TyCtxt<'tcx>> {
        normalize_alias_ty(self.tcx, EarlyBinder::bind(t).instantiate(self.tcx, self.substs))
    }

    /// Helper function to build exprs recursively.
    fn build_impl(&mut self, expr_id: thir::ExprId) -> ExprId {
        let expr_id = skip_exprs(&self.thir_body.borrow(), expr_id);
        let expr = &self.thir_body.borrow()[expr_id];
        let typ_expected = PortDecls::from_ty(self.monomorphise(expr.ty), self.tcx).unwrap();
        log::debug!("build: {:#?}", expr);

        if let Some(id) = self.thir_cache.get(expr_id) {
            return id;
        }

        let span = expr.span;

        if typ_expected.width() == 0 {
            log::debug!("early return with len 0: {span:#?}\n{expr:#?}\n{typ_expected:#?}");

            return Expr::X { typ: typ_expected, span }.alloc_with_fsm_cache(self.fsm_cache);
        }

        let expr_constructed = match &expr.kind {
            ExprKind::If { cond, then, else_opt, .. } => self.build_conditional(cond, then, else_opt, span),
            ExprKind::Scope { .. } => todo!(),
            ExprKind::Box { .. } => todo!(),
            ExprKind::Call { fun, args, .. } => self.build_call(*fun, args, span),
            ExprKind::Deref { arg } => {
                let arg_skipped = skip_exprs(&self.thir_body.borrow(), *arg);
                let skipped_expr = &self.thir_body.borrow()[arg_skipped];
                if let ExprKind::Call { fun, args, .. } = &skipped_expr.kind {
                    let rustc_type_ir::TyKind::FnDef(id, _) = &self.thir_body.borrow()[*fun].ty.kind() else {
                        panic!()
                    };
                    let parent_name = self.tcx.item_name(self.tcx.parent(*id));
                    let name = self.tcx.item_name(*id);
                    // HACK: Only this is allowed
                    assert!(name.to_string().as_str() == "index" && parent_name.to_string().as_str() == "Index");
                    return self.build_call(*fun, args.as_ref(), span);
                } else {
                    let _ = self.build_impl(*arg);
                    todo!();
                }
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let op = BinaryOp::from(*op);
                let bin_expr = Expr::BinaryOp { op, lhs: self.build_impl(*lhs), rhs: self.build_impl(*rhs), span };
                match op {
                    // TODO: is this efficient? Or should we implement our own implicit
                    // typecasting?
                    BinaryOp::Mod | BinaryOp::Add | BinaryOp::Mul => Expr::resize(
                        bin_expr.clone().alloc_with_fsm_cache(self.fsm_cache),
                        bin_expr.width(),
                        PortDecls::from_ty(self.monomorphise(expr.ty), self.tcx).unwrap().width(),
                        self.fsm_cache,
                        span,
                    ),
                    _ => bin_expr.alloc_with_fsm_cache(self.fsm_cache),
                }
            }
            ExprKind::LogicalOp { op, lhs, rhs } => {
                let op = match op {
                    thir::LogicalOp::And => BinaryOp::And,
                    thir::LogicalOp::Or => BinaryOp::Or,
                };
                Expr::BinaryOp { op, lhs: self.build_impl(*lhs), rhs: self.build_impl(*rhs), span }
                    .alloc_with_fsm_cache(self.fsm_cache)
            }
            ExprKind::Unary { op, arg } => match op {
                rustc_middle::mir::UnOp::Not => {
                    Expr::Not { inner: self.build_impl(*arg), span }.alloc_with_fsm_cache(self.fsm_cache)
                }
                rustc_middle::mir::UnOp::Neg => todo!(),
            },
            ExprKind::Cast { source } => {
                let inner = self.build_impl(*source);

                let inner = match inner.into_expr().port_decls() {
                    PortDecls::Struct(fields) => {
                        let discriminant = inner.member(0, span);

                        let discriminant_width = discriminant.width();

                        assert_eq!(discriminant_width, fields.iter().map(|field| field.1.width()).sum::<usize>());

                        discriminant.alloc_with_fsm_cache(self.fsm_cache)
                    }
                    PortDecls::Bits(_) => inner,
                };

                let PortDecls::Bits(to) = typ_expected.clone() else { panic!() };

                Expr::Cast { from: inner, to, span: expr.span }.alloc_with_fsm_cache(self.fsm_cache)
            }
            ExprKind::NeverToAny { .. } => {
                // panic!()
                Expr::X { typ: PortDecls::from_ty(self.monomorphise(expr.ty), self.tcx).unwrap(), span }
                    .alloc_with_fsm_cache(self.fsm_cache)
            }
            ExprKind::Let { expr, pat } => {
                let expr = self.build_impl(*expr);
                let cond = gen_match_cond(self.tcx, pat.as_ref(), expr, self.fsm_cache);
                assert_eq!(cond.unwrap().into_expr().port_decls().width(), 1);
                cond.unwrap()
            }
            ExprKind::Match { scrutinee, arms, .. } => self.build_match(scrutinee, arms, span),
            ExprKind::Field { lhs, variant_index, name } => self.build_field_expr(lhs, variant_index, name, span),
            ExprKind::Index { lhs, index } => {
                let _lhs = self.build_impl(*lhs);
                let _index = self.build_impl(*index);

                todo!("{:?}", expr)

                // Expr::Get { inner: lhs, typ_elt: (), index: (), span: () }
            }
            ExprKind::VarRef { id } => self.build_var_ref(id, span),
            ExprKind::UpvarRef { var_hir_id, .. } => {
                if let Some(value) = self.build_upvar_ref(var_hir_id) {
                    return value.expr().unwrap();
                }

                unreachable!("upvars: {:?} {var_hir_id:?}", self.upvars)
            }
            ExprKind::Repeat { value, count } => Expr::Repeat {
                inner: self.build_impl(*value),
                count: self.monomorphise(*count).eval_target_usize(self.tcx, ParamEnv::empty()).try_into().unwrap(),
                span,
            }
            .alloc_with_fsm_cache(self.fsm_cache),
            ExprKind::Array { fields } => {
                let fields = fields.iter().map(|expr_id| self.build_impl(*expr_id)).collect::<Vec<_>>();
                let mut field_iter = fields.iter();
                let first = field_iter.next().unwrap().into_expr().port_decls();
                assert!(field_iter.all(|field| field.into_expr().port_decls() == first));

                Expr::ConcatArray { elt_typ: fields[0].into_expr().port_decls(), inner: fields, span }
                    .alloc_with_fsm_cache(self.fsm_cache)
            }
            ExprKind::Tuple { fields } => {
                Expr::tuple(fields.iter().map(|expr_id| self.build_impl(*expr_id)).collect::<Vec<_>>(), span)
                    .alloc_with_fsm_cache(self.fsm_cache)
            }
            ExprKind::Adt(e) => self.build_adt_expr(expr, e, span),
            ExprKind::Literal { lit, neg } => build_literal(neg, lit, expr.ty, self.tcx),
            ExprKind::NamedConst { def_id, args, .. } => {
                let uneval = rustc_middle::mir::UnevaluatedConst::new(*def_id, args);
                match self.tcx.const_eval_resolve(ParamEnv::empty(), uneval, None) {
                    Ok(v) => match v {
                        rustc_middle::mir::ConstValue::Scalar(scalar) => {
                            let value = scalar_to_usize(scalar).expect("scalar value should be resolved");
                            let size = match self.monomorphise(expr.ty).kind() {
                                rustc_type_ir::TyKind::Uint(uint_ty) => uint_ty.bit_width().unwrap_or(32),
                                _ => todo!(),
                            };
                            Expr::unsigned_bits(size.try_into().unwrap(), value, span)
                                .alloc_with_fsm_cache(self.fsm_cache)
                        }
                        rustc_middle::mir::ConstValue::Indirect { alloc_id, offset } => {
                            todo!("{alloc_id:?} {offset:?}")
                        }
                        e => todo!("{e:?}"),
                    },
                    Err(_) => todo!(),
                }
            }
            ExprKind::ConstParam { param, .. } => {
                let c = self.substs.get(param.index as usize).unwrap();
                let c = evaluate_const_generic_arg(self.tcx, c).unwrap();
                match self.monomorphise(expr.ty).kind() {
                    rustc_type_ir::TyKind::Uint(uint_ty) => match uint_ty {
                        rustc_type_ir::UintTy::Usize | rustc_type_ir::UintTy::U32 => {
                            Expr::unsigned_bits(32, c, span).alloc_with_fsm_cache(self.fsm_cache)
                        }
                        rustc_type_ir::UintTy::U8 => todo!(),
                        rustc_type_ir::UintTy::U16 => todo!(),
                        rustc_type_ir::UintTy::U64 => todo!(),
                        rustc_type_ir::UintTy::U128 => todo!(),
                    },
                    rustc_type_ir::TyKind::Adt(i, x) => {
                        assert!(x.is_empty(),);

                        match i.adt_kind() {
                            AdtKind::Enum => {
                                let l = i.variants().len();

                                Expr::unsigned_bits(clog2(l), c, span).alloc_with_fsm_cache(self.fsm_cache)
                            }
                            _ => panic!(),
                        }
                    }
                    _ => todo!(),
                }
            }
            unimpl => todo!("{:#?}", unimpl),
        };
        let typ_constructed = expr_constructed.into_expr().port_decls();
        assert_eq!(
            typ_expected.width(),
            typ_constructed.width(),
            "expr: {:?}\ntypes {:?} {:?}",
            expr,
            typ_expected,
            typ_constructed
        );
        assert!(self.thir_cache.insert(expr_id, expr_constructed).is_none());
        expr_constructed
    }

    fn build_upvar_ref(&mut self, var_hir_id: &thir::LocalVarId) -> Option<PureValue<'tcx>> {
        for (id, upvar) in self.upvars.unwrap().iter() {
            match id {
                Id::Local(id) => {
                    if id == var_hir_id {
                        return Some(upvar.clone());
                    }
                }
                Id::Upvar(_) => todo!(),
            }
        }
        None
    }

    fn push_path_ctx(&mut self, expr_id: ExprId) {
        self.path_ctx.inner.push_back(expr_id)
    }

    fn pop_path_ctx(&mut self) {
        self.path_ctx.inner.pop_back().unwrap();
    }

    fn build_conditional(
        &mut self,
        cond: &thir::ExprId,
        then: &thir::ExprId,
        else_opt: &Option<thir::ExprId>,
        span: Span,
    ) -> ExprId {
        let cond_skipped = skip_exprs(&self.thir_body.borrow(), *cond);

        let cond = self.build_impl(cond_skipped);
        assert_eq!(cond.into_expr().port_decls().width(), 1);

        self.push_path_ctx(cond);

        let then = self.build_impl(*then);

        self.pop_path_ctx();

        let negated = Expr::Not { inner: cond, span }.alloc_with_fsm_cache(self.fsm_cache);

        self.push_path_ctx(negated);

        let els = self.build_impl(else_opt.unwrap());

        self.pop_path_ctx();

        Expr::Cond { cond_expr_pair: vec![(cond, then)], default: els, span }.alloc_with_fsm_cache(self.fsm_cache)
    }

    // TODO: refactor type related to be reused
    // TODO: efficient adt compilation
    fn build_adt_expr(&mut self, expr: &thir::Expr<'tcx>, e: &thir::AdtExpr<'_>, span: Span) -> ExprId {
        let ty = self.monomorphise(expr.ty);
        match ty.kind() {
            rustc_type_ir::TyKind::Adt(adt_def, substs) => match adt_def.adt_kind() {
                rustc_middle::ty::AdtKind::Enum => {
                    let variant = e.adt_def.variant(e.variant_index);
                    let discriminant = get_variant_discriminator(self.tcx, variant);
                    let discriminant =
                        Expr::unsigned_bits(clog2(e.adt_def.variants().len()), discriminant.try_into().unwrap(), span);
                    let mut inner =
                        vec![(Some("discriminant".to_string()), discriminant.alloc_with_fsm_cache(self.fsm_cache))];
                    for (idx, variant) in adt_def.variants().iter().enumerate() {
                        let variant_ident = variant.ident(self.tcx).to_string();
                        if idx == usize::from(e.variant_index) {
                            let mut variant_inner = vec![];
                            for (field_idx, field) in variant.fields.iter().enumerate() {
                                variant_inner.push((
                                    Some(field.name.to_ident_string()),
                                    e.fields
                                        .iter()
                                        .find(|field_expr| field_expr.name == field_idx.into())
                                        .map(|field_expr| self.build_impl(field_expr.expr)),
                                ));
                            }

                            match &e.base {
                                Some(base) => {
                                    if let Expr::Struct { inner: base_inner, .. } =
                                        &*self.build_impl(base.base).into_expr()
                                    {
                                        for (field_idx, field) in base_inner.iter().enumerate() {
                                            if variant_inner[field_idx].1.is_none() {
                                                variant_inner[field_idx].1 = Some(field.1)
                                            }
                                        }
                                    } else {
                                        panic!()
                                    }
                                }
                                None => {
                                    assert!(variant_inner.iter().all(|e| e.1.is_some()))
                                }
                            }
                            inner.push((
                                Some(variant_ident),
                                Expr::Struct {
                                    inner: variant_inner
                                        .into_iter()
                                        .map(|(name, expr)| (name, expr.unwrap()))
                                        .collect(),
                                    span,
                                }
                                .alloc_with_fsm_cache(self.fsm_cache),
                            ))
                        } else {
                            inner.push((
                                Some(variant_ident),
                                Expr::Struct {
                                    inner: variant
                                        .fields
                                        .iter()
                                        .map(|field| {
                                            let ty = self.tcx.type_of(field.did).instantiate(self.tcx, substs);
                                            (
                                                Some(field.ident(self.tcx).to_string()),
                                                Expr::X { typ: PortDecls::from_ty(ty, self.tcx).unwrap(), span }
                                                    .alloc_with_fsm_cache(self.fsm_cache),
                                            )
                                        })
                                        .collect(),
                                    span,
                                }
                                .alloc_with_fsm_cache(self.fsm_cache),
                            ));
                        }
                    }
                    Expr::Struct { inner, span }.alloc_with_fsm_cache(self.fsm_cache)
                }
                rustc_middle::ty::AdtKind::Struct => {
                    let variant = adt_def.variant(e.variant_index);
                    let mut variant_inner = vec![];
                    for (field_idx, field) in variant.fields.iter().enumerate() {
                        variant_inner.push((
                            Some(field.name.to_ident_string()),
                            e.fields
                                .iter()
                                .find(|field_expr| field_expr.name == field_idx.into())
                                .map(|field_expr| self.build_impl(field_expr.expr)),
                        ));
                    }

                    match &e.base {
                        Some(base) => {
                            let base_expr = self.build_impl(base.base);

                            let mut patch = vec![];
                            for (field_idx, field) in variant_inner.iter().enumerate() {
                                if field.1.is_none() {
                                    let member = Expr::Member { inner: base_expr, index: field_idx, span: expr.span }
                                        .alloc_with_fsm_cache(self.fsm_cache);
                                    patch.push((field_idx, member))
                                }
                            }

                            for (idx, member) in patch {
                                assert!(variant_inner[idx].1.is_none());
                                variant_inner[idx].1 = Some(member)
                            }

                            // // TODO: should use type info to fill in the missing fields
                            // // from base
                            // if let Expr::Struct {
                            //     inner: base_inner, ..
                            // } = &*base_expr
                            // {
                            //     for (field_idx, field) in base_inner.iter().enumerate() {
                            //         if variant_inner[field_idx].1.is_none() {
                            //             variant_inner[field_idx].1 = Some(field.1)
                            //         }
                            //     }
                            // } else {
                            //     panic!("{:?}", &*base_expr)
                            // }
                        }
                        None => {
                            assert!(variant_inner.iter().all(|e| e.1.is_some()))
                        }
                    }

                    Expr::Struct {
                        inner: variant_inner.into_iter().map(|(name, expr)| (name, expr.unwrap())).collect(),
                        span,
                    }
                    .alloc_with_fsm_cache(self.fsm_cache)
                }
                rustc_middle::ty::AdtKind::Union => todo!(),
            },
            _ => panic!(),
        }
    }

    fn build_var_ref(&mut self, id: &thir::LocalVarId, span: Span) -> ExprId {
        log::debug!("building var ref");
        let local_var_resolved = resolve_var_ref(self.tcx, self.thir_body, *id, Some(self.pat_bindings));
        assert_ne!(local_var_resolved.len(), 0);

        let mut local_vars = vec![];

        for local_var in local_var_resolved {
            let (match_condition, bounded_expr) = match local_var {
                LocalVar::Param { arg_idx, accessor, .. } => {
                    // NOTE: this is because closure silently adds itself as the first argument
                    let arg_idx = if self.is_closure() { arg_idx - 1 } else { arg_idx };
                    match self.args[arg_idx] {
                        PureValue::Expr(id) => self.build_pattern_access(id, accessor, span),
                        PureValue::Function(_) => todo!(),
                        PureValue::Misc => panic!(),
                    }
                }
                LocalVar::Stmt { expr_id, accessor, .. } => {
                    let expr_bounded = self.build_impl(expr_id);
                    self.build_pattern_access(expr_bounded, accessor, span)
                }
                LocalVar::PatBinding { expr_id, accessor, .. } => {
                    let expr_bounded = self.build_impl(expr_id);
                    self.build_pattern_access(expr_bounded, accessor, span)
                }
            };

            local_vars.push((match_condition, bounded_expr))
        }

        match local_vars.len() {
            0 => panic!(),
            1 => local_vars[0].1,
            _ => {
                let typ = local_vars[0].1.into_expr().port_decls();

                Expr::Cond {
                    cond_expr_pair: local_vars.into_iter().map(|(cond, expr)| (cond.unwrap(), expr)).collect(),
                    default: Expr::X { typ, span }.alloc_with_fsm_cache(self.fsm_cache),
                    span,
                }
                .alloc_with_fsm_cache(self.fsm_cache)
            }
        }
    }

    fn build_pattern_access(&mut self, expr_id: ExprId, accessor: PatAccessor, span: Span) -> (Option<ExprId>, ExprId) {
        let (accesed_expr, access_cond) =
            accessor.iter().fold((expr_id, vec![]), |(acc_expr, mut access_conds), elt| match elt {
                PatAccessNode::Field { idx, .. } => {
                    (acc_expr.member((*idx).into(), span).alloc_with_fsm_cache(self.fsm_cache), access_conds)
                }
                PatAccessNode::Variant { idx, discriminator, .. } => {
                    let variant_discriminator = acc_expr.member(0, span).alloc_with_fsm_cache(self.fsm_cache);

                    let discriminator =
                        Expr::unsigned_bits(variant_discriminator.into_expr().width(), *discriminator as usize, span)
                            .alloc_with_fsm_cache(self.fsm_cache);

                    let variant_match_condition = Expr::BinaryOp {
                        op: BinaryOp::EqArithmetic,
                        lhs: variant_discriminator,
                        rhs: discriminator,
                        span,
                    }
                    .alloc_with_fsm_cache(self.fsm_cache);
                    access_conds.push(variant_match_condition);

                    (acc_expr.member((*idx + 1).into(), span).alloc_with_fsm_cache(self.fsm_cache), access_conds)
                }
                PatAccessNode::Index(_) => todo!(),
            });

        match access_cond.len() {
            0 => (None, accesed_expr),
            _ => (
                Some(
                    access_cond
                        .into_iter()
                        .reduce(|acc, elt| {
                            Expr::BinaryOp { op: BinaryOp::And, lhs: acc, rhs: elt, span }
                                .alloc_with_fsm_cache(self.fsm_cache)
                        })
                        .unwrap(),
                ),
                accesed_expr,
            ),
        }
    }

    fn build_field_expr(
        &mut self,
        lhs: &thir::ExprId,
        _variant_index: &VariantIdx,
        name: &FieldIdx,
        span: Span,
    ) -> ExprId {
        let lhs_ty = self.thir_body.borrow()[skip_exprs(&self.thir_body.borrow(), *lhs)].ty;
        match lhs_ty.kind() {
            rustc_type_ir::TyKind::Tuple(_) => {
                let lhs_expr = self.build_impl(*lhs);

                lhs_expr.member(usize::from(*name), span).alloc_with_fsm_cache(self.fsm_cache)
            }
            rustc_type_ir::TyKind::Adt(def, _) => match def.adt_kind() {
                AdtKind::Enum => todo!(),
                AdtKind::Struct => {
                    let lhs_expr = self.build_impl(*lhs);

                    lhs_expr.member(usize::from(*name), span).alloc_with_fsm_cache(self.fsm_cache)
                }
                AdtKind::Union => todo!(),
            },
            _ => panic!(),
        }
    }

    fn build_match(&mut self, scrutinee: &thir::ExprId, arms: &[thir::ArmId], span: Span) -> ExprId {
        let scrutinee_expr = self.build_impl(*scrutinee);
        let mut condition_expr_pairs = vec![];

        let mut cond_concat = vec![];

        for arm_id in arms.iter() {
            let arm: &thir::Arm<'tcx> = &self.thir_body.borrow()[*arm_id];
            let arm_cond = self.build_arm_cond(scrutinee_expr, arm);

            let arm_expr = if let Some(cond) = arm_cond {
                cond_concat.push(arm_cond);
                self.push_path_ctx(cond);

                let arm_expr = self.build_impl(arm.body);

                self.pop_path_ctx();

                arm_expr
            } else {
                assert!(cond_concat.iter().all(|x| x.is_some()));

                let reduced_cond = cond_concat.clone().into_iter().map(|x| x.unwrap()).reduce(|l, r| {
                    Expr::BinaryOp { op: BinaryOp::And, lhs: l, rhs: r, span }.alloc_with_fsm_cache(self.fsm_cache)
                });

                if let Some(reduced_cond) = reduced_cond {
                    let cond = Expr::Not { inner: reduced_cond, span }.alloc_with_fsm_cache(self.fsm_cache);
                    self.push_path_ctx(cond);

                    let arm_expr = self.build_impl(arm.body);

                    self.pop_path_ctx();
                    arm_expr
                } else {
                    self.build_impl(arm.body)
                }
            };

            condition_expr_pairs.push((arm_cond, arm_expr));
        }

        match condition_expr_pairs.len() {
            0 => panic!(),
            1 => condition_expr_pairs[0].1,
            _ => {
                let (last, rest) = condition_expr_pairs.split_last().unwrap();

                assert!(!rest.is_empty());

                // The only case where the condition is None is when the arm is `_ => ...`, which
                // should only come at last.
                assert!(rest.iter().all(|x| x.0.is_some()));
                Expr::Cond {
                    cond_expr_pair: rest.iter().map(|(cond, body)| (cond.unwrap(), *body)).collect(),
                    default: last.1,
                    span,
                }
                .alloc_with_fsm_cache(self.fsm_cache)

                // // TODO: extend conditional statment to allow multiple conditions and corresponding bodies
                // rest.iter().fold(last.1, |acc, (cond, body)| {
                //     assert_eq!(acc.into_expr().port_decls(), body.into_expr().port_decls());
                //     Expr::Cond {
                //         cond: cond.unwrap(),
                //         lhs: *body,
                //         rhs: acc,
                //         span,
                //     }
                //     .alloc_with_fsm_cache(self.fsm_cache)
                // })
            }
        }
    }

    /// Builds the condition for an arm
    fn build_arm_cond(&mut self, scrutinee_expr: ExprId, arm: &thir::Arm<'tcx>) -> Option<ExprId> {
        let pattern_cond = gen_match_cond(self.tcx, arm.pattern.as_ref(), scrutinee_expr, self.fsm_cache);
        let guard_cond = arm.guard.as_ref().map(|guard| match guard {
            thir::Guard::If(guard_expr_id) => self.build_impl(*guard_expr_id),
            thir::Guard::IfLet(..) => todo!(),
        });
        match (pattern_cond, guard_cond) {
            (None, None) => None,
            (None, Some(e)) | (Some(e), None) => Some(e),
            (Some(pattern_cond), Some(guard_cond)) => Some(
                Expr::BinaryOp { op: BinaryOp::And, lhs: pattern_cond, rhs: guard_cond, span: arm.span }
                    .alloc_with_fsm_cache(self.fsm_cache),
            ),
        }
    }

    fn get_current_path(&mut self, span: Span) -> Option<ExprId> {
        let path_cloned = self.path_ctx.inner.clone();

        path_cloned.into_iter().reduce(|l, r| {
            Expr::BinaryOp { op: BinaryOp::And, lhs: l, rhs: r, span }.alloc_with_fsm_cache(self.fsm_cache)
        })
    }

    /// Builds ast of a call expression
    fn build_call(&mut self, fun_id: thir::ExprId, args: &[thir::ExprId], span: Span) -> ExprId {
        if is_closure_call_with_id(self.tcx, self.thir_body, fun_id, args) {
            assert_eq!(args.len(), 2);
            let builder_args = if let ExprKind::Tuple { fields } = &self.thir_body.borrow()[args[1]].kind {
                self.collect_args(fields)
            } else {
                panic!()
            };
            let PureValue::Function(ref builder) = self.collect_arg(&args[0]) else { panic!() };

            let (expr, tasks) = builder.build(self.tcx, builder_args, self.fsm_cache);

            self.add_tasks(tasks, span);

            return expr;
        }

        let fun_expr_id = skip_exprs(&self.thir_body.borrow(), fun_id);

        let fun_expr = &self.thir_body.borrow()[fun_expr_id];

        let build_args = self.collect_args(args);

        match fun_expr.ty.kind() {
            rustc_type_ir::TyKind::FnDef(id, substs) => {
                let instance = Instance::resolve(self.tcx, ParamEnv::reveal_all(), *id, self.monomorphise(substs))
                    .unwrap()
                    .unwrap();

                log::debug!("mono result: {:#?}", instance);

                let instance_id = instance.def_id();

                if instance_id.is_local() {
                    let (expr, tasks) =
                        FunctionBuilder::new_local(instance, self.tcx).build(self.tcx, build_args, self.fsm_cache);

                    self.add_tasks(tasks, span);

                    expr
                } else {
                    // Handle foreign crate functions
                    let Some(trait_id) = self.tcx.trait_of_item(*id) else { panic!("unimplemented") };

                    if let Some(item) = self.tcx.lang_items().iter().find(|(_, def_id)| *def_id == trait_id) {
                        todo!("{item:?}")
                    }

                    let trait_name = self.tcx.item_name(trait_id);

                    // XXX: This is a hack to handle the Default trait. If someone created a trait
                    // called default, this is likely to fail.
                    //
                    // TODO: use `Meta`
                    if trait_name.as_str() == "Default" {
                        let ty = instance.ty(self.tcx, ParamEnv::empty()).fn_sig(self.tcx).output().skip_binder();
                        match ty.kind() {
                            rustc_type_ir::TyKind::Bool => {
                                Expr::unsigned_bits(1, 0, span).alloc_with_fsm_cache(self.fsm_cache)
                            }
                            rustc_type_ir::TyKind::Uint(uint_ty) => {
                                Expr::unsigned_bits(
                                    uint_ty
                                    .bit_width()
                                    // NOTE: This fails when type is `usize`, and falls back to 32 bits
                                    .unwrap_or(32)
                                    .try_into()
                                    .unwrap(),
                                    0,
                                    span,
                                )
                                .alloc_with_fsm_cache(self.fsm_cache)
                            }
                            tykind => todo!("{tykind:?}"),
                        }
                    } else if trait_name.as_str() == "Into" {
                        todo!("Implement Into trait")
                    } else {
                        todo!("{trait_name}")
                    }
                }
            }
            rustc_type_ir::TyKind::FnPtr(i) => {
                todo!("Call2: {:#?}", i)
            }
            _ => todo!("No user ty"),
        }
    }

    fn add_tasks(&mut self, mut displays: Vec<SystemTask>, span: Span) {
        let current_path_cond = self.get_current_path(span);

        if let Some(current_path_cond) = current_path_cond {
            for display in displays.iter_mut() {
                display.add_path_cond(current_path_cond, self.fsm_cache)
            }
        }

        self.tasks_inner.append(&mut displays);
    }

    /// Collects the arguments of a function call
    fn collect_args(&mut self, args: &[thir::ExprId]) -> Vec<PureValue<'tcx>> {
        args.iter().map(|arg| self.collect_arg(arg)).collect()
    }

    /// Recursively collect an `FunctionArg` given `ExprId`
    fn collect_arg(&mut self, arg: &thir::ExprId) -> PureValue<'tcx> {
        let arg = skip_exprs(&self.thir_body.borrow(), *arg);

        let expr = &self.thir_body.borrow()[arg];
        log::debug!("Collect Arg: {:?}", expr);
        let expr_ty = self.monomorphise(self.thir_body.borrow()[arg].ty);

        if let rustc_type_ir::TyKind::Closure(..) = expr_ty.kind() {
            return match &expr.kind {
                ExprKind::Closure(closure_expr) => {
                    let instance = Instance::resolve(
                        self.tcx,
                        ParamEnv::empty(),
                        closure_expr.closure_id.to_def_id(),
                        self.monomorphise(match closure_expr.args {
                            rustc_middle::ty::UpvarArgs::Closure(substs) => substs,
                            rustc_middle::ty::UpvarArgs::Coroutine(_) => todo!(),
                        }),
                    )
                    .unwrap()
                    .unwrap();

                    let upvars = closure_expr.upvars.iter().map(|id| self.collect_upvar(id)).collect_vec();

                    PureValue::Function(FunctionBuilder::new_closure(instance, upvars, self.tcx))
                }
                ExprKind::VarRef { id } => self.collect_var_ref(id),
                ExprKind::UpvarRef { var_hir_id, .. } => self.build_upvar_ref(var_hir_id).expect("upvar not found"),
                _ => panic!(),
            };
        }

        if let rustc_type_ir::TyKind::FnDef(id, substs) = expr_ty.kind() {
            return match &expr.kind {
                ExprKind::VarRef { id } => self.collect_var_ref(id),
                ExprKind::UpvarRef { var_hir_id, .. } => self.build_upvar_ref(var_hir_id).expect("upvar not found"),
                _ => {
                    let instance = Instance::resolve(self.tcx, ParamEnv::empty(), *id, self.monomorphise(substs))
                        .unwrap()
                        .unwrap();

                    if instance.def_id().is_local() {
                        PureValue::Function(FunctionBuilder::new_local(instance, self.tcx))
                    } else {
                        // TODO: merge with `build_call`
                        panic!()
                    }
                }
            };
        }

        if let rustc_type_ir::TyKind::FnPtr(_) = expr_ty.kind() {
            todo!()
        }

        PureValue::Expr(self.build_impl(arg))
    }

    fn collect_var_ref(&mut self, id: &thir::LocalVarId) -> PureValue<'tcx> {
        let mut local_var_resolved = resolve_var_ref(self.tcx, self.thir_body, *id, Some(self.pat_bindings));
        assert_eq!(local_var_resolved.len(), 1);
        match local_var_resolved.pop().unwrap() {
            LocalVar::Param { arg_idx, accessor, .. } => {
                assert!(accessor.is_empty());
                let arg_idx = if self.is_closure() {
                    // NOTE: this is because closure silently adds itself as the first argument
                    arg_idx - 1
                } else {
                    arg_idx
                };

                assert!(self.args[arg_idx].function().unwrap().ast.is_closure());

                self.args[arg_idx].clone()
            }
            LocalVar::Stmt { expr_id, accessor, .. } => {
                assert!(accessor.is_empty());
                self.collect_arg(&expr_id)
            }
            LocalVar::PatBinding { .. } => todo!(),
        }
    }

    fn collect_upvar(&mut self, arg: &thir::ExprId) -> (Id, PureValue<'tcx>) {
        let expr = &self.thir_body.borrow()[*arg];
        match &expr.kind {
            ExprKind::Scope { lint_level, .. } => match lint_level {
                thir::LintLevel::Inherited => todo!(),
                thir::LintLevel::Explicit(id) => (Id::Upvar(*id), self.collect_arg(arg)),
            },
            ExprKind::Field { lhs, .. } => self.collect_upvar(lhs),
            ExprKind::VarRef { id, .. } => (Id::Local(*id), self.collect_arg(arg)),
            ExprKind::UpvarRef { var_hir_id, .. } => (Id::Local(*var_hir_id), self.collect_arg(arg)),
            ExprKind::Borrow { borrow_kind, arg } => {
                assert_eq!(*borrow_kind, BorrowKind::Shared);
                self.collect_upvar(arg)
            }
            unimpl => panic!("{unimpl:?}"),
        }
    }
}

/// Build an expression from a literal
pub fn build_literal<'tcx>(
    neg: &bool,
    lit: &&rustc_span::source_map::Spanned<rustc_ast::LitKind>,
    ty: Ty<'tcx>,
    tcx: TyCtxt<'tcx>,
) -> ExprId {
    assert!(!neg);
    let typ = PortDecls::from_ty(ty, tcx).unwrap();
    let expr = match lit.node {
        rustc_ast::LitKind::Str(..) => todo!(),
        rustc_ast::LitKind::ByteStr(..) => todo!(),
        rustc_ast::LitKind::CStr(..) => todo!(),
        rustc_ast::LitKind::Byte(_) => todo!(),
        rustc_ast::LitKind::Char(_) => todo!(),
        rustc_ast::LitKind::Int(value, _) => {
            log::debug!("ty: {:?}, value: {:?}", typ, value);
            if typ.is_signed() {
                Expr::signed_bits(typ.width(), value.try_into().unwrap(), lit.span)
            } else {
                Expr::unsigned_bits(typ.width(), value.try_into().unwrap(), lit.span)
            }
        }
        rustc_ast::LitKind::Float(..) => todo!(),
        rustc_ast::LitKind::Bool(b) => Expr::unsigned_bits(1, b as usize, lit.span),
        rustc_ast::LitKind::Err => todo!(),
    };
    ExprId::alloc_expr(expr)
}

/// Build a constant expression while constructing submodule graph
pub fn build_const_expr<'tcx>(
    tcx: TyCtxt<'tcx>,
    expr_id: thir::ExprId,
    thir_body: &'tcx rustc_data_structures::steal::Steal<Thir<'tcx>>,
    substs: GenericArgsRef<'tcx>,
    args: &[PureValue<'tcx>],
    upvars: Option<&[(Id, PureValue<'tcx>)]>,
) -> ExprId {
    let (expr, displays) = ExprBuilder {
        tcx,
        expr_id,
        thir_body,
        substs,
        // TODO: global?
        thir_cache: &mut ThirCache::default(),
        fsm_cache: &mut FsmCache::default(),
        args,
        upvars,
        pat_bindings: &[],
        tasks_inner: vec![],
        path_ctx: PathCtx::default(),
    }
    .build();
    assert!(displays.is_empty());
    expr
}

/// Generate match condition for a pattern and expr pair
#[allow(clippy::needless_lifetimes)]
pub fn gen_match_cond<'tcx>(
    tcx: TyCtxt<'tcx>,
    pattern: &thir::Pat<'tcx>,
    match_arg: ExprId,
    fsm_cache: &mut FsmCache,
) -> Option<ExprId> {
    match &pattern.kind {
        thir::PatKind::Wild => None,
        thir::PatKind::AscribeUserType { subpattern, .. } => {
            gen_match_cond(tcx, subpattern.as_ref(), match_arg, fsm_cache)
        }
        thir::PatKind::Binding { subpattern, .. } => {
            assert!(subpattern.is_none());
            None
        }
        thir::PatKind::Variant { adt_def, variant_index, subpatterns, .. } => match adt_def.adt_kind() {
            rustc_middle::ty::AdtKind::Struct => todo!(),
            rustc_middle::ty::AdtKind::Union => todo!(),
            rustc_middle::ty::AdtKind::Enum => {
                let arg_discriminant = match_arg.member(0, pattern.span);

                let discriminant = get_variant_discriminator(tcx, adt_def.variant(*variant_index)) as usize;

                // TODO: get the discriminant
                let discriminant = Expr::unsigned_bits(arg_discriminant.width(), discriminant, pattern.span);
                assert_eq!(discriminant.port_decls(), arg_discriminant.port_decls());
                let discriminant_eq = Expr::BinaryOp {
                    op: BinaryOp::EqArithmetic,
                    lhs: arg_discriminant.alloc_with_fsm_cache(fsm_cache),
                    rhs: discriminant.alloc_with_fsm_cache(fsm_cache),
                    span: pattern.span,
                }
                .alloc_with_fsm_cache(fsm_cache);

                let conds = subpatterns
                    .iter()
                    .filter_map(|fieldpat| {
                        let variant_expr = match_arg
                            .member(1usize + usize::from(*variant_index), pattern.span)
                            .alloc_with_fsm_cache(fsm_cache)
                            .member(fieldpat.field.into(), pattern.span);
                        gen_match_cond(
                            tcx,
                            fieldpat.pattern.as_ref(),
                            variant_expr.alloc_with_fsm_cache(fsm_cache),
                            fsm_cache,
                        )
                    })
                    .collect::<Vec<_>>();

                // TODO: use reduction operator
                Some(conds.into_iter().fold(discriminant_eq, |acc, elt| {
                    assert_eq!(acc.into_expr().port_decls(), elt.into_expr().port_decls());
                    Expr::BinaryOp { op: BinaryOp::And, lhs: acc, rhs: elt, span: pattern.span }
                        .alloc_with_fsm_cache(fsm_cache)
                }))
            }
        },
        thir::PatKind::Leaf { subpatterns } => {
            let mut conds = vec![];

            for subpat in subpatterns.iter() {
                let e = match_arg.member(subpat.field.into(), pattern.span).alloc_with_fsm_cache(fsm_cache);

                if let Some(cond) = gen_match_cond(tcx, subpat.pattern.as_ref(), e, fsm_cache) {
                    conds.push(cond);
                }
            }

            match conds.len() {
                0 => None,
                _ => {
                    let (first, rest) = conds.split_first().unwrap();
                    // TODO: use reduction operator
                    Some(rest.iter().fold(*first, |acc, elt| {
                        assert_eq!(acc.into_expr().port_decls(), elt.into_expr().port_decls());
                        Expr::BinaryOp { op: BinaryOp::And, lhs: acc, rhs: *elt, span: pattern.span }
                            .alloc_with_fsm_cache(fsm_cache)
                    }))
                }
            }
        }
        thir::PatKind::Deref { .. } => todo!(),
        thir::PatKind::Constant { value } => match value {
            rustc_middle::mir::Const::Ty(c) => {
                let ty = PortDecls::from_ty(c.ty(), tcx).unwrap();
                assert_eq!(ty, match_arg.into_expr().port_decls());

                let value = c.try_eval_bits(tcx, ParamEnv::empty()).unwrap();
                let const_expr = if ty.is_signed() {
                    todo!()
                } else {
                    Expr::unsigned_bits(ty.width(), value.try_into().unwrap(), pattern.span)
                        .alloc_with_fsm_cache(fsm_cache)
                };

                assert_eq!(const_expr.into_expr().port_decls(), match_arg.into_expr().port_decls());

                Some(
                    Expr::BinaryOp { op: BinaryOp::EqArithmetic, lhs: const_expr, rhs: match_arg, span: pattern.span }
                        .alloc_with_fsm_cache(fsm_cache),
                )
            }
            rustc_middle::mir::Const::Unevaluated(..) => {
                todo!("constant pattern: {:?}", value)
            }
            rustc_middle::mir::Const::Val(..) => todo!("constant pattern: {:?}", value),
        },
        thir::PatKind::Range(_) => todo!(),
        thir::PatKind::Slice { .. } => todo!(),
        thir::PatKind::Array { .. } => todo!(),
        thir::PatKind::Or { pats } => {
            let mut conds = vec![];
            for pat in pats.iter() {
                conds.push(gen_match_cond(tcx, pat.as_ref(), match_arg, fsm_cache).unwrap())
            }
            match conds.len() {
                0 => panic!(),
                _ => {
                    // TODO: use reduction operator
                    let (first, rest) = conds.split_first().unwrap();
                    Some(rest.iter().fold(*first, |acc, elt| {
                        assert_eq!(acc.into_expr().port_decls(), elt.into_expr().port_decls());
                        Expr::BinaryOp { op: BinaryOp::Or, lhs: acc, rhs: *elt, span: pattern.span }
                            .alloc_with_fsm_cache(fsm_cache)
                    }))
                }
            }
        }
        thir::PatKind::InlineConstant { .. } => todo!(),
        thir::PatKind::Never => todo!(),
        thir::PatKind::Error(_) => todo!(),
    }
}
