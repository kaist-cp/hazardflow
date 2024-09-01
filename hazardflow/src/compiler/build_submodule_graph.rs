//! This module constructs the submodule graph of a given module.
//!
//! TODO: remove all the expr constructing logic by hand
use std::collections::HashMap;

use itertools::Itertools;
use rustc_middle::mir::BorrowKind;
use rustc_middle::thir::{self, ClosureExpr, ExprId, ExprKind, Thir};
use rustc_middle::ty::{EarlyBinder, Generics, Instance, ParamEnv, Ty, TyCtxt};
use rustc_type_ir::fold::TypeFoldable;

use super::*;
use crate::utils::*;

/// Construct the submodule graph of a given function
///
/// It traverses the function's thir and constructs the submodule graph.
struct ModuleGraphConstructor<'tcx, 'a> {
    instance: Instance<'tcx>,

    tcx: TyCtxt<'tcx>,

    sig: &'a ModuleSig<'tcx>,

    args: &'a [ModuleGraphValue<'tcx>],

    upvars: Option<&'a [(Id, ModuleGraphValue<'tcx>)]>,

    input_interface: Interface,

    output_interface: Interface,

    thir_body: &'tcx rustc_data_structures::steal::Steal<Thir<'tcx>>,

    module_args: HashMap<ExprId, ModuleGraphValue<'tcx>>,

    submodules: Vec<ModuleGraphEdge<'tcx>>,

    meta: &'a Meta,

    prefix: &'a [String],
}

/// Function Type
///
/// In hazardflow, we consider all functions except `magic` functions as submodule
#[derive(Debug)]
#[allow(unused)]
enum FunctionTyp<'tcx> {
    /// Primitive Fsm function (`Interface::fsm`)
    InterfaceFsm(ModuleSig<'tcx>),

    /// Foregin function interface
    Ffi { sig: ModuleSig<'tcx>, module_name: String, params: Vec<(String, usize)> },

    /// Submodule
    Submodule(ModuleSig<'tcx>, Instance<'tcx>),

    /// Pure function
    Pure,

    /// Module split
    ModuleSplit(ModuleSig<'tcx>),

    /// Seq
    Seq { sig: ModuleSig<'tcx> },

    /// FromFn
    FromFn {
        i_typ: InterfaceTyp,
        o_typ: InterfaceTyp,
        j_typ: InterfaceTyp,
        t_sig: ModuleSig<'tcx>,
        t_inst: Instance<'tcx>,
        n: usize,
    },

    /// TODO
    FnPtr,
}

/// Type that represents edge(can be multiple edges from different submodules) of the submodule graph.
///
/// It is a tuple of a module and an interface, where interface indicates source nodes of the edge
/// it is from, and module indicates the target node of the edge.
pub(crate) type ModuleGraphEdge<'tcx> = (Module<'tcx>, Interface);

/// Creates new input interface from given interface type.
fn input_interface(interface_typ: &InterfaceTyp) -> Interface {
    interface_typ
        .into_primitives()
        .into_iter()
        .map(|(typ, path)| {
            (
                match typ {
                    InterfaceTyp::Unit => Interface::Unit,
                    InterfaceTyp::Channel(channel_typ) => {
                        Interface::Channel(Channel { typ: channel_typ, endpoint: Endpoint::input(path.clone()) })
                    }
                    _ => panic!("not primitive type"),
                },
                path,
            )
        })
        .collect()
}

/// Creates new output interface from given interface type and submodule index
fn submodule_output_interface(interface_typ: InterfaceTyp, submodule_index: usize) -> Interface {
    interface_typ
        .into_primitives()
        .into_iter()
        .map(|(typ, path)| {
            (
                match typ {
                    InterfaceTyp::Unit => Interface::Unit,
                    InterfaceTyp::Channel(channel_typ) => Interface::Channel(Channel {
                        typ: channel_typ,
                        endpoint: Endpoint::submodule(submodule_index, path.clone()),
                    }),
                    _ => panic!("not primitive type"),
                },
                path,
            )
        })
        .collect()
}

impl<'tcx> ModuleGraphConstructor<'tcx, '_> {
    /// Skip exprs that are not used in the module graph.
    fn skip_exprs(&self, expr_id: ExprId) -> VirgenResult<ExprId> {
        Ok(skip_exprs(&self.thir_body.borrow(), expr_id))
    }

    fn insert_module_arg(&mut self, expr_id: ExprId, module_arg: ModuleGraphValue<'tcx>) {
        let prev_module_arg = self.module_args.insert(expr_id, module_arg);
        assert!(prev_module_arg.is_none(), "Duplicated module arg for expr_id: {:#?}", expr_id);
    }

    fn monomorphise<T: TypeFoldable<TyCtxt<'tcx>>>(&self, t: T) -> T {
        let t = EarlyBinder::bind(t).instantiate(self.tcx, self.instance.args);
        normalize_alias_ty(self.tcx, t)
    }

    /// Returns the function type of the given function expression.
    fn function_typ(&self, fun: Ty<'tcx>) -> FunctionTyp<'tcx> {
        log::debug!("fun: {fun:#?}");
        let Some(instance) = self.ty_to_instance(fun) else {
            return FunctionTyp::FnPtr;
        };

        let Some(sig) = ModuleSig::from_instance(self.tcx, self.meta, instance, self.sig.generic_map.clone().into())
        else {
            return FunctionTyp::Pure;
        };

        let hazardflow_attributes =
            get_hazardflow_attribute(self.tcx, self.tcx.local_def_id_to_hir_id(instance.def_id().expect_local()));

        log::debug!("fun: {fun:#?}");
        log::debug!("hazardflow_attributes: {:#?}", hazardflow_attributes);

        if let Some(attr) = hazardflow_attributes {
            match attr {
                HazardFlowAttr::InterfaceMagic(arg) => match arg {
                    InterfaceMagic::Fsm => return FunctionTyp::InterfaceFsm(sig),
                    _ => unreachable!(),
                },
                HazardFlowAttr::FFI { module_name, params } => {
                    let generics: &Generics = self.tcx.generics_of(instance.def_id());

                    let mut instantiated_params = vec![];

                    for param_name in params {
                        if let Some(generic_def) =
                            generics.params.iter().find(|generic_def| generic_def.name.to_ident_string() == param_name)
                        {
                            let index = generic_def.index;
                            let arg = instance.args.get(index as usize).unwrap();
                            instantiated_params.push((
                                param_name,
                                evaluate_const_generic_arg(self.tcx, arg)
                                    .unwrap_or_else(|| panic!("failed to evaluate {:?} as usize", arg)),
                            ))
                        }
                    }

                    return FunctionTyp::Ffi { sig, module_name: *module_name, params: instantiated_params };
                }
                HazardFlowAttr::ModuleMagic(module_magic) => match module_magic {
                    ModuleMagic::ModuleSplit => match sig.ret_ty.as_ref() {
                        ModuleGraphType::ComposedModule(_) => return FunctionTyp::ModuleSplit(sig),
                        _ => unreachable!(),
                    },
                    ModuleMagic::FromFn => {
                        let [i, o, j, t, n] = instance.args.as_slice() else { panic!() };

                        let (def_id, args) = match self.monomorphise(t.expect_ty()).kind() {
                            rustc_type_ir::TyKind::FnDef(def_id, args)
                            | rustc_type_ir::TyKind::Closure(def_id, args) => (def_id, args),
                            _ => todo!(),
                        };

                        let t_sig = ModuleSig::from_instance(
                            self.tcx,
                            self.meta,
                            self.ty_to_instance(t.expect_ty()).expect("TODO: take care when None"),
                            Some(self.sig.generic_map.clone()),
                        )
                        .unwrap();

                        return FunctionTyp::FromFn {
                            i_typ: InterfaceTyp::from_ty(i.expect_ty(), self.meta.interface_did(), self.tcx).unwrap(),
                            o_typ: InterfaceTyp::from_ty(o.expect_ty(), self.meta.interface_did(), self.tcx).unwrap(),
                            j_typ: InterfaceTyp::from_ty(j.expect_ty(), self.meta.interface_did(), self.tcx).unwrap(),
                            t_sig,
                            t_inst: Instance::resolve(self.tcx, ParamEnv::empty(), *def_id, args).unwrap().unwrap(),
                            n: evaluate_const_generic_arg(self.tcx, n).unwrap(),
                        };
                    }
                    ModuleMagic::Seq => {
                        return FunctionTyp::Seq { sig };
                    }
                },
                HazardFlowAttr::Synthesize => {
                    panic!("Are you sure that only the top level function has `#[synthesize]` attribute?")
                }
                _ => panic!(),
            }
        }

        FunctionTyp::Submodule(sig, instance)
    }

    /// Constructs a submodule.
    fn construct_submodule(
        &mut self,
        instance: Instance<'tcx>,
        sig: ModuleSig<'tcx>,
        args: &[ExprId],
        force_construction: Option<String>,
    ) -> VirgenResult<ModuleGraphValue<'tcx>> {
        let args = args.iter().map(|arg| self.get_module_arg(*arg, force_construction.clone())).collect::<Vec<_>>();

        let (unwired_input_interface, module_arg) = self.get_wired_input_interface(&sig, &args, None);

        log::debug!("Unwired Input Interface: {:#?}", unwired_input_interface);

        let module_inst = ModuleInst {
            inst_name: join_options("_", [Some(sig.name.clone()), force_construction, Some("inst".to_string())])
                .unwrap(),
            instance,
            prefix: self.alloc_prefix(),
            args: sig
                .params
                .iter()
                .zip_eq(args)
                .enumerate()
                .map(|(i, (p, a))| match p {
                    ModuleGraphType::Interface(InterfaceTyp::Unit) => ModuleGraphValue::Unit,
                    ModuleGraphType::Interface(_) => InterfaceValue::external_interface(
                        EndpointPath::default().append_field("input").append_field(&i.to_string()),
                    )
                    .into(),
                    ModuleGraphType::Module(_) => ModuleValue::external_module(
                        EndpointPath::default().append_field("input").append_field(&i.to_string()),
                    )
                    .into(),
                    ModuleGraphType::Misc(_) => a,
                    ModuleGraphType::ComposedModule(composed) => match composed {
                        ComposedModuleTy::Tuple(_) => todo!(),
                        ComposedModuleTy::Array(_param, len) => {
                            let ModuleGraphValue::Module(ModuleValue::Composite(a_composite)) = a else { panic!() };
                            let CompositeModuleArg::Array(a_args, a_len) = a_composite else { panic!() };
                            assert_eq!(a_len, *len);
                            ModuleValue::composite_module(CompositeModuleArg::Array(a_args, *len)).into()
                        }
                    },
                })
                .collect(),
            sig,
            // TODO: calculate parameters from const generic parameters
            params: vec![],
            upvars: None,
        };

        self.submodules.push((module_inst.into(), unwired_input_interface));
        Ok(module_arg)
    }

    // TODO: Rename required
    fn get_wired_input_interface(
        &mut self,
        sig: &ModuleSig<'tcx>,
        args: &[ModuleGraphValue<'tcx>],
        upvars: Option<&[ModuleGraphValue<'tcx>]>,
    ) -> (Interface, ModuleGraphValue<'tcx>) {
        let mut unwired_input_interface = Interface::Unwired(sig.input_interface_typ());

        let submodule_index = self.submodules.len();
        let output_interface_typ = sig.output_interface_typ();
        let constructed_output_interface = submodule_output_interface(output_interface_typ, submodule_index);

        for (arg_idx, arg) in args.iter().enumerate() {
            let arg_path = EndpointPath::default().append_field("input").append_field(&arg_idx.to_string());

            if let Some(external_path) = arg.external_path() {
                let incoming = self.input_interface.get_subinterface(external_path.clone());
                unwired_input_interface.wire(arg_path.clone(), incoming);

                let outgoing = constructed_output_interface.get_subinterface(arg_path.clone());
                self.output_interface.wire(external_path.clone(), outgoing);

                continue;
            }

            if let Some(interface_arg) = arg.interface_arg() {
                match interface_arg {
                    InterfaceValue::ExternalInterface(_) => {
                        unreachable!()
                    }
                    InterfaceValue::CallResultInterface(incoming) => {
                        unwired_input_interface.wire(arg_path.clone(), incoming.clone());
                    }
                }

                continue;
            }

            if let Some(module_arg) = arg.module_arg() {
                match module_arg {
                    ModuleValue::External(_) => {
                        unreachable!()
                    }
                    ModuleValue::CallResult { submodule_index, output_interface, path } => {
                        unwired_input_interface.wire(arg_path.clone(), output_interface.clone());

                        let wiring_path = path.clone();
                        self.submodules[*submodule_index]
                            .1
                            .wire(wiring_path, constructed_output_interface.get_subinterface(arg_path.clone()));
                    }
                    ModuleValue::Closure { submodule_index, output_interface } => {
                        unwired_input_interface.wire(arg_path.clone(), output_interface.clone());

                        let in_wiring_path =
                            arg_path.clone().append_node(EndpointNode::Field("input".to_string(), None));
                        self.submodules[*submodule_index].1.wire(
                            [EndpointNode::Field("input".to_string(), None)].into_iter().collect(),
                            constructed_output_interface.get_subinterface(in_wiring_path),
                        );

                        // assert_eq!(output_interface, &Interface::Unit);

                        let out_wiring_path =
                            arg_path.clone().append_node(EndpointNode::Field("output".to_string(), None));
                        self.submodules[*submodule_index].1.wire(
                            [EndpointNode::Field("output".to_string(), None)].into_iter().collect(),
                            constructed_output_interface.get_subinterface(out_wiring_path),
                        );
                    }
                    ModuleValue::Function { submodule_index, output_interface } => {
                        unwired_input_interface.wire(arg_path.clone(), output_interface.clone());

                        self.submodules[*submodule_index].1.wire(
                            EndpointPath::default(),
                            constructed_output_interface.get_subinterface(arg_path.clone()),
                        );
                    }
                    ModuleValue::Composite(module_args) => {
                        match module_args {
                            CompositeModuleArg::Tuple(_) => todo!(),
                            CompositeModuleArg::Array(args, ..) => {
                                for (idx, arg) in args.iter().enumerate() {
                                    match arg {
                                        ModuleValue::Function { submodule_index, output_interface }
                                        | ModuleValue::Closure { submodule_index, output_interface } => {
                                            // wire output of submodule
                                            unwired_input_interface
                                                .wire(arg_path.clone().append_index(idx), output_interface.clone());

                                            // wire input of submodule
                                            self.submodules[*submodule_index].1.wire(
                                                EndpointPath::default().append_field("input"),
                                                constructed_output_interface.get_subinterface(
                                                    arg_path.clone().append_index(idx).append_field("input"),
                                                ),
                                            );
                                        }
                                        ModuleValue::CallResult { submodule_index, output_interface, path } => {
                                            // wire output of submodule
                                            unwired_input_interface
                                                .wire(arg_path.clone().append_index(idx), output_interface.clone());

                                            // wire input of submodule
                                            if self.submodules[*submodule_index].1.contains_unwired() {
                                                self.submodules[*submodule_index].1.wire(
                                                    path.clone(),
                                                    constructed_output_interface
                                                        .clone()
                                                        .get_subinterface(arg_path.clone().append_index(idx)),
                                                );
                                            }
                                        }
                                        ModuleValue::Composite(_) => todo!(),
                                        ModuleValue::External(_) => todo!(),
                                    }
                                }
                            }
                        }
                    }
                }

                continue;
            }

            match arg {
                ModuleGraphValue::ConstantFunctionArgs(_) | ModuleGraphValue::Unit => {
                    unwired_input_interface.wire(arg_path, Interface::Unit);
                }
                _ => {
                    unreachable!()
                }
            }
        }

        let captured_path = EndpointPath::default().append_field("captured");
        if let Some(_upvars) = upvars {
            todo!()
        } else {
            unwired_input_interface.wire(captured_path, Interface::Unit)
        }

        let out_path = EndpointPath::default().append_field("output");
        let module_arg = match sig.ret_ty.as_ref() {
            ModuleGraphType::Interface(_) => {
                unwired_input_interface.wire(out_path, Interface::Unit);
                InterfaceValue::call_result_interface(
                    constructed_output_interface
                        .get_subinterface([EndpointNode::Field("output".to_string(), None)].into_iter().collect()),
                )
                .into()
            }
            ModuleGraphType::Module(sig) => {
                unwired_input_interface.wire(out_path.clone(), Interface::Unwired(sig.input_interface_typ()));

                ModuleValue::call_result_module(
                    submodule_index,
                    constructed_output_interface.get_subinterface(out_path.clone()),
                    out_path,
                )
                .into()
            }
            ModuleGraphType::Misc(_) => panic!(),
            ModuleGraphType::ComposedModule(composed_module_param) => match composed_module_param {
                ComposedModuleTy::Tuple(params) => {
                    let mut args = vec![];

                    // WARN: Is this correct? Look again
                    for (i, param) in params.iter().enumerate() {
                        let out_path = out_path.clone().append_field(&i.to_string());
                        unwired_input_interface
                            .wire(out_path.clone(), Interface::Unwired(param.output_interface_typ()));

                        args.push(ModuleValue::call_result_module(
                            submodule_index,
                            constructed_output_interface.get_subinterface(out_path.clone()),
                            out_path,
                        ));
                    }

                    ModuleValue::composite_module(CompositeModuleArg::Tuple(args)).into()
                }
                ComposedModuleTy::Array(param, len) => {
                    let mut args = vec![];

                    // WARN: Is this correct? Look again
                    for i in 0..*len {
                        let out_path = out_path.clone().append_index(i);
                        unwired_input_interface
                            .wire(out_path.clone(), Interface::Unwired(param.output_interface_typ()));

                        args.push(ModuleValue::call_result_module(
                            submodule_index,
                            constructed_output_interface.get_subinterface(out_path.clone()),
                            out_path,
                        ));
                    }

                    ModuleValue::composite_module(CompositeModuleArg::Array(args, *len)).into()
                }
            },
        };
        (unwired_input_interface, module_arg)
    }

    fn collect_interface(&self, input: Interface, captured: Vec<Interface>, output: Interface) -> Interface {
        Interface::Struct(
            [
                ("input".to_string(), input),
                ("captured".to_string(), captured.into_iter().collect()),
                ("output".to_string(), output),
            ]
            .into_iter()
            .map(|(k, v)| (k, (None, v)))
            .collect(),
        )
    }

    fn construct_module_arg(&mut self, id: ExprId, force_construction: Option<String>) -> ModuleGraphValue<'tcx> {
        let expr = &self.thir_body.borrow().exprs[id];
        match &expr.kind {
            ExprKind::Scope { value, .. } => self.get_module_arg(*value, force_construction),
            ExprKind::Call { fun, args, .. } => {
                let function_id = self.skip_exprs(*fun).unwrap();

                let instance =
                    self.ty_to_instance(self.thir_body.borrow()[function_id].ty).expect("TODO: take care when None");

                if let Some(_sig) =
                    ModuleSig::from_instance(self.tcx, self.meta, instance, self.sig.generic_map.clone().into())
                {
                    self.construct_function_call(expr, fun, args, force_construction).unwrap().unwrap()
                } else {
                    let args = args
                        .iter()
                        .map(|arg| self.get_module_arg(*arg, force_construction.clone()))
                        .collect::<Vec<_>>();

                    let ty = PortDecls::from_ty(self.monomorphise(expr.ty), self.tcx);
                    assert!(ty.is_some());
                    match &self.thir_body.borrow().exprs[function_id].ty.kind() {
                        rustc_type_ir::TyKind::FnDef(id, substs) => {
                            let instance =
                                Instance::resolve(self.tcx, ParamEnv::empty(), *id, self.monomorphise(substs))
                                    .unwrap()
                                    .unwrap();

                            let f = if instance.def_id().is_local() {
                                FunctionBuilder::new_local(instance, self.tcx)
                            } else {
                                panic!()
                            };

                            let (expr, displays) = f.build(
                                self.tcx,
                                args.into_iter().map(|arg| arg.function_arg().unwrap()).collect(),
                                &mut FsmCache::default(),
                            );

                            assert!(displays.is_empty(), "trying to display outside of fsm");

                            ModuleGraphValue::ConstantFunctionArgs(PureValue::Expr(expr))
                        }
                        _ => panic!(),
                    }
                }
            }
            ExprKind::Block { block } => {
                let block = &self.thir_body.borrow()[*block];
                block.expr.map(|expr| self.get_module_arg(expr, force_construction)).unwrap()
            }
            ExprKind::Field { lhs, variant_index, name } => {
                assert!(variant_index.index() == 0, "relax when needed");
                match self.get_module_arg(*lhs, force_construction) {
                    ModuleGraphValue::Interface(interface_arg) => match interface_arg {
                        InterfaceValue::ExternalInterface(path) => {
                            self.output_interface.wire(path.clone(), Interface::Unit);
                            assert!(
                                matches!(self.input_interface.get_subinterface(path.clone()), Interface::Struct(_)),
                                "relax when needed"
                            );
                            ModuleGraphValue::Interface(InterfaceValue::CallResultInterface(
                                self.input_interface
                                    .get_subinterface(path.clone().append_field(&name.index().to_string())),
                            ))
                        }
                        InterfaceValue::CallResultInterface(i) => match self.thir_body.borrow()[*lhs].ty.kind() {
                            rustc_type_ir::TyKind::Adt(adt_def, _) => {
                                assert!(adt_def.is_struct());
                                let field_name = &adt_def.variant(*variant_index).fields[*name].name.to_string();
                                let subinterface = i.get_subinterface(EndpointPath::default().append_field(field_name));
                                InterfaceValue::call_result_interface(subinterface).into()
                            }
                            rustc_type_ir::TyKind::Tuple(_) => {
                                let subinterface =
                                    i.get_subinterface(EndpointPath::default().append_field(&name.index().to_string()));
                                InterfaceValue::call_result_interface(subinterface).into()
                            }
                            _ => panic!(),
                        },
                    },
                    ModuleGraphValue::Module(_) => todo!(),
                    ModuleGraphValue::ConstantFunctionArgs(_) => todo!(),
                    ModuleGraphValue::Unit => todo!(),
                }
            }
            ExprKind::VarRef { id } => {
                let mut local_var_resolved = resolve_var_ref(self.tcx, self.thir_body, *id, None);
                assert_eq!(local_var_resolved.len(), 1);

                let local_var_resolved = local_var_resolved.pop().unwrap();
                match local_var_resolved {
                    LocalVar::Param { arg_idx, accessor, .. } => {
                        let bounded_arg = if self.is_closure() {
                            // NOTE: We use `arg_idx-1` because closure silently adds itself as the first argument
                            self.args[arg_idx - 1].clone()
                        } else {
                            self.args[arg_idx].clone()
                        };
                        accessor.iter().fold(bounded_arg, |module_arg, accessor| match accessor {
                            PatAccessNode::Field { idx: _, name } => match module_arg {
                                ModuleGraphValue::Interface(interface_arg) => match &interface_arg {
                                    InterfaceValue::ExternalInterface(path) => {
                                        self.output_interface.wire(path.clone(), Interface::Unit);
                                        let interface = self.input_interface.get_subinterface(path.clone());
                                        match interface {
                                            Interface::Struct(inner) => {
                                                if let Some((_, interface)) = inner.get(name) {
                                                    InterfaceValue::call_result_interface(interface.clone()).into()
                                                } else {
                                                    panic!()
                                                }
                                            }
                                            Interface::Unwired(_) => todo!(),
                                            _ => panic!(),
                                        }
                                    }
                                    InterfaceValue::CallResultInterface(_) => todo!(),
                                },
                                ModuleGraphValue::Module(_) => todo!(),
                                ModuleGraphValue::ConstantFunctionArgs(_) => todo!(),
                                ModuleGraphValue::Unit => todo!(),
                            },
                            PatAccessNode::Variant { .. } => todo!(),
                            PatAccessNode::Index(_) => todo!(),
                        })
                    }
                    LocalVar::Stmt { expr_id, accessor, .. } => {
                        let bounded_arg = self.get_module_arg(expr_id, force_construction);
                        accessor.iter().fold(bounded_arg, |acc, elt| match elt {
                            PatAccessNode::Field { name, .. } => match acc {
                                ModuleGraphValue::Interface(interface_arg) => match interface_arg {
                                    InterfaceValue::ExternalInterface(path) => {
                                        self.output_interface.wire(path.clone(), Interface::Unit);
                                        let interface = self.input_interface.get_subinterface(path);
                                        match interface {
                                            Interface::Struct(inner) => {
                                                if let Some((_, interface)) = inner.get(name) {
                                                    InterfaceValue::call_result_interface(interface.clone()).into()
                                                } else {
                                                    panic!()
                                                }
                                            }
                                            Interface::Unwired(_) => todo!(),
                                            _ => panic!(),
                                        }
                                    }
                                    InterfaceValue::CallResultInterface(interface) => match interface {
                                        Interface::Struct(inner) => {
                                            if let Some((_, interface)) = inner.get(name) {
                                                InterfaceValue::call_result_interface(interface.clone()).into()
                                            } else {
                                                panic!()
                                            }
                                        }
                                        Interface::Unwired(_) => todo!(),
                                        _ => todo!(),
                                    },
                                },
                                ModuleGraphValue::Module(module_arg) => match module_arg {
                                    ModuleValue::Composite(composite_module) => match composite_module {
                                        CompositeModuleArg::Tuple(inner) => {
                                            let index = name
                                                .parse::<usize>()
                                                .expect("Tuple access should be done by unsigned integer");
                                            inner[index].clone().into()
                                        }
                                        CompositeModuleArg::Array(..) => panic!("Array should be accessed by index"),
                                    },
                                    _ => panic!(),
                                },
                                ModuleGraphValue::ConstantFunctionArgs(_) => todo!(),
                                ModuleGraphValue::Unit => todo!(),
                            },
                            PatAccessNode::Index(index) => match acc {
                                ModuleGraphValue::Interface(interface_arg) => match interface_arg {
                                    InterfaceValue::CallResultInterface(interface) => match interface {
                                        Interface::Array(inner) => {
                                            InterfaceValue::call_result_interface(inner[*index].clone()).into()
                                        }
                                        Interface::Unwired(_) => todo!(),
                                        _ => todo!(),
                                    },
                                    InterfaceValue::ExternalInterface(_) => todo!(),
                                },
                                ModuleGraphValue::Module(module_arg) => match module_arg {
                                    ModuleValue::Composite(composite_module) => match composite_module {
                                        CompositeModuleArg::Array(args, len) => {
                                            let index = *index;
                                            assert!(index < len, "index out of bound");
                                            args[index].clone().into()
                                        }
                                        CompositeModuleArg::Tuple(_) => panic!("Tuple should be accessed by field"),
                                    },
                                    _ => panic!(),
                                },
                                ModuleGraphValue::ConstantFunctionArgs(_) => todo!(),
                                ModuleGraphValue::Unit => todo!(),
                            },
                            PatAccessNode::Variant { .. } => panic!(),
                        })
                    }
                    LocalVar::PatBinding { .. } => panic!(),
                }
            }
            ExprKind::UpvarRef { var_hir_id, .. } => {
                for (id, upvar) in self.upvars.unwrap().iter() {
                    match id {
                        Id::Local(id) => {
                            if id == var_hir_id {
                                return upvar.clone();
                            }
                        }
                        Id::Upvar(_) => todo!(),
                    }
                }
                unreachable!()
            }
            ExprKind::Array { fields } => {
                if let Ok(InterfaceTyp::Array(..)) =
                    InterfaceTyp::from_ty(self.monomorphise(expr.ty), self.meta.interface_did(), self.tcx)
                {
                    let args = fields
                        .iter()
                        .map(|arg| match self.get_module_arg(*arg, force_construction.clone()) {
                            ModuleGraphValue::Interface(interface_arg) => match interface_arg {
                                InterfaceValue::ExternalInterface(path) => {
                                    self.output_interface.wire(path.clone(), Interface::Unit);
                                    self.input_interface.get_subinterface(path)
                                }
                                InterfaceValue::CallResultInterface(i) => i,
                            },
                            _ => todo!(),
                        })
                        .collect::<Vec<_>>();
                    InterfaceValue::call_result_interface(Interface::Array(args)).into()
                } else {
                    let field_args = fields
                        .iter()
                        .map(|id| self.get_module_arg(*id, force_construction.clone()))
                        .collect::<Vec<_>>();
                    let arr_len = field_args.len();
                    if field_args.iter().all(|arg| matches!(arg, ModuleGraphValue::Module(_))) {
                        ModuleGraphValue::Module(ModuleValue::Composite(CompositeModuleArg::Array(
                            {
                                let mut module_arg_modules = vec![];
                                for arg in field_args {
                                    if let ModuleGraphValue::Module(module_arg) = arg {
                                        module_arg_modules.push(module_arg);
                                    } else {
                                        unreachable!()
                                    }
                                }
                                module_arg_modules
                            },
                            arr_len,
                        )))
                    } else {
                        todo!("{field_args:#?}")
                    }
                }
            }
            ExprKind::Tuple { fields } => {
                let field_args =
                    fields.iter().map(|id| self.get_module_arg(*id, force_construction.clone())).collect::<Vec<_>>();
                if field_args.is_empty() {
                    ModuleGraphValue::Unit
                } else if field_args
                    .iter()
                    .all(|arg| matches!(arg, ModuleGraphValue::Interface(_) | ModuleGraphValue::Unit))
                {
                    InterfaceValue::call_result_interface(
                        field_args
                            .into_iter()
                            .map(|arg| {
                                if let Some(interface) = arg.interface_arg() {
                                    match interface {
                                        InterfaceValue::ExternalInterface(path) => {
                                            self.output_interface.wire(path.clone(), Interface::Unit);
                                            self.input_interface.get_subinterface(path.clone())
                                        }
                                        InterfaceValue::CallResultInterface(interface) => interface.clone(),
                                    }
                                } else if let ModuleGraphValue::Unit = arg {
                                    Interface::Unit
                                } else {
                                    unreachable!()
                                }
                            })
                            .collect(),
                    )
                    .into()
                } else if field_args.iter().all(|arg| matches!(arg, ModuleGraphValue::Module(_))) {
                    ModuleValue::composite_module(CompositeModuleArg::Tuple(
                        field_args.into_iter().map(|arg| arg.module_arg().unwrap().clone()).collect(),
                    ))
                    .into()
                } else {
                    todo!()
                }
            }
            ExprKind::Adt(e) => {
                let ty = self.monomorphise(expr.ty);
                match ty.kind() {
                    rustc_type_ir::TyKind::Adt(adt_def, _) => match adt_def.adt_kind() {
                        rustc_middle::ty::AdtKind::Enum => {
                            unreachable!()
                        }
                        rustc_middle::ty::AdtKind::Struct => {
                            let fields = e
                                .fields
                                .iter()
                                .map(|field_expr| {
                                    let field_name =
                                        adt_def.variant(e.variant_index).fields[field_expr.name].name.to_ident_string();
                                    let x = match self
                                        .get_module_arg(field_expr.expr, force_construction.clone())
                                        .interface_arg()
                                        .expect("we currenty expect composition of interfaces")
                                    {
                                        InterfaceValue::CallResultInterface(interface) => interface.clone(),
                                        InterfaceValue::ExternalInterface(path) => {
                                            self.output_interface.wire(path.clone(), Interface::Unit);
                                            self.input_interface.get_subinterface(path.clone())
                                        }
                                    };
                                    (field_name, (None, x))
                                })
                                .collect();

                            let struct_interface = Interface::Struct(fields);
                            InterfaceValue::call_result_interface(struct_interface).into()
                        }
                        rustc_middle::ty::AdtKind::Union => todo!(),
                    },
                    _ => panic!(),
                }
            }
            ExprKind::Closure(closure_expr) => self.closure_to_module_arg(closure_expr, expr.ty, force_construction),
            ExprKind::Literal { lit, neg } => {
                ModuleGraphValue::ConstantFunctionArgs(PureValue::Expr(build_literal(neg, lit, expr.ty, self.tcx)))
            }
            ExprKind::ZstLiteral { .. } => self.zst_lit_to_module_arg(expr.ty),
            ExprKind::PointerCoercion { cast, source } => match cast {
                rustc_middle::ty::adjustment::PointerCoercion::ClosureFnPointer(unsafety) => match unsafety {
                    rustc_hir::Unsafety::Normal => self.get_module_arg(*source, force_construction),
                    rustc_hir::Unsafety::Unsafe => panic!(),
                },
                // Go from a fn-item type to a fn-pointer type.
                rustc_middle::ty::adjustment::PointerCoercion::ReifyFnPointer => {
                    self.get_module_arg(*source, force_construction)
                }
                _ => panic!("{cast:#?}"),
            },
            _ => todo!("{expr:?}"),
        }
    }

    // Returns the ModuleArg that corresponds to the given `ExprId`
    //
    // If the `force_construction` is None,
    //   - This function will search the cache(`self.module_args`) and return the module_arg if it exists.
    //   - This function will construct(instantiate) a new module and store to the cache if the module_arg doesn't exist.
    // Othewise, if the `force_construction` is Some,
    //    - This function will not search and store to the cache(`self.module_args`)
    //    - This function will construct(instantiate) a new module.
    // Currently, the `force_construction` is used when the module is constructed from the `from_fn` combinator.
    fn get_module_arg(&mut self, id: ExprId, force_construction: Option<String>) -> ModuleGraphValue<'tcx> {
        let expr = &self.thir_body.borrow().exprs[id];
        log::debug!("Get Arg: {expr:#?}\nspan: {:#?}", expr.span);

        // 1. If the module arg is already calculated, return it
        if force_construction.is_none() {
            if let Some(arg) = self.module_args.get(&id) {
                return arg.clone();
            }
        }

        if let ExprKind::Tuple { fields } = &expr.kind {
            if fields.is_empty() {
                return ModuleGraphValue::Unit;
            }
        }

        if InterfaceTyp::from_ty(self.monomorphise(expr.ty), self.meta.interface_did(), self.tcx).is_err() {
            if let Some(ty) = PortDecls::from_ty(self.monomorphise(expr.ty), self.tcx) {
                log::debug!("Const expr: {ty:#?}");

                let upvars = self.upvars.map(|upvars| {
                    upvars.iter().map(|(id, arg)| (*id, arg.function_arg().unwrap_or(PureValue::Misc))).collect_vec()
                });

                return ModuleGraphValue::ConstantFunctionArgs(PureValue::Expr(build_const_expr(
                    self.tcx,
                    id,
                    self.thir_body,
                    self.instance.args,
                    &self.args.iter().map(|arg| arg.function_arg().unwrap_or(PureValue::Misc)).collect::<Vec<_>>(),
                    upvars.as_deref(),
                )));
            }
        }

        if force_construction.is_some() {
            assert!(self.module_args.get(&id).is_some());
        }

        // 2. If the expression id doesn't exist, then construct module_arg
        let module_arg = self.construct_module_arg(id, force_construction.clone());

        // 3. Store the calculated module_arg to the expression id
        if force_construction.is_none() {
            self.insert_module_arg(id, module_arg.clone());
        }

        module_arg
    }

    fn ty_to_instance(&self, ty: Ty<'tcx>) -> Option<Instance<'tcx>> {
        match self.monomorphise(ty).kind() {
            rustc_type_ir::TyKind::FnDef(id, substs) | rustc_type_ir::TyKind::Closure(id, substs) => {
                Instance::resolve(self.tcx, ParamEnv::empty(), *id, substs).unwrap().unwrap().into()
            }
            rustc_type_ir::TyKind::Alias(kind, alias) => match kind {
                rustc_type_ir::AliasKind::Opaque => {
                    match self.tcx.try_expand_impl_trait_type(alias.def_id, alias.args) {
                        Ok(expanded_ty) => self.ty_to_instance(expanded_ty),
                        Err(_) => todo!(),
                    }
                }
                _ => todo!(),
            },
            rustc_type_ir::TyKind::FnPtr(_bind) => None,
            tykind => panic!("{:?}", tykind),
        }
    }

    fn zst_lit_to_module_arg(&mut self, ty: Ty<'tcx>) -> ModuleGraphValue<'tcx> {
        match self.function_typ(ty) {
            FunctionTyp::Ffi { sig, module_name, params } => self.construct_ffi(sig, module_name, params),
            FunctionTyp::Submodule(sig, instance) => {
                let mut input_interface = Interface::Unwired(sig.input_interface_typ());

                input_interface
                    .wire([EndpointNode::Field("captured".to_string(), None)].into_iter().collect(), Interface::Unit);

                match sig.ret_ty.as_ref() {
                    ModuleGraphType::Interface(_) => input_interface
                        .wire([EndpointNode::Field("output".to_string(), None)].into_iter().collect(), Interface::Unit),
                    ModuleGraphType::Module(_sig) => {
                        todo!()
                    }
                    ModuleGraphType::Misc(_) => todo!(),
                    ModuleGraphType::ComposedModule(_) => todo!(),
                };

                let submodule_index = self.submodules.len();
                let output_interface = submodule_output_interface(sig.output_interface_typ(), submodule_index);

                let module = ModuleInst {
                    inst_name: join_options("_", [Some(sig.name.clone()), Some("inst".to_string())]).unwrap(),
                    instance,
                    args: sig
                        .params
                        .iter()
                        .enumerate()
                        .map(|(i, p)| match p {
                            ModuleGraphType::Interface(InterfaceTyp::Unit) => ModuleGraphValue::Unit,
                            ModuleGraphType::Interface(_) => InterfaceValue::external_interface(
                                [
                                    EndpointNode::Field("input".to_string(), None),
                                    EndpointNode::Field(i.to_string(), None),
                                ]
                                .into_iter()
                                .collect(),
                            )
                            .into(),
                            ModuleGraphType::Module(_) => ModuleValue::external_module(
                                [
                                    EndpointNode::Field("input".to_string(), None),
                                    EndpointNode::Field(i.to_string(), None),
                                ]
                                .into_iter()
                                .collect(),
                            )
                            .into(),
                            ModuleGraphType::Misc(_) => panic!(),
                            ModuleGraphType::ComposedModule(_) => todo!(),
                        })
                        .collect(),
                    prefix: self.alloc_prefix(),
                    sig,
                    // TODO: calculate parameters from const generic parameters
                    params: vec![],
                    upvars: None,
                };

                // XXX: insert to interfaces..?
                // Maybe not since this path can only be reached when a function is being passed to
                // another function
                self.submodules.push((module.into(), input_interface));
                ModuleValue::function_module(submodule_index, output_interface).into()
            }
            FunctionTyp::Pure => {
                if let Some(pure) = self.ty_to_function_builder(ty) {
                    ModuleGraphValue::ConstantFunctionArgs(PureValue::Function(pure))
                } else {
                    panic!()
                }
            }
            _ => unreachable!(),
        }
    }

    fn construct_ffi(
        &mut self,
        sig: ModuleSig<'tcx>,
        module_name: String,
        params: Vec<(String, usize)>,
    ) -> ModuleGraphValue<'tcx> {
        let mut input_interface = Interface::Unwired(sig.input_interface_typ());

        input_interface
            .wire([EndpointNode::Field("captured".to_string(), None)].into_iter().collect(), Interface::Unit);

        match sig.ret_ty.as_ref() {
            ModuleGraphType::Interface(_) => input_interface
                .wire([EndpointNode::Field("output".to_string(), None)].into_iter().collect(), Interface::Unit),
            ModuleGraphType::Module(_sig) => {
                todo!()
            }
            ModuleGraphType::Misc(_) => todo!(),
            ModuleGraphType::ComposedModule(_) => todo!(),
        };

        let submodule_index = self.submodules.len();
        let output_interface = submodule_output_interface(sig.output_interface_typ(), submodule_index);
        let module = Ffi { sig, inst_name: format!("ffi_{module_name}_{}", submodule_index), module_name, params };
        // XXX: insert to interfaces..?
        // Maybe not since this path can only be reached when a function is being passed to
        // another function
        self.submodules.push((module.into(), input_interface));
        ModuleValue::function_module(submodule_index, output_interface).into()
    }

    fn ty_to_function_builder(&self, ty: Ty<'tcx>) -> Option<FunctionBuilder<'tcx>> {
        match ty.kind() {
            rustc_type_ir::TyKind::FnDef(id, substs) => {
                let instance =
                    Instance::resolve(self.tcx, ParamEnv::empty(), *id, self.monomorphise(substs)).unwrap().unwrap();

                log::debug!("instance: {:#?}", instance.def);

                if instance.def_id().is_local() {
                    Some(FunctionBuilder::new_local(instance, self.tcx))
                } else {
                    // TODO: merge with `build_call`
                    panic!()
                }
            }
            rustc_type_ir::TyKind::Closure(..) => todo!(),
            rustc_type_ir::TyKind::FnPtr(_) => todo!(),
            _ => None,
        }
    }

    fn alloc_prefix(&self) -> Vec<String> {
        let mut prefix = self.prefix.to_vec();

        if let Some(name) = self.tcx.opt_item_name(self.instance.def_id()) {
            prefix.push(format!("{}_{:02}", name.to_ident_string(), self.submodules.len()));
            prefix
        } else if self.is_closure() {
            prefix.push(format!("closure_{}", self.submodules.len()));
            prefix
        } else {
            todo!()
        }
    }

    /// Construct module topology graph.
    fn construct_graph(mut self) -> VirgenResult<(Vec<ModuleGraphEdge<'tcx>>, Interface)> {
        // 1. Traverse module function calls, and construct node/egde for each submodule
        self.traverse_function_calls()?;

        // 2. Resolve output interface, by looking at the last expression (i.e., the return value of the module)
        self.wire_output_interface()?;

        // 3. Resolve unit interfaces that is not wired
        self.resolve_pure_interfaces();

        Ok((self.submodules, self.output_interface))
    }

    fn resolve_pure_interfaces_inner(&mut self, param: &ModuleGraphType<'tcx>, param_path: EndpointPath) {
        match param {
            ModuleGraphType::Interface(ty) => {
                if let InterfaceTyp::Unit = ty {
                    self.output_interface.wire(param_path, Interface::Unit)
                }
            }
            ModuleGraphType::Module(_) => {}
            ModuleGraphType::Misc(_) => self.output_interface.wire(param_path, Interface::Unit),
            ModuleGraphType::ComposedModule(composed) => match composed {
                ComposedModuleTy::Tuple(_) => todo!(),
                ComposedModuleTy::Array(param_inner, len) => {
                    for i in 0..*len {
                        let path = param_path.clone().append_index(i);
                        self.resolve_pure_interfaces_inner(param_inner, path);
                    }
                }
            },
        }
    }

    fn resolve_pure_interfaces(&mut self) {
        for (param_idx, param) in self.sig.params.iter().enumerate() {
            let param_path =
                [EndpointNode::Field("input".to_string(), None), EndpointNode::Field(param_idx.to_string(), None)]
                    .into_iter()
                    .collect::<EndpointPath>();

            self.resolve_pure_interfaces_inner(param, param_path);
        }
    }

    fn construct_function_call(
        &mut self,
        expr: &thir::Expr<'tcx>,
        fun: &ExprId,
        args: &[ExprId],
        force_construction: Option<String>,
    ) -> VirgenResult<Option<ModuleGraphValue<'tcx>>> {
        if is_closure_call_with_id(self.tcx, self.thir_body, *fun, args) {
            let module_arg = self.handle_closure_call(args, force_construction)?;
            return Ok(Some(module_arg));
        }

        let function_expr = &self.thir_body.borrow().exprs[*fun];

        log::debug!("expr span: {:#?}", expr.span);
        let module_arg = match self.function_typ(function_expr.ty) {
            FunctionTyp::Submodule(sig, instance) => {
                assert!(!matches!(function_expr.kind, ExprKind::Closure(_)), "TODO");
                self.construct_submodule(instance, sig, args.as_ref(), force_construction)?
            }
            FunctionTyp::InterfaceFsm(sig) => self.construct_fsm(sig, args.as_ref(), force_construction)?,
            FunctionTyp::ModuleSplit(sig) => self.construct_module_split(sig, args.as_ref(), force_construction)?,
            FunctionTyp::Seq { sig } => self.construct_module_seq(sig, args.as_ref(), force_construction)?,
            FunctionTyp::FromFn { n, .. } => self.construct_from_fn(n, args.as_ref(), force_construction)?,
            FunctionTyp::FnPtr => self.construct_fn_ptr(*fun, args, force_construction)?,
            FunctionTyp::Ffi { sig, module_name, params } => {
                let ffi = self.construct_ffi(sig, module_name, params);

                let ModuleGraphValue::Module(ModuleValue::Function { submodule_index, output_interface }) = ffi else {
                    panic!()
                };

                let args =
                    args.iter().map(|arg| self.get_module_arg(*arg, force_construction.clone())).collect::<Vec<_>>();

                for (idx, arg) in args.into_iter().enumerate() {
                    match arg {
                        ModuleGraphValue::Interface(interface) => match interface {
                            InterfaceValue::ExternalInterface(path) => {
                                self.submodules[submodule_index].1.wire(
                                    EndpointPath::default().append_field("input").append_field(&idx.to_string()),
                                    self.input_interface.get_subinterface(path.clone()),
                                );
                                self.output_interface.wire(path, Interface::Unit);
                            }
                            InterfaceValue::CallResultInterface(interface) => {
                                self.submodules[submodule_index].1.wire(
                                    EndpointPath::default().append_field("input").append_field(&idx.to_string()),
                                    interface,
                                );
                            }
                        },
                        _ => todo!(),
                    }
                }

                InterfaceValue::call_result_interface(
                    output_interface.get_subinterface(EndpointPath::default().append_field("output")),
                )
                .into()
            }
            // NOTE: It is not a *submodule instantiation*, so do nothing
            FunctionTyp::Pure => return Ok(None),
        };

        Ok(Some(module_arg))
    }

    /// Traverse function calls and construct submodules
    ///
    /// This function constructs submodules by traversing function calls in the module.
    /// - If the function is a submodule, construct a submodule node and edge.
    /// - If the function is a foreign function interface, construct a submodule node and edge.
    /// - If the function is a pure function, skip.
    fn traverse_function_calls(&mut self) -> Result<(), VirgenError> {
        for (expr_id, expr) in self.thir_body.borrow().exprs.iter().enumerate() {
            if matches!(expr.kind, ExprKind::If { .. } | ExprKind::Loop { .. } | ExprKind::Return { .. }) {
                return Err(VirgenError::collect_fsm_error(
                    "control flow is not allowed in interface level, since interface topology is fixed in a circuit"
                        .to_string(),
                ));
            }

            if let ExprKind::Call {
                fun,
                args,
                // from_hir_call,
                // fn_span,
                ..
            } = &expr.kind
            {
                if let Some(module_arg) = self.construct_function_call(expr, fun, args, None)? {
                    self.insert_module_arg(expr_id.into(), module_arg);
                }
            }
        }

        Ok(())
    }

    fn wire_output_interface(&mut self) -> Result<(), VirgenError> {
        let final_expr_id = self.skip_exprs(self.thir_body.borrow().exprs.last_index().unwrap())?;

        let output_base_path = EndpointPath::default().append_field("output");

        if let Some(arg) = self.module_args.get(&final_expr_id) {
            if let Some(interface_arg) = arg.interface_arg() {
                match interface_arg {
                    InterfaceValue::ExternalInterface(_) => todo!(),
                    InterfaceValue::CallResultInterface(interface) => {
                        self.output_interface.wire(output_base_path, interface.clone())
                    }
                }
                return Ok(());
            }

            if let Some(module_arg) = arg.module_arg() {
                log::debug!("module_arg: {:#?}", module_arg);
                match module_arg {
                    ModuleValue::Composite(composite_module_arg) => match composite_module_arg {
                        CompositeModuleArg::Tuple(inner) => {
                            for (i, module) in inner.iter().enumerate() {
                                match module {
                                    ModuleValue::External(_) => todo!(),
                                    ModuleValue::CallResult { submodule_index, output_interface, path } => {
                                        self.submodules[*submodule_index].1.wire(
                                            path.clone(),
                                            self.input_interface
                                                .get_subinterface(output_base_path.append_field(&i.to_string())),
                                        );

                                        self.output_interface.wire(
                                            output_base_path.append_field(&i.to_string()),
                                            output_interface.clone(),
                                        );
                                    }
                                    ModuleValue::Closure { .. } => todo!(),
                                    ModuleValue::Function { .. } => todo!(),
                                    ModuleValue::Composite(_) => todo!(),
                                }
                            }
                        }
                        CompositeModuleArg::Array(inner, ..) => {
                            for (i, module) in inner.iter().enumerate() {
                                match module {
                                    ModuleValue::External(_) => todo!(),
                                    ModuleValue::CallResult { submodule_index, output_interface, path } => {
                                        self.submodules[*submodule_index].1.wire(
                                            path.clone(),
                                            self.input_interface.get_subinterface(output_base_path.append_index(i)),
                                        );

                                        self.output_interface
                                            .wire(output_base_path.append_index(i), output_interface.clone());
                                    }
                                    ModuleValue::Closure { .. } => todo!(),
                                    ModuleValue::Function { .. } => todo!(),
                                    ModuleValue::Composite(_) => todo!(),
                                }
                            }
                        }
                    },
                    x => todo!("{x:?}"),
                }

                return Ok(());
            } else {
                todo!("arg: {:#?}", arg);
            }
        }

        let arg = self.get_module_arg(final_expr_id, None);

        if let Some(interface_arg) = arg.interface_arg() {
            match interface_arg {
                InterfaceValue::ExternalInterface(path) => {
                    self.output_interface.wire(path.clone(), Interface::Unit);
                    let interface = self.input_interface.get_subinterface(path.clone());
                    self.output_interface.wire(EndpointPath::default().append_field("output"), interface)
                }
                InterfaceValue::CallResultInterface(interface) => {
                    self.output_interface.wire(EndpointPath::default().append_field("output"), interface.clone())
                }
            }

            return Ok(());
        }

        if let Some(m) = arg.module_arg() {
            match m {
                ModuleValue::External(_) => todo!(),
                ModuleValue::CallResult { .. } => todo!(),
                ModuleValue::Closure { submodule_index, output_interface } => {
                    self.submodules[*submodule_index].1.wire(
                        EndpointPath::default().append_field("input"),
                        self.input_interface
                            .get_subinterface(EndpointPath::default().append_field("output").append_field("input")),
                    );
                    assert_eq!(
                        output_interface.get_subinterface(EndpointPath::default().append_field("captured")),
                        Interface::Unit
                    );
                    self.submodules[*submodule_index].1.wire(
                        EndpointPath::default().append_field("output"),
                        self.input_interface
                            .get_subinterface(EndpointPath::default().append_field("output").append_field("output")),
                    );
                    self.output_interface.wire(EndpointPath::default().append_field("output"), output_interface.clone())
                }
                ModuleValue::Function { .. } => todo!(),
                ModuleValue::Composite(cm) => match cm {
                    CompositeModuleArg::Tuple(ms) => {
                        for (i, m) in ms.iter().enumerate() {
                            match m {
                                ModuleValue::External(_) => todo!(),
                                ModuleValue::CallResult { submodule_index, output_interface, path } => {
                                    let submodule_input = &mut self.submodules[*submodule_index].1;
                                    submodule_input.wire(
                                        path.clone(),
                                        self.input_interface.get_subinterface(
                                            EndpointPath::default().append_field("output").append_field(&i.to_string()),
                                        ),
                                    );

                                    self.output_interface.wire(
                                        EndpointPath::default().append_field("output").append_field(&i.to_string()),
                                        output_interface.clone(),
                                    );
                                }
                                ModuleValue::Closure { .. } => todo!(),
                                ModuleValue::Function { .. } => todo!(),
                                ModuleValue::Composite(_) => todo!(),
                            }
                        }
                    }
                    CompositeModuleArg::Array(..) => todo!(),
                },
            }

            return Ok(());
        }

        panic!()
    }

    /// TODO: This is done separately since closures are in a different form considered to normal functions.
    /// But the implementation can be merged by some preprocessing.
    fn handle_closure_call(
        &mut self,
        args: &[ExprId],
        force_construction: Option<String>,
    ) -> Result<ModuleGraphValue<'tcx>, VirgenError> {
        let closure_arg = self.get_module_arg(args[0], force_construction.clone());
        if let Some(module) = closure_arg.module_arg() {
            let args: Vec<ModuleGraphValue<'tcx>> = match &self.thir_body.borrow()[self.skip_exprs(args[1])?].kind {
                ExprKind::Tuple { fields } => {
                    fields.iter().map(|arg| self.get_module_arg(*arg, force_construction.clone())).collect()
                }
                _ => panic!(),
            };
            match module {
                ModuleValue::External(path) => {
                    let input = args
                        .into_iter()
                        .map(|arg| match arg {
                            ModuleGraphValue::Interface(interface_arg) => match interface_arg {
                                InterfaceValue::ExternalInterface(path) => {
                                    self.output_interface.wire(path.clone(), Interface::Unit);
                                    self.input_interface.get_subinterface(path)
                                }
                                InterfaceValue::CallResultInterface(i) => i,
                            },
                            ModuleGraphValue::Module(_) => todo!(),
                            ModuleGraphValue::ConstantFunctionArgs(_) => todo!(),
                            ModuleGraphValue::Unit => Interface::Unit,
                        })
                        .collect::<Interface>();
                    let input = self.collect_interface(input, vec![], Interface::Unit);
                    self.output_interface.wire(path.clone(), input);
                    // TODO: If the return type of this module is module, we should put
                    // "ModuleArg::Module". For this case, below code will panic, so take
                    // care when needed.
                    Ok(InterfaceValue::call_result_interface(
                        self.input_interface
                            .get_subinterface(path.clone())
                            .get_subinterface(EndpointPath::default().append_field("output")),
                    )
                    .into())
                }
                ModuleValue::CallResult { submodule_index, output_interface, path } => {
                    for (arg_idx, arg) in args.iter().enumerate() {
                        match arg {
                            ModuleGraphValue::Interface(interface_arg) => match interface_arg {
                                InterfaceValue::ExternalInterface(path) => {
                                    self.output_interface.wire(path.clone(), Interface::Unit);
                                    self.submodules[*submodule_index].1.wire(
                                        EndpointPath::default()
                                            .append_field("output")
                                            .append_field("input")
                                            .append_field(&arg_idx.to_string()),
                                        self.input_interface.get_subinterface(path.clone()),
                                    );
                                }
                                InterfaceValue::CallResultInterface(i) => self.submodules[*submodule_index].1.wire(
                                    EndpointPath::default()
                                        .append_field("output")
                                        .append_field("input")
                                        .append_field(&arg_idx.to_string()),
                                    i.clone(),
                                ),
                            },
                            ModuleGraphValue::Module(_) => todo!(),
                            ModuleGraphValue::ConstantFunctionArgs(_) => todo!(),
                            ModuleGraphValue::Unit => {}
                        }
                    }

                    // TODO: If the return type of this module is module, we should put
                    // "ModuleArg::Module". For this case, below code will panic, so take
                    // care when needed.
                    Ok(InterfaceValue::call_result_interface(output_interface.get_subinterface(path.clone())).into())
                }
                _ => todo!("module: {module:#?}"),
            }
        } else {
            unreachable!()
        }
    }

    /// Construct a FSM.
    fn construct_fsm(
        &mut self,
        sig: ModuleSig<'tcx>,
        args: &[ExprId],
        force_construction: Option<String>,
    ) -> VirgenResult<ModuleGraphValue<'tcx>> {
        let [input_interface_id, init_value_id, fsm_logic_id] = args else { unreachable!() };

        let input_interface = match self.get_module_arg(*input_interface_id, force_construction.clone()) {
            ModuleGraphValue::Interface(interface_arg) => match interface_arg {
                InterfaceValue::ExternalInterface(path) => {
                    self.output_interface.wire(path.clone(), Interface::Unit);
                    self.input_interface.get_subinterface(path)
                }
                InterfaceValue::CallResultInterface(i) => i,
            },
            ModuleGraphValue::Unit => Interface::Unit,
            _ => panic!(),
        };
        let endpoint = submodule_output_interface(sig.output_interface_typ(), self.submodules.len())
            .get_subinterface(EndpointPath::default().append_field("output"));

        let input_interface = self.collect_interface(
            // TODO: just use get_args
            vec![input_interface, Interface::Unit, Interface::Unit].into_iter().collect(),
            vec![],
            Interface::Unit,
        );

        let init_span = self.thir_body.borrow()[self.skip_exprs(*init_value_id)?].span;

        let init_value = self.get_module_arg(*init_value_id, force_construction.clone());

        let fsm = Fsm {
            sig,
            // instance,
            // We add expr id to differentiate multiple fsm calls in same module.
            module_name: "fsm".to_string(),
            init_value: if let ModuleGraphValue::Unit = init_value {
                Expr::unit(init_span)
            } else {
                init_value.function_arg().unwrap().expr().unwrap()
            },
            fsm_logic: self
                .get_module_arg(*fsm_logic_id, force_construction.clone())
                .function_arg()
                .unwrap()
                .function()
                .unwrap(),
        };
        let edge = (fsm.into(), input_interface);

        self.submodules.push(edge);

        Ok(InterfaceValue::call_result_interface(endpoint).into())
    }

    fn closure_to_module_arg(
        &mut self,
        closure_expr: &ClosureExpr<'tcx>,
        ty: Ty<'tcx>,
        force_construction: Option<String>,
    ) -> ModuleGraphValue<'tcx> {
        let instance = self.ty_to_instance(ty).expect("TODO: take care when None");
        if let Some(sig) = ModuleSig::from_instance(self.tcx, self.meta, instance, self.sig.generic_map.clone().into())
        {
            // TODO: fix as construct_submodule
            log::debug!("sig: {sig:#?}");
            let mut input_interface = Interface::Unwired(sig.input_interface_typ());

            let upvars = closure_expr
                .upvars
                .iter()
                .map(|upvar| self.get_upvar(*upvar, force_construction.clone()))
                .collect::<Vec<_>>();
            let submodule_index = self.submodules.len(); // index of the module we are constructing
            let constructed_output_interface = submodule_output_interface(sig.output_interface_typ(), submodule_index);
            for (upvar_idx, (_, upvar_arg)) in upvars.iter().enumerate() {
                let captured_path =
                    EndpointPath::default().append_field("captured").append_field(&upvar_idx.to_string());

                if let Some(external_path) = upvar_arg.external_path() {
                    self.output_interface.wire(
                        external_path.clone(),
                        constructed_output_interface.get_subinterface(captured_path.clone()),
                    );
                    input_interface.wire(captured_path, self.input_interface.get_subinterface(external_path.clone()));
                } else if let Some(module) = upvar_arg.module_arg() {
                    match module {
                        ModuleValue::CallResult { submodule_index, output_interface, path } => {
                            let submodule_input = &mut self.submodules[*submodule_index].1;
                            submodule_input.wire(
                                path.clone(),
                                constructed_output_interface.get_subinterface(captured_path.clone()),
                            );

                            input_interface.wire(captured_path, output_interface.clone());
                        }
                        ModuleValue::Closure { .. } => todo!(),
                        ModuleValue::Function { .. } => todo!(),
                        ModuleValue::Composite(_) => todo!(),
                        ModuleValue::External(_) => panic!(),
                    }
                } else {
                    todo!()
                }
            }

            match sig.ret_ty.as_ref() {
                ModuleGraphType::Interface(_) => {
                    input_interface.wire(EndpointPath::default().append_field("output"), Interface::Unit)
                }
                ModuleGraphType::Module(_) => todo!(),
                ModuleGraphType::Misc(_) => todo!(),
                ModuleGraphType::ComposedModule(_) => todo!(),
            }

            let module = ModuleInst {
                inst_name: join_options("_", [Some(sig.name.clone()), Some("inst".to_string())]).unwrap(),
                instance,
                args: sig
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| match p {
                        ModuleGraphType::Interface(InterfaceTyp::Unit) => ModuleGraphValue::Unit,
                        ModuleGraphType::Interface(_) => InterfaceValue::external_interface(
                            EndpointPath::default().append_field("input").append_field(&i.to_string()),
                        )
                        .into(),
                        ModuleGraphType::Module(_) => ModuleValue::external_module(
                            EndpointPath::default().append_field("input").append_field(&i.to_string()),
                        )
                        .into(),
                        ModuleGraphType::Misc(_) => panic!(),
                        ModuleGraphType::ComposedModule(_) => todo!(),
                    })
                    .collect(),
                prefix: self.alloc_prefix(),
                // TODO: calculate parameters from const generic parameters
                params: vec![],
                upvars: Some(
                    sig.captured
                        .as_ref()
                        .unwrap()
                        .iter()
                        .zip_eq(upvars)
                        .enumerate()
                        .map(|(i, (param, upvar))| {
                            let external_arg = match param {
                                ModuleGraphType::Interface(InterfaceTyp::Unit) => ModuleGraphValue::Unit,
                                ModuleGraphType::Interface(_) => InterfaceValue::external_interface(
                                    EndpointPath::default().append_field("captured").append_field(&i.to_string()),
                                )
                                .into(),
                                ModuleGraphType::Module(_) => ModuleValue::external_module(
                                    EndpointPath::default().append_field("captured").append_field(&i.to_string()),
                                )
                                .into(),
                                ModuleGraphType::Misc(_) => {
                                    if let Some(module) = upvar.1.module_arg() {
                                        module.clone().into()
                                    } else {
                                        panic!("i: {i:#?}, param: {param:#?}, upvar: {upvar:#?}")
                                    }
                                }
                                ModuleGraphType::ComposedModule(_) => todo!(),
                            };
                            (upvar.0, external_arg)
                        })
                        .collect(),
                ),
                sig,
            };

            self.submodules.push((module.into(), input_interface));

            ModuleValue::closure_module(
                submodule_index,
                constructed_output_interface.swap_field("captured", Interface::Unit),
            )
            .into()
        } else {
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

            assert_eq!(closure_expr.closure_id, instance.def_id().expect_local());

            let upvars = closure_expr
                .upvars
                .iter()
                .map(|upvar| {
                    let (id, arg) = self.get_upvar(*upvar, force_construction.clone());
                    (id, arg.function_arg().unwrap())
                })
                .collect();

            ModuleGraphValue::ConstantFunctionArgs(PureValue::Function(FunctionBuilder::new_closure(
                instance, upvars, self.tcx,
            )))
        }
    }

    fn get_upvar(&mut self, arg: ExprId, force_construction: Option<String>) -> (Id, ModuleGraphValue<'tcx>) {
        match &self.thir_body.borrow().exprs[arg].kind {
            ExprKind::Scope { lint_level, .. } => match lint_level {
                thir::LintLevel::Inherited => todo!(),
                thir::LintLevel::Explicit(id) => (Id::Upvar(*id), self.get_module_arg(arg, force_construction)),
            },
            ExprKind::VarRef { id, .. } => (Id::Local(*id), self.get_module_arg(arg, force_construction)),
            ExprKind::UpvarRef { var_hir_id, .. } => {
                (Id::Local(*var_hir_id), self.get_module_arg(arg, force_construction))
            }
            ExprKind::Borrow { borrow_kind, arg } => {
                assert_eq!(*borrow_kind, BorrowKind::Shared);
                self.get_upvar(*arg, force_construction)
            }
            unimpl => todo!("{:?}", unimpl),
        }
    }

    fn construct_module_split(
        &mut self,
        sig: ModuleSig<'tcx>,
        args: &[ExprId],
        force_construction: Option<String>,
    ) -> VirgenResult<ModuleGraphValue<'tcx>> {
        let args = args.iter().map(|arg| self.get_module_arg(*arg, force_construction.clone())).collect::<Vec<_>>();
        let (unwired_input_interface, module_arg) = self.get_wired_input_interface(&sig, &args, None);

        let module_split = ModuleSplit { sig, module_name: "module_split".to_string() };

        self.submodules.push((module_split.into(), unwired_input_interface));

        Ok(module_arg)
    }

    #[allow(clippy::too_many_arguments)]
    fn construct_module_seq(
        &mut self,
        sig: ModuleSig<'tcx>,
        args: &[ExprId],
        force_construction: Option<String>,
    ) -> VirgenResult<ModuleGraphValue<'tcx>> {
        // Create inner modules
        // This will instantiate the unwired inner modules
        assert!(args.len() == 1);
        let seq_inner_modules = self.get_module_arg(args[0], force_construction);
        assert!(matches!(
            seq_inner_modules,
            ModuleGraphValue::Module(ModuleValue::Composite(CompositeModuleArg::Array(..)))
        ));
        // Now, inner moudles are instantiated.

        // In the `get_wired_input_interface` function below, we will do the following:
        // 1. Wire the input interface of the inner modules to the corresponding output interface of the ModuleSeq module.
        // 2. Wire the output interface of the inner modules to the corresponding inner interface of the the MoudleSeq module.
        let (unwired_input_interface, module_arg) = self.get_wired_input_interface(&sig, &[seq_inner_modules], None);

        let module_seq = ModuleSeq { sig, module_name: "module_seq".to_string() };

        self.submodules.push((module_seq.into(), unwired_input_interface.clone()));

        Ok(module_arg)
    }

    fn construct_from_fn(
        &mut self,
        n: usize,
        args: &[ExprId],
        force_construction: Option<String>,
    ) -> VirgenResult<ModuleGraphValue<'tcx>> {
        assert_eq!(args.len(), 1);
        let mut modules = vec![];

        // The `from_fn` duplicates the module `n` times, meaning that should get `n` instantiated modules.
        // When calling `get_module_arg`, if `force_construction` is not None, it will instantiate the module.
        // The first module is already instantiated, so we don't need to call `get_module_arg` with `Some` type of `force_construction`.
        // For n-1 modules, we call `get_module_arg` with `Some` type of `force_construction`.
        let first_module_arg = self.get_module_arg(args[0], force_construction.clone());
        modules.push(first_module_arg.module_arg().unwrap().clone());
        for idx in 1..n {
            let module = self.get_module_arg(
                args[0],
                join_options("_", [force_construction.clone(), Some("from_fn".to_string()), Some(idx.to_string())]),
            );
            modules.push(module.module_arg().unwrap().clone());
        }

        let array_module = ModuleGraphValue::Module(ModuleValue::Composite(CompositeModuleArg::Array(modules, n)));

        Ok(array_module)
    }

    fn construct_fn_ptr(
        &mut self,
        fn_ptr_id: ExprId,
        args: &[ExprId],
        force_construction: Option<String>,
    ) -> VirgenResult<ModuleGraphValue<'tcx>> {
        let module_arg = self.get_module_arg(fn_ptr_id, force_construction.clone());

        if let Some(module) = module_arg.module_arg() {
            match module {
                ModuleValue::Function { submodule_index, output_interface } => {
                    let submodule_index = *submodule_index;

                    let args = args
                        .iter()
                        .map(|arg| self.get_module_arg(*arg, force_construction.clone()))
                        .collect::<Vec<_>>();

                    let (node, edge) = self.submodules.get_mut(submodule_index).unwrap();
                    for (i, arg) in args.into_iter().enumerate() {
                        match arg {
                            ModuleGraphValue::Interface(interface_arg) => match interface_arg {
                                InterfaceValue::ExternalInterface(path) => {
                                    self.output_interface.wire(path.clone(), Interface::Unit);

                                    let interface = self.input_interface.get_subinterface(path);
                                    edge.wire(
                                        EndpointPath::default().append_field("input").append_field(&i.to_string()),
                                        interface,
                                    );
                                }
                                InterfaceValue::CallResultInterface(_interface) => {
                                    todo!()
                                }
                            },
                            ModuleGraphValue::Module(_) => todo!(),
                            ModuleGraphValue::ConstantFunctionArgs(_) => todo!(),
                            ModuleGraphValue::Unit => todo!(),
                        }
                    }

                    let sig = node.sig().clone();

                    let module_arg = match sig.ret_ty.as_ref() {
                        ModuleGraphType::Interface(_) => InterfaceValue::call_result_interface(
                            output_interface.get_subinterface(EndpointPath::default().append_field("output")),
                        ),
                        _ => todo!(),
                    };

                    Ok(module_arg.into())
                }
                ModuleValue::External(module_path) => {
                    let args = args
                        .iter()
                        .map(|arg| self.get_module_arg(*arg, force_construction.clone()))
                        .collect::<Vec<_>>();

                    for (i, arg) in args.into_iter().enumerate() {
                        match arg {
                            ModuleGraphValue::Interface(interface_arg) => match interface_arg {
                                InterfaceValue::ExternalInterface(_path) => {
                                    todo!()
                                }
                                InterfaceValue::CallResultInterface(interface) => {
                                    self.output_interface.wire(
                                        module_path.append_field("input").append_field(&i.to_string()),
                                        interface,
                                    );
                                }
                            },
                            ModuleGraphValue::Module(_) => todo!(),
                            ModuleGraphValue::ConstantFunctionArgs(_) => todo!(),
                            ModuleGraphValue::Unit => todo!(),
                        }
                    }

                    Ok(InterfaceValue::call_result_interface(
                        self.input_interface.get_subinterface(module_path.append_field("output")),
                    )
                    .into())
                }
                _ => todo!("{:#?}", module),
            }
        } else {
            todo!()
        }
    }

    fn is_closure(&self) -> bool {
        self.upvars.is_some()
    }
}

/// Collect all fsms given a synthesizable function id.
pub(crate) fn construct_submodule_graph<'tcx>(
    meta: &Meta,
    tcx: TyCtxt<'tcx>,
    module: &Virgen<'tcx>,
) -> VirgenResult<(Vec<ModuleGraphEdge<'tcx>>, Interface)> {
    let collecter = ModuleGraphConstructor {
        instance: module.instance,
        tcx,
        input_interface: input_interface(&module.input_interface_typ()),
        output_interface: Interface::Unwired(module.output_interface_typ()),
        thir_body: thir_body(tcx, module.instance.def_id().expect_local()),
        module_args: Default::default(),
        submodules: Default::default(),
        sig: &module.sig,
        args: &module.args,
        prefix: &module.prefix,
        upvars: module.upvars.as_deref(),
        meta,
    };

    log::info!("Constructing Submodule graph of {}", module.name());

    let graph = collecter.construct_graph()?;

    log::info!("Graph result of {}: {} submodules", module.name(), graph.0.len());
    for (module, in_interface) in &graph.0 {
        log::info!("Type checking submodule interfaces.. {:#?}", module.get_module_name());

        assert_eq!(module.input_interface_typ(), in_interface.typ(), "{:#?}", in_interface);
        assert!(!in_interface.contains_unwired(), "{:#?}", in_interface);
    }

    log::info!("Type checking output interface..");
    assert_eq!(module.output_interface_typ(), graph.1.typ());

    assert!(!graph.1.contains_unwired(), "{:#?}", graph.1);

    Ok(graph)
}
