//! Virgen

use std::collections::HashMap;

use hir::def_id::LocalDefId;
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
use rustc_hir as hir;
use rustc_middle::ty::{Instance, ParamEnv, TyCtxt};

use super::*;
use crate::utils::*;
use crate::vir;
use crate::vir::{ContinuousAssign, Declaration, Expression, Range, Statement};

/// Virgen a single module, which results in a sinvle `*.v` file
pub(crate) struct Virgen<'tcx> {
    /// TyCtxt
    tcx: TyCtxt<'tcx>,

    /// Meta
    meta: Rc<Meta>,

    /// If `true`, generate `$fdisplay`. Otherwise ignore them.
    options: Rc<Options>,

    /// def id of the module to be virgened
    pub(crate) instance: Instance<'tcx>,

    /// Module Signature
    pub(crate) sig: ModuleSig<'tcx>,

    /// Arguements
    pub(crate) args: Vec<ModuleGraphValue<'tcx>>,

    pub(crate) upvars: Option<Vec<(Id, ModuleGraphValue<'tcx>)>>,

    /// Modules in the module
    pub(crate) submodules: Vec<(Module<'tcx>, Interface)>,

    /// Module's Output Interface
    pub(crate) output_interface: Option<Interface>,

    /// Prefix
    pub(crate) prefix: Vec<String>,
}

impl<'tcx> std::fmt::Debug for Virgen<'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Virgen")
            .field("id", &self.instance)
            .field("name", &self.name())
            .field("modules", &self.submodules)
            .finish()
    }
}

impl<'tcx> Virgen<'tcx> {
    /// Creates new `Virgen` context for top-level module.
    pub(crate) fn top(tcx: TyCtxt<'tcx>, meta: Rc<Meta>, options: Rc<Options>, id: LocalDefId) -> Self {
        let rustc_type_ir::TyKind::FnDef(id, substs) = tcx.type_of(id).skip_binder().kind() else { panic!() };
        let instance = Instance::resolve(tcx, ParamEnv::empty(), *id, substs).unwrap().unwrap();

        let sig = ModuleSig::from_instance(tcx, meta.as_ref(), instance, None).unwrap();

        let args = sig
            .params
            .iter()
            .enumerate()
            .map(|(i, p)| match p {
                ModuleGraphType::Interface(_) => InterfaceValue::external_interface(
                    [EndpointNode::Field("input".to_string(), None), EndpointNode::Field(i.to_string(), None)]
                        .into_iter()
                        .collect(),
                )
                .into(),
                ModuleGraphType::Module(_) => ModuleValue::external_module(
                    [EndpointNode::Field("input".to_string(), None), EndpointNode::Field(i.to_string(), None)]
                        .into_iter()
                        .collect(),
                )
                .into(),
                ModuleGraphType::Misc(_) => panic!(),
                ModuleGraphType::ComposedModule(_) => todo!(),
            })
            .collect();

        Self {
            tcx,
            meta,
            instance,
            sig,
            args,
            submodules: vec![],
            output_interface: None,
            prefix: vec![],
            upvars: None,
            options,
        }
    }

    /// Creates new `Virgen` context for submodule.
    pub(crate) fn submodule(
        tcx: TyCtxt<'tcx>,
        meta: Rc<Meta>,
        options: Rc<Options>,
        module_inst: ModuleInst<'tcx>,
    ) -> Self {
        Self {
            tcx,
            meta,
            instance: module_inst.instance,
            sig: module_inst.sig,
            args: module_inst.args,
            prefix: module_inst.prefix,
            submodules: Default::default(),
            output_interface: None,
            upvars: module_inst.upvars,
            options,
        }
    }

    /// Returns the name of the module
    pub(crate) fn name(&self) -> String {
        let (prefix, postfix) = if self.prefix.is_empty() {
            (None, Some("top".to_string()))
        } else {
            (Some(self.prefix.clone().join("_")), None)
        };

        join_options("_", [
            prefix,
            if self.is_closure() {
                Some("closure".to_string())
            } else {
                Some(self.tcx.item_name(self.instance.def_id()).to_string())
            },
            postfix,
        ])
        .unwrap()
    }

    pub(crate) fn input_interface_typ(&self) -> InterfaceTyp {
        self.sig.input_interface_typ()
    }

    pub(crate) fn output_interface_typ(&self) -> InterfaceTyp {
        self.sig.output_interface_typ()
    }

    pub(crate) fn output_interface(&self) -> VirgenResult<Interface> {
        self.output_interface
            .as_ref()
            .ok_or_else(|| VirgenError::Misc { msg: "No output interface. Call preprocess first".to_string() })
            .cloned()
    }

    /// Preprocesses the module.
    ///
    /// It does the following:
    /// - Collects all the modules and how their interfaces are interwined
    /// - returns all the module instantiations in the module
    pub(crate) fn preprocess(&mut self) -> VirgenResult<Vec<Module<'tcx>>> {
        log::info!("Preprocessing {:?}", self.name());

        let (submodule_graph, output_interface) = construct_submodule_graph(self.meta.as_ref(), self.tcx, self)?;

        self.submodules = submodule_graph;
        self.output_interface = Some(output_interface);

        Ok(self
            .submodules
            .iter()
            .filter_map(|(module, _)| match &*module.inner {
                ModuleInner::ModuleInst(_) => Some(module.clone()),
                _ => None,
            })
            .collect())
    }

    /// Generate the virgen module
    ///
    /// NOTE: This function should only be called after `preprocess`
    pub(crate) fn virgen(&self) -> VirgenResult<vir::Module> {
        log::info!("Translating module {}", self.name());

        log::info!("Generating Port declarations");
        // 1. Generate all the port declarations from function signature
        let port_decls = self.gen_port_decls()?;

        log::info!("Generating Module Items");
        // 2. Translate generate function body
        let module_items = self.gen_module_items()?;

        // 3. Generate the module
        let module = vir::Module { name: self.name(), port_decls, module_items };
        log::info!("Translation finished");

        Ok(module)
    }

    /// TODO: need to refactor. Don't do string spliting
    pub(crate) fn top_module_name(&self) -> String {
        if self.prefix.is_empty() { self.name() } else { self.prefix[0].clone() }
            .split('_')
            .collect_vec()
            .split_last()
            .unwrap()
            .1
            .join("_")
    }

    fn gen_port_decls(&self) -> VirgenResult<Vec<vir::PortDeclaration>> {
        Ok(gen_port_decls(self)?
            .into_iter()
            .map(|(dir, width, name)| match dir {
                Direction::Input => vir::PortDeclaration::input(width, name),
                Direction::Output => vir::PortDeclaration::output(width, name),
            })
            .collect())
    }

    fn gen_module_wiring(&self, prefix: Option<String>) -> VirgenResult<Vec<ContinuousAssign>> {
        Ok(gen_wiring(self, prefix)?
            .into_iter()
            .map(|(lvalue, lvalue_range, rvalue, rvalue_range)| {
                let lvalue_expr = match lvalue_range {
                    Some((index, elt_size)) => vir::Expression::ident(lvalue).with_range(vir::Range::new_range(
                        vir::Expression::binary(
                            BinaryOp::Mul,
                            vir::Expression::number(index.to_string()),
                            vir::Expression::number(elt_size.to_string()),
                        ),
                        vir::Expression::number(elt_size.to_string()),
                    )),
                    None => vir::Expression::ident(lvalue),
                };
                let rvalue_expr = match rvalue_range {
                    Some((index, elt_size)) => vir::Expression::ident(rvalue).with_range(vir::Range::new_range(
                        vir::Expression::binary(
                            BinaryOp::Mul,
                            vir::Expression::number(index.to_string()),
                            vir::Expression::number(elt_size.to_string()),
                        ),
                        vir::Expression::number(elt_size.to_string()),
                    )),
                    None => vir::Expression::ident(rvalue),
                };
                vir::ContinuousAssign::new(lvalue_expr, rvalue_expr)
            })
            .collect())
    }

    fn gen_module_items(&self) -> VirgenResult<Vec<vir::ModuleItem>> {
        let mut ctx = Context::new();

        let mut module_items = vec![];

        let mut decls = LinkedHashMap::<String, Vec<Declaration>>::new();

        gen_submodule_wires(self, &mut ctx)?
            .into_iter()
            .for_each(|(meta, name, shape)| decls.entry(meta).or_default().push(vir::Declaration::net(shape, name)));

        for (meta, decls) in decls.into_iter() {
            if !decls.is_empty() {
                module_items.push(vir::ModuleItem::Commented(
                    format!("Wires declared by {}", meta),
                    Some(format!("End wires declared by {}", meta)),
                    vec![vir::ModuleItem::Declarations(decls)],
                ));
            }
        }

        let conts = self.gen_module_wiring(ctx.get_prefix())?;
        if !conts.is_empty() {
            module_items.push(vir::ModuleItem::Commented(
                format!("Wiring by {}", &self.name()),
                Some(format!("End wiring by {}", &self.name())),
                vec![vir::ModuleItem::ContinuousAssigns(conts)],
            ));
        }

        let mut submodule_items = vec![];

        // Add inner submodule's logic.
        for (index, (submodule, _)) in self.submodules.iter().enumerate() {
            let comp_name = submodule.get_module_name();
            ctx.enter_scope(format!("{comp_name}_{index}"));
            match &*submodule.inner {
                ModuleInner::Fsm(module) => {
                    submodule_items.append(&mut self.gen_module_fsm(module, &mut ctx)?);
                }
                ModuleInner::ModuleInst(module) => {
                    submodule_items.append(&mut self.gen_module_inst(module, &mut ctx)?);
                }
                ModuleInner::Ffi(module) => {
                    submodule_items.append(&mut self.gen_module_ffi(module, &mut ctx)?);
                }
                ModuleInner::ModuleSplit(module) => {
                    submodule_items.append(&mut self.gen_module_split(module, &mut ctx)?);
                }
                ModuleInner::ModuleSeq(module) => {
                    submodule_items.append(&mut self.gen_module_seq(module, &mut ctx)?);
                }
            }
            ctx.leave_scope();
        }
        if !submodule_items.is_empty() {
            module_items.push(vir::ModuleItem::Commented(
                format!("Submodules of {}", self.name()),
                Some(format!("End submodules of {}", self.name())),
                submodule_items,
            ));
        }
        Ok(module_items)
    }

    fn gen_module_ffi(&self, module: &Ffi<'tcx>, ctx: &mut Context) -> VirgenResult<Vec<vir::ModuleItem>> {
        let connections = gen_connections(module, ctx)?
            .into_iter()
            .map(|(_, port, expr)| (port, vir::Expression::ident(expr)))
            .collect();

        let module_inst = vir::ModuleInstantiation::new(
            module.get_module_name(),
            module.inst_name.clone(),
            module.params.clone(),
            connections,
        );

        Ok(vec![vir::ModuleItem::ModuleInstantiation(module_inst)])
    }

    fn gen_module_inst(&self, module: &ModuleInst<'tcx>, ctx: &mut Context) -> VirgenResult<Vec<vir::ModuleItem>> {
        let connections = gen_connections(module, ctx)?
            .into_iter()
            .map(|(_, port, expr)| (port, vir::Expression::ident(expr)))
            .collect();

        let module_inst = vir::ModuleInstantiation::new(
            if module.prefix.is_empty() {
                module.get_module_name()
            } else {
                format!("{}_{}", module.prefix.join("_"), module.get_module_name())
            },
            if module.prefix.is_empty() {
                module.inst_name.clone()
            } else {
                format!("{}_{}", module.prefix.join("_"), module.inst_name)
            },
            module.params.clone(),
            connections,
        );

        Ok(vec![vir::ModuleItem::ModuleInstantiation(module_inst)])
    }

    fn gen_module_split(&self, module: &ModuleSplit<'tcx>, ctx: &mut Context) -> VirgenResult<Vec<vir::ModuleItem>> {
        let wires = gen_module_split_assigns(module, ctx)?;
        log::debug!("Wiring: {:?}", wires);

        Ok(vec![vir::ModuleItem::ContinuousAssigns(
            wires
                .into_iter()
                .map(|(dir, incoming, outgoing)| match dir {
                    Direction::Input => ContinuousAssign::new(Expression::ident(outgoing), Expression::ident(incoming)),
                    Direction::Output => {
                        ContinuousAssign::new(Expression::ident(incoming), Expression::ident(outgoing))
                    }
                })
                .collect(),
        )])
    }

    fn gen_module_seq(&self, module: &ModuleSeq<'tcx>, ctx: &mut Context) -> VirgenResult<Vec<vir::ModuleItem>> {
        let wires = gen_module_seq_assigns(module, ctx)?;

        // TODO: Use `gen_module_wiring`
        Ok(vec![vir::ModuleItem::ContinuousAssigns(
            wires
                .into_iter()
                .map(|(lvalue, lvalue_range, rvalue, rvalue_range)| {
                    let lvalue_expr = match lvalue_range {
                        Some((index, elt_size)) => vir::Expression::ident(lvalue).with_range(vir::Range::new_range(
                            vir::Expression::binary(
                                BinaryOp::Mul,
                                vir::Expression::number(index.to_string()),
                                vir::Expression::number(elt_size.to_string()),
                            ),
                            vir::Expression::number(elt_size.to_string()),
                        )),
                        None => vir::Expression::ident(lvalue),
                    };
                    let rvalue_expr = match rvalue_range {
                        Some((index, elt_size)) => vir::Expression::ident(rvalue).with_range(vir::Range::new_range(
                            vir::Expression::binary(
                                BinaryOp::Mul,
                                vir::Expression::number(index.to_string()),
                                vir::Expression::number(elt_size.to_string()),
                            ),
                            vir::Expression::number(elt_size.to_string()),
                        )),
                        None => vir::Expression::ident(rvalue),
                    };
                    vir::ContinuousAssign::new(lvalue_expr, rvalue_expr)
                })
                .collect(),
        )])
    }

    fn gen_module_fsm(&self, module: &Fsm<'tcx>, ctx: &mut Context) -> VirgenResult<Vec<vir::ModuleItem>> {
        let module_prefix = ctx.get_prefix();

        let fsm_function_builder = &module.fsm_logic;

        let sig = fsm_function_builder.sig(self.tcx);

        let (ip, eb, ep, ib) = gen_fsm_identifiers(module, ctx)?;

        let fsm_inputs = ["ip", "eb", "state"]
            .iter()
            .zip_eq(sig.inputs())
            .map(|(prefix, ty)| {
                PureValue::Expr(ExprId::alloc_expr(Expr::input(
                    Some(prefix.to_string()),
                    PortDecls::from_ty(*ty, self.tcx).unwrap(),
                    fsm_function_builder.span,
                )))
            })
            .collect_vec();

        let mut cache = HashMap::new();

        let (fsm_wire_decls, state_reg, ingress_conts) =
            self.gen_fsm_prelude(&fsm_inputs, module_prefix, ctx, &mut cache, ip, eb)?;

        // XXX: This is a bad design
        ctx.clear_fsm_ctx();

        let (fsm_ast, displays) = fsm_function_builder.build(self.tcx, fsm_inputs, &mut ctx.fsm_cache);

        let fsm_ast = &fsm_ast.into_expr();
        // NOTE: This should come before translating exprs for displays
        let (fsm_decls, fsm_stmts, fsm_expr) = self.gen_expr(fsm_ast, ctx, &mut cache)?;

        let (mut decls_for_displays, mut stmts_for_displays) = (vec![], vec![]);
        for display in displays {
            let (mut d, mut s) = self.gen_system_task(display, ctx, &mut cache)?;
            decls_for_displays.append(&mut d);
            stmts_for_displays.append(&mut s);
        }
        for display in ctx.displays.clone() {
            let (mut d, mut s) = self.gen_system_task(display, ctx, &mut cache)?;
            decls_for_displays.append(&mut d);
            stmts_for_displays.append(&mut s);
        }

        let (fsm_result_conts, state_results) = match (fsm_expr, fsm_ast.port_decls()) {
            (CompositeExpr::Struct(fsm_exprs), PortDecls::Struct(inner_types)) => {
                assert_eq!(fsm_exprs.len(), 3);
                let ep_assigns = self.cont_assign_exprs(
                    ep.into_iter().map(|(_, wire, _)| Expression::ident(wire)),
                    filter_nonzero(fsm_exprs[0].clone(), inner_types[0].1.clone()),
                )?;
                let ih_assigns = self.cont_assign_exprs(
                    ib.into_iter().map(|(_, wire, _)| Expression::ident(wire)),
                    filter_nonzero(fsm_exprs[1].clone(), inner_types[1].1.clone()),
                )?;
                ([ep_assigns, ih_assigns].concat(), fsm_exprs[2].clone())
            }
            _ => unreachable!(),
        };

        let mut blocking_stmts = vec![];
        let mut var_array_updates = vec![];

        // TODO: Remove this HACK
        for stmt in fsm_stmts {
            match stmt {
                Statement::NonblockingAssignment(..) => var_array_updates.push(stmt),
                Statement::Conditional(ref cond_expr_pairs, ref else_stmt, _) => {
                    let is_var_array_update_1 = cond_expr_pairs.iter().all(|(_, stmts)| {
                        stmts.iter().all(|stmt| matches!(stmt, Statement::NonblockingAssignment(..)))
                    });

                    let is_var_array_update_2 =
                        else_stmt.iter().all(|stmt| matches!(stmt, Statement::NonblockingAssignment(..)));

                    if is_var_array_update_1 && is_var_array_update_2 {
                        var_array_updates.push(stmt);
                    } else {
                        blocking_stmts.push(stmt);
                    }
                }
                _ => blocking_stmts.push(stmt),
            }
        }

        let always_comb = vir::ModuleItem::AlwaysConstruct("always @*".to_string(), blocking_stmts.clone());

        // 3. generate the state update logic
        // always @(posedge clk) begin
        //     ... // (4) state update logic
        //
        //     if (rst) begin
        //         ... // (4) state update logic (reset)
        //     end else begin
        //         ... // (4) state update logic (non-reset)
        //     end
        // end
        let state_update = state_reg
            .iter()
            .zip_eq(state_results.iter())
            .filter_map(|(state_decl, state_next)| {
                if state_decl.shape().dim() == 2 {
                    None
                } else {
                    Some(vir::Statement::nonblocking_assignment(
                        state_decl.ident(),
                        state_next,
                        fsm_function_builder.span,
                    ))
                }
            })
            .collect::<Vec<_>>();

        let (init_decls, init_stmts, init_expr) = self.gen_expr(&module.init_value.into_expr(), ctx, &mut cache)?;

        let reset_update = state_reg
            .iter()
            // TODO: make it zip_eq
            .zip_eq(&init_expr)
            .filter_map(|(s, p)| {
                if s.shape().dim() == 2 {
                    None
                } else {
                    Some(vir::Statement::nonblocking_assignment(
                        s.ident(),
                        p,
                        fsm_function_builder.span,
                    ))
                }
            })
            .collect::<Vec<_>>();

        let always_posedge = vir::ModuleItem::AlwaysConstruct(
            "always @(posedge clk)".to_string(),
            [
                vec![vir::Statement::Conditional(
                    vec![(
                        vir::Expression::ident("rst".to_string()),
                        // state update logic (reset)
                        [init_stmts, reset_update].concat(),
                    )],
                    // state update logic
                    [state_update].concat(),
                    fsm_function_builder.span,
                )],
                var_array_updates,
                stmts_for_displays,
            ]
            .concat(),
        );

        // (2) state initialization with dimension > 1
        let var_array_state_init = gen_var_arr_state_init(&state_reg, ctx, fsm_function_builder);

        Ok([
            vec![
                vir::ModuleItem::Declarations(fsm_wire_decls),
                vir::ModuleItem::Declarations(fsm_decls),
                vir::ModuleItem::Declarations(decls_for_displays),
                vir::ModuleItem::Declarations(state_reg.iter().collect::<Vec<_>>()),
                vir::ModuleItem::Declarations(init_decls),
                vir::ModuleItem::ContinuousAssigns(ingress_conts),
                always_comb,
                vir::ModuleItem::ContinuousAssigns(fsm_result_conts),
                always_posedge,
            ],
            var_array_state_init,
        ]
        .concat())
    }

    fn gen_fsm_prelude(
        &self,
        fsm_inputs: &[PureValue<'tcx>],
        module_prefix: Option<String>,
        ctx: &mut Context,
        cache: &mut HashMap<Expr, String>,
        ip: Vec<(Shape, String, String)>,
        eb: Vec<(Shape, String, String)>,
    ) -> VirgenResult<(Vec<Declaration>, CompositeExpr<Declaration>, Vec<ContinuousAssign>)> {
        let mut fsm_wire_decls = vec![];
        let st_input = &*fsm_inputs[2].expr().unwrap().into_expr();
        let state_reg = CompositeExpr::from_typ(
            st_input.port_decls(),
            join_options("_", [module_prefix, "state".to_string().into()]).unwrap(),
        )
        .map(|(name, shape)| Declaration::reg(shape, name));
        let ip_input = &*fsm_inputs[0].expr().unwrap().into_expr();
        let ip_expr = filter_nonzero(self.gen_expr(ip_input, ctx, cache)?.2, ip_input.port_decls());
        let ip_assigns =
            self.cont_assign_exprs(ip_expr.clone(), ip.into_iter().map(|(_, wire, _)| Expression::ident(wire)))?;
        fsm_wire_decls.append(
            &mut ip_expr
                .into_iter()
                .zip_eq(&ip_input.port_decls())
                .map(|(ident, (_, shape))| Declaration::net(shape, ident.to_string()))
                .collect::<Vec<_>>(),
        );
        let eb_input = &*fsm_inputs[1].expr().unwrap().into_expr();
        let eb_expr = filter_nonzero(self.gen_expr(eb_input, ctx, cache)?.2, eb_input.port_decls());
        let eb_assigns =
            self.cont_assign_exprs(eb_expr.clone(), eb.into_iter().map(|(_, wire, _)| Expression::ident(wire)))?;
        fsm_wire_decls.append(
            &mut eb_expr
                .into_iter()
                .zip_eq(&eb_input.port_decls())
                .map(|(ident, (_, shape))| Declaration::net(shape, ident.to_string()))
                .collect::<Vec<_>>(),
        );
        Ok((fsm_wire_decls, state_reg, [ip_assigns, eb_assigns].concat()))
    }

    /// Generates corresponding Verilog code for Expr.
    ///
    /// Returns required declarations and statements for expr output, and the expression tree
    /// indicating the expr output. If the expr has invalid width or mismatched type, returns `Err`.
    fn gen_expr(
        &self,
        expr: &Expr,
        ctx: &mut Context,
        cache: &mut HashMap<Expr, String>,
    ) -> VirgenResult<(Vec<Declaration>, Vec<Statement>, CompositeExpr<Expression>)> {
        if let Some(prefix) = cache.get(expr) {
            return Ok((
                Vec::new(),
                Vec::new(),
                CompositeExpr::from_typ(expr.port_decls(), prefix.clone())
                    .map(|(ident, _)| vir::Expression::ident(ident)),
            ));
        }

        match expr {
            Expr::X { .. } | Expr::Constant { .. } => {
                let literal = gen_expr_literal(expr).map(|s| {
                    if s.is_empty() {
                        vir::Expression::number("0".to_string())
                    } else if s.iter().all(|x| *x == LogicValue::False) {
                        vir::Expression::number(format!("{}'b0", s.len()))
                    } else if s.iter().all(|x| *x == LogicValue::X) {
                        vir::Expression::number(format!("{}'bx", s.len()))
                    } else {
                        vir::Expression::number(format!("{}'b{}", s.len(), s.to_string(),))
                    }
                });

                Ok((Vec::new(), Vec::new(), literal))
            }
            Expr::BinaryOp { op, lhs, rhs, span } => {
                self.gen_expr_binary_op(expr.clone(), *op, &lhs.into_expr(), &rhs.into_expr(), *span, ctx, cache)
            }
            Expr::Member { inner, index, .. } => {
                let (decls_for_inner, stmts_for_inner, exprs_for_inner) =
                    self.gen_expr(&inner.into_expr(), ctx, cache)?;

                match exprs_for_inner {
                    CompositeExpr::Struct(mut fields) => {
                        Ok((decls_for_inner, stmts_for_inner, fields.swap_remove(*index)))
                    }
                    _ => panic!("gen_expr: cannot index bits"),
                }
            }
            Expr::Concat { inner, .. } => self.gen_expr(&inner.into_expr(), ctx, cache),
            Expr::Fold { inner, typ_elt, func, init, span } => {
                self.gen_expr_fold(expr, *inner, typ_elt, &init.into_expr(), &func.into_function(), *span, ctx, cache)
            }
            Expr::Map { inner, typ_elt, func, span, len, .. } => {
                self.gen_expr_map(expr, *inner, typ_elt, &func.into_function(), *span, *len, ctx, cache)
            }
            Expr::Repeat { inner, count, .. } => {
                let (decls_for_inner, stmts_for_inner, exprs_for_inner) =
                    self.gen_expr(&inner.into_expr(), ctx, cache)?;
                let exprs = exprs_for_inner.map(|expr| expr.multiple_concat(*count));

                Ok((decls_for_inner, stmts_for_inner, exprs))
            }
            Expr::Var { name, .. } => {
                let prefix = join_options("_", [ctx.get_prefix(), name.clone()]).unwrap();
                let output = CompositeExpr::from_typ(expr.port_decls(), prefix.clone())
                    .map(|(ident, _)| vir::Expression::ident(ident));

                assert!(cache.insert(expr.clone(), prefix).is_none());

                Ok((Vec::new(), Vec::new(), output))
            }
            Expr::Not { inner, .. } => self.gen_expr_unary_op(UnaryOp::Negation, &inner.into_expr(), ctx, cache),
            // TODO: Use conditional expression?
            Expr::Cond { cond_expr_pair, default, span } => {
                assert!(!cond_expr_pair.is_empty(), "{span:?}");
                let mut decls = vec![];
                let mut stmts = vec![];
                let mut cond_body_expr_pairs = vec![];
                for (cond, body_expr) in cond_expr_pair {
                    let (mut decls_for_cond, mut stmts_for_cond, exprs_for_cond) =
                        self.gen_expr(&cond.into_expr(), ctx, cache)?;
                    decls.append(&mut decls_for_cond);
                    stmts.append(&mut stmts_for_cond);

                    let (mut decls_for_body_expr, mut stmts_for_body_expr, exprs_for_body_expr) =
                        self.gen_expr(&body_expr.into_expr(), ctx, cache)?;
                    decls.append(&mut decls_for_body_expr);
                    stmts.append(&mut stmts_for_body_expr);

                    cond_body_expr_pairs.push((exprs_for_cond, exprs_for_body_expr));
                }

                let (decls_for_output, exprs_for_output) = self.alloc_exprs(expr.clone(), ctx, cache)?;

                let x = cond_body_expr_pairs
                    .into_iter()
                    .map(|(cond, body)| {
                        Ok((cond.into_expr(), self.assign_exprs(exprs_for_output.clone(), body, *span)?))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let (decls_for_default, stmts_for_default, exprs_for_default) =
                    self.gen_expr(&default.into_expr(), ctx, cache)?;

                let stmt_for_conditional = Statement::Conditional(
                    x,
                    self.assign_exprs(exprs_for_output.clone(), exprs_for_default, *span)?,
                    *span,
                );

                let decls = [decls, decls_for_default, decls_for_output].concat();
                let stmts = [stmts, stmts_for_default, vec![stmt_for_conditional]].concat();

                Ok((decls, stmts, exprs_for_output))
            }
            Expr::Chunk { inner, .. } => self.gen_expr(&inner.into_expr(), ctx, cache),
            Expr::Get { inner, typ_elt, index, span } => {
                assert_eq!(clog2(inner.into_expr().width() / typ_elt.width()), index.into_expr().width());

                let (decls_for_inner, stmts_for_inner, exprs_for_inner) =
                    self.gen_expr_to_idents(&inner.into_expr(), *span, ctx, cache)?;
                let (decls_for_index, stmts_for_index, exprs_for_index) =
                    self.gen_expr(&index.into_expr(), ctx, cache)?;

                let (decls_for_output, exprs_for_output) = self.alloc_exprs(expr.clone(), ctx, cache)?;

                let exprs_for_rhs = self.indexing_exprs(
                    exprs_for_inner,
                    exprs_for_index.into_expr(),
                    typ_elt.clone(),
                    inner.into_expr().port_decls(),
                )?;

                let stmts_for_assign = self.assign_exprs(exprs_for_output.clone(), exprs_for_rhs, *span)?;

                let decls = [decls_for_inner, decls_for_index, decls_for_output].concat();
                let stmts = [stmts_for_inner, stmts_for_index, stmts_for_assign].concat();

                Ok((decls, stmts, exprs_for_output))
            }
            Expr::Clip { inner, from, size, typ_elt, span } => {
                let (decls_for_inner, stmts_for_inner, exprs_for_inner) =
                    self.gen_expr_to_idents(&inner.into_expr(), *span, ctx, cache)?;
                let (decls_for_from, stmts_for_from, exprs_for_from) = self.gen_expr(&from.into_expr(), ctx, cache)?;

                let (decls_for_output, exprs_for_output) = self.alloc_exprs(expr.clone(), ctx, cache)?;

                let exprs_for_elts = self.range_indexing_exprs(
                    exprs_for_inner,
                    exprs_for_from.into_expr(),
                    vir::Expression::number(size.to_string()),
                    typ_elt.clone(),
                )?;
                let stmts_for_assign = self.assign_exprs(exprs_for_output.clone(), exprs_for_elts, *span)?;

                let decls = [decls_for_inner, decls_for_from, decls_for_output].concat();
                let stmts = [stmts_for_inner, stmts_for_from, stmts_for_assign].concat();

                Ok((decls, stmts, exprs_for_output))
            }
            Expr::Append { lhs, rhs, .. } => {
                let (decls_for_lhs, stmts_for_lhs, exprs_for_lhs) = self.gen_expr(&lhs.into_expr(), ctx, cache)?;
                let (decls_for_rhs, stmts_for_rhs, exprs_for_rhs) = self.gen_expr(&rhs.into_expr(), ctx, cache)?;

                let decls = [decls_for_lhs, decls_for_rhs].concat();
                let stmts = [stmts_for_lhs, stmts_for_rhs].concat();
                let exprs = exprs_for_lhs.zip(exprs_for_rhs).map(|(lhs, rhs)| rhs.concat(lhs));

                Ok((decls, stmts, exprs))
            }
            Expr::Zip { inner, span, .. } => {
                let (decls_for_inner, stmts_for_inner, exprs_for_inner) = inner
                    .iter()
                    .map(|expr_id| self.gen_expr(&expr_id.into_expr(), ctx, cache).expect("gen_expr: zip"))
                    .fold(
                        (Vec::new(), Vec::new(), Vec::new()),
                        |(mut acc_decls, mut acc_stmts, mut acc_exprs), (decls, stmts, exprs)| {
                            acc_decls.push(decls);
                            acc_stmts.push(stmts);
                            acc_exprs.push(exprs);
                            (acc_decls, acc_stmts, acc_exprs)
                        },
                    );
                let (decls_for_output, exprs_for_output) = self.alloc_exprs(expr.clone(), ctx, cache)?;

                let exprs_for_zipped = CompositeExpr::Struct(exprs_for_inner);
                let stmts_for_assign = self.assign_exprs(exprs_for_output.clone(), exprs_for_zipped, *span)?;

                let decls = [decls_for_inner.concat(), decls_for_output].concat();
                let stmts = [stmts_for_inner.concat(), stmts_for_assign].concat();

                Ok((decls, stmts, exprs_for_output))
            }
            Expr::Struct { inner, .. } => {
                let (decls, stmts, exprs) = inner
                    .iter()
                    .map(|(_, inner)| self.gen_expr(&inner.into_expr(), ctx, cache).unwrap())
                    .fold((Vec::new(), Vec::new(), Vec::new()), |mut acc, mut x| {
                        acc.0.append(&mut x.0);
                        acc.1.append(&mut x.1);
                        acc.2.push(x.2);
                        acc
                    });

                Ok((decls, stmts, CompositeExpr::Struct(exprs)))
            }
            Expr::Repr { inner, .. } => self.gen_expr(&inner.into_expr(), ctx, cache),
            Expr::Set { inner, index, elt, span, .. } => {
                assert_eq!(clog2(inner.into_expr().width() / elt.into_expr().width()), index.into_expr().width());
                let (decls_for_inner, stmts_for_inner, exprs_for_inner) =
                    self.gen_expr(&inner.into_expr(), ctx, cache)?;
                let (decls_for_index, stmts_for_index, exprs_for_index) =
                    self.gen_expr(&index.into_expr(), ctx, cache)?;
                let (decls_for_elt, stmts_for_elt, exprs_for_elt) = self.gen_expr(&elt.into_expr(), ctx, cache)?;

                let (decls_for_output, exprs_for_output) = self.alloc_exprs(expr.clone(), ctx, cache)?;
                let stmts_for_assign = self.assign_exprs(exprs_for_output.clone(), exprs_for_inner, *span)?;

                let exprs_for_output_elt = self.indexing_exprs(
                    exprs_for_output.clone(),
                    exprs_for_index.into_expr(),
                    elt.into_expr().port_decls(),
                    expr.port_decls(),
                )?;
                let stmts_for_assign_elt = self.assign_exprs(exprs_for_output_elt, exprs_for_elt, *span)?;

                let decls = [decls_for_inner, decls_for_index, decls_for_elt, decls_for_output].concat();
                let stmts =
                    [stmts_for_inner, stmts_for_index, stmts_for_elt, stmts_for_assign, stmts_for_assign_elt].concat();

                Ok((decls, stmts, exprs_for_output))
            }
            Expr::SetRange { inner, typ_elt, index, elts, span, .. } => {
                let (decls_for_inner, stmts_for_inner, exprs_for_inner) =
                    self.gen_expr(&inner.into_expr(), ctx, cache)?;
                let (decls_for_index, stmts_for_index, exprs_for_index) =
                    self.gen_expr(&index.into_expr(), ctx, cache)?;
                let (decls_for_elts, stmts_for_elts, exprs_for_elts) = self.gen_expr(&elts.into_expr(), ctx, cache)?;

                let (decls_for_output, exprs_for_output) = self.alloc_exprs(expr.clone(), ctx, cache)?;
                let stmts_for_assign = self.assign_exprs(exprs_for_output.clone(), exprs_for_inner, *span)?;

                let elts_count = elts.into_expr().width() / typ_elt.width();

                let exprs_for_output_elts = self.range_indexing_exprs(
                    exprs_for_output.clone(),
                    exprs_for_index.into_expr(),
                    vir::Expression::number(elts_count.to_string()),
                    typ_elt.clone(),
                )?;
                let stmts_for_assign_elts = self.assign_exprs(exprs_for_output_elts, exprs_for_elts, *span)?;

                let decls = [decls_for_inner, decls_for_index, decls_for_elts, decls_for_output].concat();
                let stmts = [stmts_for_inner, stmts_for_index, stmts_for_elts, stmts_for_assign, stmts_for_assign_elts]
                    .concat();

                Ok((decls, stmts, exprs_for_output))
            }
            Expr::Case { case_expr, case_items, default, span } => {
                let (decls_for_case_expr, stmts_for_case_expr, exprs_for_case_expr) =
                    self.gen_expr(&case_expr.into_expr(), ctx, cache)?;

                let (
                    decls_for_case_conds,
                    stmts_for_case_conds,
                    exprs_for_case_conds,
                    decls_for_case_stmts,
                    stmts_for_case_stmts,
                    exprs_for_case_stmts,
                ) = case_items
                    .iter()
                    .map(|(cond, expr)| {
                        (
                            self.gen_expr(&cond.into_expr(), ctx, cache).unwrap(),
                            self.gen_expr(&expr.into_expr(), ctx, cache).unwrap(),
                        )
                    })
                    .fold((Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()), |mut acc, x| {
                        acc.0.push(x.0 .0.clone());
                        acc.1.push(x.0 .1.clone());
                        acc.2.push(x.0 .2.clone());
                        acc.3.push(x.1 .0.clone());
                        acc.4.push(x.1 .1.clone());
                        acc.5.push(x.1 .2);
                        acc
                    });

                let (decls_for_default, stmts_for_default, exprs_for_default) =
                    (*default).map_or((None, None, None), |d| {
                        let (decls, stmts, exprs) = self.gen_expr(&d.into_expr(), ctx, cache).unwrap();
                        (Some(decls), Some(stmts), Some(exprs))
                    });

                let decls_for_default = decls_for_default.unwrap_or_default();
                let stmts_for_default = stmts_for_default.unwrap_or_default();

                let (decls_for_output, exprs_for_output) = self.alloc_exprs(expr.clone(), ctx, cache)?;

                let stmt_for_case = Statement::Case(
                    exprs_for_case_expr.into_expr(),
                    itertools::izip!(exprs_for_case_conds, exprs_for_case_stmts)
                        .map(|(expr_cond, expr_stmt)| {
                            (
                                expr_cond.into_expr(),
                                self.assign_exprs(exprs_for_output.clone(), expr_stmt, *span).unwrap(),
                            )
                        })
                        .collect::<Vec<_>>(),
                    exprs_for_default
                        .map(|exprs| self.assign_exprs(exprs_for_output.clone(), exprs, *span).unwrap())
                        .unwrap_or_default(),
                    *span,
                );

                let decls = [
                    decls_for_case_expr,
                    decls_for_case_conds.concat(),
                    decls_for_case_stmts.concat(),
                    decls_for_default,
                    decls_for_output,
                ]
                .concat();

                let stmts = [
                    stmts_for_case_expr,
                    stmts_for_case_conds.concat(),
                    stmts_for_case_stmts.concat(),
                    stmts_for_default,
                    vec![stmt_for_case],
                ]
                .concat();

                Ok((decls, stmts, exprs_for_output))
            }
            Expr::TreeFold { inner, op, lhs, rhs, acc, span } => self.gen_expr_tree_fold(
                expr,
                &inner.into_expr(),
                &op.into_expr(),
                &lhs.into_expr(),
                &rhs.into_expr(),
                &acc.into_expr(),
                *span,
                ctx,
                cache,
            ),
            Expr::ConcatArray { inner, elt_typ, span, .. } => {
                let (decls_for_output, exprs_for_output) = self.alloc_exprs(expr.clone(), ctx, cache)?;
                let mut assign_decls = vec![];
                let mut assign_stmts = vec![];

                for (i, expr_elt) in inner.iter().enumerate() {
                    let (decls_for_elt, stmts_for_elt, exprs_for_elt) =
                        self.gen_expr(&expr_elt.into_expr(), ctx, cache)?;
                    let stmts_for_assign = self.assign_exprs(
                        self.indexing_exprs(
                            exprs_for_output.clone(),
                            vir::Expression::number(i.to_string()),
                            elt_typ.clone(),
                            expr.port_decls(),
                        )?,
                        exprs_for_elt,
                        *span,
                    )?;

                    assign_decls.extend(decls_for_elt);
                    assign_stmts.extend([stmts_for_elt, stmts_for_assign].concat());
                }

                let decls = [decls_for_output, assign_decls].concat();
                let stmts = assign_stmts;

                Ok((decls, stmts, exprs_for_output))
            }
            Expr::Range { .. } => todo!(),
            Expr::Cast { from, to, span } => {
                let from_typ = from.into_expr().port_decls();

                let PortDecls::Bits(from_typ) = from_typ else { panic!() };

                assert!(from_typ.dim() < 2);

                let (decls_for_from, stmts_for_from, exprs_for_from) =
                    self.gen_expr_to_idents(&from.into_expr(), *span, ctx, cache)?;
                let exprs_for_from = exprs_for_from.into_expr();

                let (decls_for_output, exprs_for_output) = self.alloc_exprs(expr.clone(), ctx, cache)?;
                let exprs_for_output_inner = exprs_for_output.clone().into_expr();

                let exprs_for_elts = match from_typ.width().cmp(&to.width()) {
                    std::cmp::Ordering::Less => vir::Expression::number("1'b0".to_string())
                        .multiple_concat(to.width() - from_typ.width())
                        .concat(exprs_for_from),
                    std::cmp::Ordering::Equal => exprs_for_from,
                    std::cmp::Ordering::Greater => exprs_for_from.with_range(Range::new_range(
                        vir::Expression::number("0".to_string()),
                        vir::Expression::number(to.width().to_string()),
                    )),
                };

                let stmts_for_assign =
                    vir::Statement::blocking_assignment(exprs_for_output_inner, exprs_for_elts, *span);

                Ok((
                    [decls_for_from, decls_for_output].concat(),
                    [stmts_for_from, vec![stmts_for_assign]].concat(),
                    exprs_for_output,
                ))
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn gen_expr_tree_fold(
        &self,
        expr: &Expr,
        inner: &Expr,
        op: &Expr,
        lhs: &Expr,
        rhs: &Expr,
        acc: &Expr,
        span: rustc_span::Span,
        ctx: &mut Context,
        cache: &mut HashMap<Expr, String>,
    ) -> VirgenResult<(Vec<Declaration>, Vec<Statement>, CompositeExpr<Expression>)> {
        let num_elts = inner.width() / lhs.width();

        let (decls_for_inner, stmts_for_inner, exprs_for_inner) = self.gen_expr_to_idents(inner, span, ctx, cache)?;

        // decls for outer for loop
        let outer_loop_int = ctx.alloc_int_id_with_prefix();
        let outer_loop_int_variable = format!("{outer_loop_int}_level");
        let outer_loop_count = clog2(num_elts);
        let decl_for_outer_loop_count = Declaration::integer(outer_loop_int_variable.clone());

        let tree_fold_prefix = ctx.alloc_temp_id_with_prefix();
        let decl_acc_reg = Declaration::reg_with_typ(inner.port_decls(), Some(format!("{tree_fold_prefix}_acc")));

        let mut ctx = Context::new();

        ctx.enter_scope(tree_fold_prefix.clone());

        let (decls_for_acc, stmts_for_acc, exprs_for_acc) = self.gen_expr(acc, &mut ctx, cache)?;
        let stmts_for_acc_init = self.assign_exprs(exprs_for_acc.clone(), exprs_for_inner, span)?;

        let inner_loop_int = ctx.alloc_int_id_with_prefix();
        let decl_for_inner_loop_count = Declaration::integer(inner_loop_int.clone());

        let decl_lhs_reg = Declaration::reg_with_typ(lhs.port_decls(), Some(format!("{tree_fold_prefix}_lhs")));
        let (decls_for_lhs, stmts_for_lhs, exprs_for_lhs) = self.gen_expr(lhs, &mut ctx, cache)?;
        let stmts_for_lhs_expr = self.assign_exprs(
            exprs_for_lhs,
            self.indexing_exprs(
                exprs_for_acc.clone(),
                // idx 2*i
                vir::Expression::binary(
                    BinaryOp::Mul,
                    vir::Expression::number(2.to_string()),
                    vir::Expression::ident(inner_loop_int.clone()),
                ),
                lhs.port_decls(),
                inner.port_decls(),
            )?,
            span,
        )?;

        let decl_rhs_reg = Declaration::reg_with_typ(rhs.port_decls(), Some(format!("{tree_fold_prefix}_rhs")));
        let (decls_for_rhs, stmts_for_rhs, exprs_for_rhs) = self.gen_expr(rhs, &mut ctx, cache)?;
        let stmts_for_rhs_expr = self.assign_exprs(
            exprs_for_rhs,
            self.indexing_exprs(
                exprs_for_acc.clone(),
                // idx 2*i + 1
                vir::Expression::binary(
                    BinaryOp::Add,
                    vir::Expression::binary(
                        BinaryOp::Mul,
                        vir::Expression::number(2.to_string()),
                        vir::Expression::ident(inner_loop_int.clone()),
                    ),
                    vir::Expression::number(1.to_string()),
                ),
                rhs.port_decls(),
                inner.port_decls(),
            )?,
            span,
        )?;

        let (decls_for_loop_op, stmts_for_loop_op, exprs_for_loop_op) = self.gen_expr(op, &mut ctx, cache)?;
        let stmt_for_loop_body_operation = self.assign_exprs(
            self.indexing_exprs(
                exprs_for_acc.clone(),
                vir::Expression::ident(inner_loop_int.clone()),
                lhs.port_decls(),
                inner.port_decls(),
            )?,
            exprs_for_loop_op,
            span,
        )?;

        let stmt_for_inner_loop = Statement::Loop(
            inner_loop_int,
            vir::Expression::binary(
                BinaryOp::Div,
                vir::Expression::number(num_elts.to_string()),
                vir::Expression::binary(
                    BinaryOp::ShiftLeft,
                    vir::Expression::number(1.to_string()),
                    vir::Expression::binary(
                        BinaryOp::Add,
                        vir::Expression::ident(outer_loop_int_variable.clone()),
                        vir::Expression::number(1.to_string()),
                    ),
                ),
            ),
            [stmts_for_lhs_expr, stmts_for_rhs_expr, stmts_for_loop_op, stmt_for_loop_body_operation].concat(),
            span,
        );

        let stmt_for_outer_loop = Statement::Loop(
            outer_loop_int_variable,
            vir::Expression::number(outer_loop_count.to_string()),
            vec![stmt_for_inner_loop],
            span,
        );

        let decls_for_loop = [
            vec![decl_for_outer_loop_count, decl_for_inner_loop_count],
            decl_acc_reg,
            decl_lhs_reg,
            decl_rhs_reg,
            decls_for_acc,
            decls_for_lhs,
            decls_for_rhs,
            decls_for_loop_op,
        ]
        .concat();

        let fold_prefix = ctx.alloc_temp_id_with_prefix();
        let expr_for_fold_output = codegen::CompositeExpr::from_typ(lhs.port_decls(), fold_prefix.clone())
            .map(|(ident, _)| vir::Expression::ident(ident));

        let decl_for_fold_output = Declaration::reg_with_typ(lhs.port_decls(), Some(fold_prefix.clone()));
        let stmt_epilogue = self.assign_exprs(
            expr_for_fold_output.clone(),
            self.indexing_exprs(
                exprs_for_acc,
                vir::Expression::number(0.to_string()),
                lhs.port_decls(),
                inner.port_decls(),
            )?,
            span,
        )?;

        let decls = [decls_for_inner, decls_for_loop, decl_for_fold_output].concat();
        let stmts = [
            stmts_for_acc,
            stmts_for_lhs,
            stmts_for_rhs,
            stmts_for_inner,
            stmts_for_acc_init,
            vec![stmt_for_outer_loop],
            stmt_epilogue,
        ]
        .concat();

        cache.insert(expr.clone(), fold_prefix);

        Ok((decls, stmts, expr_for_fold_output))
    }

    #[allow(clippy::too_many_arguments)]
    fn gen_expr_fold(
        &self,
        expr: &Expr,
        inner: ExprId,
        typ_elt: &PortDecls,
        init: &Expr,
        func: &FunctionBuilder<'tcx>,
        span: rustc_span::Span,
        ctx: &mut Context,
        cache: &mut HashMap<Expr, String>,
    ) -> VirgenResult<(Vec<Declaration>, Vec<Statement>, CompositeExpr<Expression>)> {
        let loop_int = ctx.alloc_int_id();
        let loop_var = join_options("_", [ctx.get_prefix(), Some(loop_int.clone())]).unwrap();
        let loop_count = inner.into_expr().width() / typ_elt.width();
        let decl_for_loop_int = Declaration::integer(loop_var.clone());

        let (decls_for_inner, stmts_for_inner, _) = self.gen_expr_to_idents(&inner.into_expr(), span, ctx, cache)?;
        let (decls_for_init, stmts_for_init, init_expr) = self.gen_expr(init, ctx, cache)?;

        let (mut decls_for_captures, mut stmts_for_captures) = (vec![], vec![]);

        if let Some(upvars) = func.expect_fn().upvars.as_ref() {
            for (_, captured) in upvars.iter() {
                if let PureValue::Expr(upvar) = captured {
                    let (mut decls_for_upvar, mut stmts_for_upvar, _) =
                        self.gen_expr(&upvar.into_expr(), ctx, cache)?;
                    decls_for_captures.append(&mut decls_for_upvar);
                    stmts_for_captures.append(&mut stmts_for_upvar);
                }
            }
        }

        let int_var = Expr::Var { name: loop_int.into(), typ: PortDecls::unsigned_bits(clog2(loop_count)), span };
        let inner_indexed = Expr::Get { inner, typ_elt: typ_elt.clone(), index: ExprId::alloc_expr(int_var), span };

        let fold_body_temp_id = ctx.alloc_temp_id();
        let fold_body_temp_id_with_prefix =
            join_options("_", [ctx.get_prefix(), Some(fold_body_temp_id.clone())]).unwrap();
        let decl_for_fold_acc = Declaration::reg_with_typ(init.port_decls(), Some(fold_body_temp_id_with_prefix));

        let acc_var = Expr::Var { name: Some(fold_body_temp_id), typ: init.port_decls(), span };

        let (decls_for_fold_output, exprs_for_fold_output) = self.alloc_exprs(expr.clone(), ctx, cache)?;

        let (decls_for_acc, stmts_for_acc, exprs_for_acc) = self.gen_expr(&acc_var, ctx, cache)?;
        let stmt_acc_initialization = self.assign_exprs(exprs_for_acc.clone(), init_expr, span)?;

        let (expr_folded, mut displays) = func.build(
            self.tcx,
            vec![
                PureValue::Expr(ExprId::alloc_expr(acc_var)),
                PureValue::Expr(inner_indexed.alloc_with_fsm_cache(&mut ctx.fsm_cache)),
            ],
            &mut ctx.fsm_cache,
        );
        ctx.displays.append(&mut displays);

        let expr_folded = &expr_folded.into_expr();
        let (decls_for_loop_body, stmts_for_loop_body, exprs_for_loop_body) = self.gen_expr(expr_folded, ctx, cache)?;
        let stmt_for_loop_body_output = self.assign_exprs(exprs_for_acc.clone(), exprs_for_loop_body, span)?;

        let stmt_for_loop = Statement::Loop(
            loop_var,
            vir::Expression::number(loop_count.to_string()),
            [stmts_for_loop_body, stmt_for_loop_body_output].concat(),
            span,
        );

        let decls_for_loop =
            [[decl_for_fold_acc, vec![decl_for_loop_int], decls_for_fold_output, decls_for_loop_body, decls_for_acc]
                .concat()]
            .concat();

        // let fold_prefix = ctx.alloc_temp_id_with_prefix();
        // let decl_epilogue_reg =
        //     Declaration::reg_with_typ(init.port_decls(), Some(fold_prefix.clone()));
        let stmt_epilogue = self.assign_exprs(exprs_for_fold_output.clone(), exprs_for_acc, span)?;

        let decls = [decls_for_captures, decls_for_inner, decls_for_init, decls_for_loop].concat();
        let stmts = [
            stmts_for_captures,
            stmts_for_inner,
            stmts_for_acc,
            stmts_for_init,
            stmt_acc_initialization,
            vec![stmt_for_loop],
            stmt_epilogue,
        ]
        .concat();

        Ok((decls, stmts, exprs_for_fold_output))
    }

    fn gen_expr_unary_op(
        &self,
        op: UnaryOp,
        inner: &Expr,
        ctx: &mut Context,
        cache: &mut HashMap<Expr, String>,
    ) -> VirgenResult<(Vec<Declaration>, Vec<Statement>, CompositeExpr<Expression>)> {
        let (decls_for_inner, stmts_for_inner, exprs_for_inner) = self.gen_expr(inner, ctx, cache)?;

        let expr = vir::Expression::unary(op, exprs_for_inner.into_expr());
        let exprs = CompositeExpr::Bits(expr);

        Ok((decls_for_inner, stmts_for_inner, exprs))
    }

    #[allow(clippy::too_many_arguments)]
    fn gen_expr_binary_op(
        &self,
        expr: Expr,
        op: BinaryOp,
        lhs: &Expr,
        rhs: &Expr,
        span: rustc_span::Span,
        ctx: &mut Context,
        cache: &mut HashMap<Expr, String>,
    ) -> VirgenResult<(Vec<Declaration>, Vec<Statement>, CompositeExpr<Expression>)> {
        let (decls_for_lhs, stmts_for_lhs, exprs_for_lhs) = self.gen_expr(lhs, ctx, cache)?;
        let (decls_for_rhs, stmts_for_rhs, exprs_for_rhs) = self.gen_expr(rhs, ctx, cache)?;
        let (decls_for_output, exprs_for_output) = self.alloc_exprs(expr, ctx, cache)?;
        let expr = match op {
            BinaryOp::EqArithmetic => exprs_for_lhs
                .into_iter()
                .zip_eq(&exprs_for_rhs)
                .map(|(l, r)| Expression::binary(BinaryOp::EqArithmetic, l, r))
                .reduce(|acc, elt| Expression::binary(BinaryOp::And, acc, elt))
                .unwrap(),
            BinaryOp::NeArithmetic => exprs_for_lhs
                .into_iter()
                .zip_eq(&exprs_for_rhs)
                .map(|(l, r)| Expression::binary(BinaryOp::NeArithmetic, l, r))
                .reduce(|acc, elt| Expression::binary(BinaryOp::Or, acc, elt))
                .unwrap(),
            _ => Expression::binary(op, exprs_for_lhs.into_expr(), exprs_for_rhs.into_expr()),
        };
        let exprs = CompositeExpr::Bits(expr);
        let stmts_for_assignment = self.assign_exprs(exprs_for_output.clone(), exprs, span)?;

        let decls = [decls_for_lhs, decls_for_rhs, decls_for_output].concat();
        let stmts = [stmts_for_lhs, stmts_for_rhs, stmts_for_assignment].concat();

        Ok((decls, stmts, exprs_for_output))
    }

    #[allow(clippy::too_many_arguments)]
    fn gen_expr_map(
        &self,
        expr: &Expr,
        inner: ExprId,
        typ_elt: &PortDecls,
        func: &FunctionBuilder<'tcx>,
        span: rustc_span::Span,
        len: usize,
        ctx: &mut Context,
        cache: &mut HashMap<Expr, String>,
    ) -> VirgenResult<(Vec<Declaration>, Vec<Statement>, CompositeExpr<Expression>)> {
        let loop_int = ctx.alloc_int_id();
        let loop_var = join_options("_", [ctx.get_prefix(), Some(loop_int.clone())]).unwrap();
        let loop_count = len;
        let decl_for_loop_int = Declaration::integer(loop_var.clone());

        let (decls_for_inner, stmts_for_inner, _) = self.gen_expr_to_idents(&inner.into_expr(), span, ctx, cache)?;
        let (decls_for_loop_output, exprs_for_loop_output) = self.alloc_exprs(expr.clone(), ctx, cache)?;

        let (mut decls_for_captures, mut stmts_for_captures) = (vec![], vec![]);

        if let Some(upvars) = func.expect_fn().upvars.as_ref() {
            for (_, captured) in upvars.iter() {
                if let PureValue::Expr(upvar) = captured {
                    let (mut decls_for_upvar, mut stmts_for_upvar, _) =
                        self.gen_expr(&upvar.into_expr(), ctx, cache)?;
                    decls_for_captures.append(&mut decls_for_upvar);
                    stmts_for_captures.append(&mut stmts_for_upvar);
                }
            }
        }

        let variable = Expr::Var { name: loop_int.into(), typ: PortDecls::unsigned_bits(clog2(loop_count)), span };
        let inner_indexed = Expr::Get { inner, typ_elt: typ_elt.clone(), index: ExprId::alloc_expr(variable), span };
        let (expr_mapped, mut displays) = func.build(
            self.tcx,
            vec![PureValue::Expr(inner_indexed.alloc_with_fsm_cache(&mut ctx.fsm_cache))],
            &mut ctx.fsm_cache,
        );
        ctx.displays.append(&mut displays);
        let expr_mapped = &expr_mapped.into_expr();
        let (decls_for_loop_body, stmts_for_loop_body, exprs_for_loop_body) = self.gen_expr(expr_mapped, ctx, cache)?;

        let stmts_for_loop_body_output = self.assign_exprs(
            self.indexing_exprs(
                exprs_for_loop_output.clone(),
                vir::Expression::ident(loop_var.clone()),
                expr_mapped.port_decls(),
                expr.port_decls(),
            )?,
            exprs_for_loop_body,
            span,
        )?;

        let decls_for_loop = [vec![decl_for_loop_int], decls_for_loop_output, decls_for_loop_body].concat();

        let stmt_for_loop = Statement::Loop(
            loop_var,
            vir::Expression::number(loop_count.to_string()),
            [stmts_for_loop_body, stmts_for_loop_body_output].concat(),
            span,
        );

        let decls = [decls_for_captures, decls_for_inner, decls_for_loop].concat();
        let stmts = [stmts_for_captures, stmts_for_inner, vec![stmt_for_loop]].concat();

        Ok((decls, stmts, exprs_for_loop_output))
    }

    fn gen_expr_to_idents(
        &self,
        expr: &Expr,
        span: rustc_span::Span,
        ctx: &mut Context,
        cache: &mut HashMap<Expr, String>,
    ) -> VirgenResult<(Vec<Declaration>, Vec<Statement>, CompositeExpr<Expression>)> {
        let (mut decls, mut stmts, exprs) = self.gen_expr(expr, ctx, cache)?;

        // If every expressions are idents, return immediately
        if exprs.iter().all(|expr| expr.is_identifier()) {
            return Ok((decls, stmts, exprs));
        }

        let (mut decls_for_alloc, new_exprs) = self.alloc_exprs(expr.clone(), ctx, &mut HashMap::new())?;
        let mut stmts_for_assign = self.assign_exprs(new_exprs.clone(), exprs, span)?;

        decls.append(&mut decls_for_alloc);
        stmts.append(&mut stmts_for_assign);

        Ok((decls, stmts, new_exprs))
    }

    fn indexing_exprs(
        &self,
        exprs: CompositeExpr<Expression>,
        index: Expression,
        typ_elt: PortDecls,
        typ: PortDecls,
    ) -> VirgenResult<CompositeExpr<Expression>> {
        let exprs_for_elt = exprs.zip(typ_elt.into()).zip(typ.into()).map(|((expr, (_, shape_elt)), (_, shape))| {
            // `gen_expr()` considers all `Expr`s with width 1 as single bit, not an array.
            if shape.width() > 1 {
                expr.with_range(Range::new_range(
                    vir::Expression::binary(
                        BinaryOp::Mul,
                        index.clone(),
                        vir::Expression::number(shape_elt.width().to_string()),
                    ),
                    vir::Expression::number(shape_elt.width().to_string()),
                ))
            } else {
                expr
            }
        });

        Ok(exprs_for_elt)
    }

    fn range_indexing_exprs(
        &self,
        exprs: CompositeExpr<Expression>,
        base: Expression,
        offset: Expression,
        typ_elt: PortDecls,
    ) -> VirgenResult<CompositeExpr<Expression>> {
        let exprs = exprs.zip(typ_elt.into()).map(|(expr, (_, shape))| {
            expr.with_range(Range::new_range(
                vir::Expression::binary(
                    BinaryOp::Mul,
                    base.clone(),
                    vir::Expression::number(shape.width().to_string()),
                ),
                vir::Expression::binary(
                    BinaryOp::Mul,
                    offset.clone(),
                    vir::Expression::number(shape.width().to_string()),
                ),
            ))
        });

        Ok(exprs)
    }

    fn alloc_exprs(
        &self,
        expr: Expr,
        ctx: &mut Context,
        cache: &mut HashMap<Expr, String>,
    ) -> VirgenResult<(Vec<Declaration>, CompositeExpr<Expression>)> {
        let typ = expr.port_decls();
        let prefix = ctx.alloc_temp_id_with_prefix();
        let exprs = CompositeExpr::from_typ(typ, prefix.clone());

        let decls = exprs.iter().map(|(ident, shape)| vir::Declaration::reg(shape, ident)).collect::<Vec<_>>();
        let exprs = exprs.map(|(ident, _)| vir::Expression::ident(ident));

        cache.insert(expr, prefix);

        Ok((decls, exprs))
    }

    fn assign_exprs(
        &self,
        lhs: CompositeExpr<Expression>,
        rhs: CompositeExpr<Expression>,
        span: rustc_span::Span,
    ) -> VirgenResult<Vec<Statement>> {
        let stmts = lhs
            .zip(rhs)
            .iter()
            .map(|(lvalue, expr)| Statement::blocking_assignment(lvalue, expr, span))
            .collect::<Vec<_>>();

        Ok(stmts)
    }

    fn cont_assign_exprs<It1: IntoIterator<Item = Expression>, It2: IntoIterator<Item = Expression>>(
        &self,
        lhs: It1,
        rhs: It2,
    ) -> VirgenResult<Vec<ContinuousAssign>> {
        let conts = lhs
            .into_iter()
            .zip_eq(rhs.into_iter())
            .map(|(lvalue, expr)| ContinuousAssign(lvalue, expr))
            .collect::<Vec<_>>();

        Ok(conts)
    }

    // TODO: fix
    fn gen_system_task(
        &self,
        SystemTask { kind, fstring, path_cond, args, span }: SystemTask,
        ctx: &mut Context,
        cache: &mut HashMap<Expr, String>,
    ) -> VirgenResult<(Vec<Declaration>, Vec<Statement>)> {
        if !self.options.system_task {
            return Ok((vec![], vec![]));
        }

        match kind {
            SystemTaskKind::Display => {
                let (decls_for_cond, stmts_for_cond, cond) = if let Some(cond) = path_cond {
                    let (decls, stmts, cond) = self.gen_expr(&cond.into_expr(), ctx, cache)?;
                    (decls, stmts, Some(cond.into_expr()))
                } else {
                    (vec![], vec![], None)
                };

                let (mut decls_for_args, mut stmts_for_args, mut arg_exprs) = (vec![], vec![], vec![]);

                for arg in args {
                    let (decls, stmts, arg_expr) = self.gen_expr(&arg.into_expr(), ctx, cache)?;
                    decls_for_args.extend(decls);
                    stmts_for_args.extend(stmts);
                    arg_exprs.push(arg_expr.into_expr());
                }

                let display_stmt = Statement::Display(fstring, arg_exprs, span);

                let display_stmt = if let Some(cond) = cond {
                    Statement::Conditional(vec![(cond, vec![display_stmt])], vec![], span)
                } else {
                    display_stmt
                };

                Ok((
                    [decls_for_cond, decls_for_args].concat(),
                    [stmts_for_cond, stmts_for_args, vec![display_stmt]].concat(),
                ))
            }
            SystemTaskKind::Assert { cond } => {
                let (decls_for_assert_cond, stmts_for_assert_cond, assert_cond) =
                    self.gen_expr(&cond.into_expr(), ctx, cache)?;

                let assert_cond = assert_cond.into_expr();
                let assert_cond =
                    vir::Expression::binary(BinaryOp::NeStrict, assert_cond, vir::Expression::ident("1".to_string()));

                let (decls_for_cond, stmts_for_cond, cond) = if let Some(cond) = path_cond {
                    let (decls, stmts, cond) = self.gen_expr(&cond.into_expr(), ctx, cache)?;
                    (decls, stmts, Some(cond.into_expr()))
                } else {
                    (vec![], vec![], None)
                };

                let cond = if let Some(cond) = cond {
                    vir::Expression::binary(BinaryOp::And, cond, assert_cond)
                } else {
                    assert_cond
                };

                let (mut decls_for_args, mut stmts_for_args, mut arg_exprs) = (vec![], vec![], vec![]);

                for arg in args {
                    let (decls, stmts, arg_expr) = self.gen_expr(&arg.into_expr(), ctx, cache)?;
                    decls_for_args.extend(decls);
                    stmts_for_args.extend(stmts);
                    arg_exprs.push(arg_expr.into_expr());
                }

                let display_stmt = Statement::Display(format!("ERROR: {fstring}"), arg_exprs, span);

                let assert_stmt =
                    Statement::Conditional(vec![(cond, vec![display_stmt, Statement::Fatal])], vec![], span);

                Ok((
                    [decls_for_assert_cond, decls_for_cond, decls_for_args].concat(),
                    [stmts_for_cond, stmts_for_args, stmts_for_assert_cond, vec![assert_stmt]].concat(),
                ))
            }
        }
    }

    fn is_closure(&self) -> bool {
        self.upvars.is_some()
    }
}

fn gen_var_arr_state_init(
    state_reg: &CompositeExpr<Declaration>,
    ctx: &mut Context,
    fsm_function_builder: &FunctionBuilder<'_>,
) -> Vec<vir::ModuleItem> {
    let (mut decls, mut stmts) = (Vec::new(), Vec::new());
    let mut int_name = None;
    state_reg.iter().filter(|reg| reg.shape().dim() > 1).for_each(|reg| {
        let shape = reg.shape();
        let reg_name = reg.ident().to_string();

        let int_name = int_name.get_or_insert(ctx.alloc_int_id());
        let body = vec![Statement::blocking_assignment(
            vir::Expression::ident(reg_name)
                .with_range(vir::Range::new_index(vir::Expression::ident(int_name.clone()))),
            vir::Expression::number("0".to_string()),
            fsm_function_builder.span,
        )];

        stmts.push(Statement::Loop(
            int_name.clone(),
            vir::Expression::number(shape.get(0).to_string()),
            body,
            fsm_function_builder.span,
        ));
    });
    if let Some(int_name) = int_name {
        decls.push(Declaration::integer(int_name));
    }
    let mut module_items = vec![];
    if !decls.is_empty() {
        module_items.push(vir::ModuleItem::Declarations(decls));
    }
    if !stmts.is_empty() {
        module_items.push(vir::ModuleItem::AlwaysConstruct("initial".to_string(), stmts));
    }
    module_items
}

fn filter_nonzero(expr: CompositeExpr<Expression>, typ: PortDecls) -> Vec<Expression> {
    expr.into_iter()
        .zip_eq(typ.iter_with_zero_width(None))
        .filter_map(|(expr, (_, shape))| if shape.width() > 0 { Some(expr) } else { None })
        .collect::<Vec<_>>()
}
