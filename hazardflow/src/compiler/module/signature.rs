//! Module Signature.

use linked_hash_map::LinkedHashMap;
use rustc_middle::ty::{ClosureArgs, Instance, ParamEnv};

use super::*;
use crate::utils::*;

#[derive(Debug, Clone)]
pub(crate) struct ModuleSig<'tcx> {
    /// Name
    pub(crate) name: String,

    /// Module Parameters
    pub(crate) params: Vec<ModuleGraphType<'tcx>>,

    /// Captured interfaces
    pub(crate) captured: Option<Vec<ModuleGraphType<'tcx>>>,

    /// Module Return Type
    pub(crate) ret_ty: Box<ModuleGraphType<'tcx>>,

    /// Generic Map for this module, which has been passed down from the top level module.
    pub(crate) generic_map: GenericMap<'tcx>,
}

impl<'tcx> ModuleSig<'tcx> {
    fn from_fn_sig(
        sig: rustc_middle::ty::FnSig<'tcx>,
        tcx: TyCtxt<'tcx>,
        meta: &Meta,
        name: String,
        generic_map: GenericMap<'tcx>,
    ) -> Option<Self> {
        let inputs =
            sig.inputs().iter().map(|ty: &Ty<'tcx>| ModuleGraphType::from_ty(tcx, meta, &generic_map, *ty)).collect();

        let output = sig.output();
        let ret_ty = ModuleGraphType::from_ty(tcx, meta, &generic_map, output);
        if matches!(ret_ty, ModuleGraphType::Misc(_)) {
            log::debug!("returning!: {:?}", ret_ty);
            return None;
        }
        ModuleSig { params: inputs, captured: None, ret_ty: ret_ty.into(), generic_map, name }.into()
    }

    pub(crate) fn from_instance(
        tcx: TyCtxt<'tcx>,
        meta: &Meta,
        instance: Instance<'tcx>,
        generic_map: Option<GenericMap<'tcx>>,
    ) -> Option<Self> {
        log::debug!("sig from_instance: {:#?}", instance);
        let ty = instance.ty(tcx, ParamEnv::empty());

        let generic_map = generic_map.unwrap_or_else(|| get_generic_map(tcx, instance));

        let module = match ty.kind() {
            rustc_type_ir::TyKind::FnDef(id, substs) => {
                let sig = tcx.type_of(id).instantiate(tcx, substs).fn_sig(tcx).skip_binder();

                log::debug!("sig: {:#?}", sig);

                Self::from_fn_sig(sig, tcx, meta, tcx.item_name(instance.def_id()).to_ident_string(), generic_map)?
            }
            rustc_type_ir::TyKind::Closure(_, args) => {
                let closure: ClosureArgs<'tcx> = args.as_closure();

                let sig = closure.sig().skip_binder();

                assert_eq!(sig.inputs().len(), 1);

                // NOTE: We unwrap the tuple because closure args are passed after being all wrapped as a tuple.
                let inputs = match sig.inputs()[0].kind() {
                    rustc_type_ir::TyKind::Tuple(inner) => {
                        inner.iter().map(|ty| ModuleGraphType::from_ty(tcx, meta, &generic_map, ty)).collect()
                    }
                    _ => panic!(),
                };

                let output = sig.output();

                let captured = closure
                    .upvar_tys()
                    .iter()
                    .map(|ty| ModuleGraphType::from_ty(tcx, meta, &generic_map, ty))
                    .collect::<Vec<_>>();
                let ret_ty = ModuleGraphType::from_ty(tcx, meta, &generic_map, output);
                if matches!(ret_ty, ModuleGraphType::Misc(_)) {
                    return None;
                }
                ModuleSig {
                    params: inputs,
                    captured: Some(captured),
                    ret_ty: ret_ty.into(),
                    generic_map,
                    name: "closure".to_string(),
                }
            }
            _ => panic!(),
        };

        if module.is_valid_module() {
            Some(module)
        } else {
            None
        }
    }

    pub(crate) fn input_interface_typ(&self) -> InterfaceTyp {
        let mut input_interface_types = vec![];
        for param in &self.params {
            input_interface_types.push(param.input_interface_typ());
        }

        let mut captured_interfaces = vec![];
        if let Some(captured) = self.captured.as_ref() {
            for captured in captured {
                captured_interfaces.push(captured.input_interface_typ());
            }
        }

        let output_interface_type = self.ret_ty.as_ref().output_interface_typ();

        let mut interface_struct = LinkedHashMap::new();
        interface_struct.insert("input".to_string(), (None, input_interface_types.into_iter().collect()));
        interface_struct.insert("captured".to_string(), (None, captured_interfaces.into_iter().collect()));
        interface_struct.insert("output".to_string(), (None, output_interface_type));

        InterfaceTyp::Struct(interface_struct)
    }

    // TODO: fix struct
    pub(crate) fn output_interface_typ(&self) -> InterfaceTyp {
        let mut input_interface_types = vec![];
        for param in &self.params {
            input_interface_types.push(param.output_interface_typ());
        }

        let mut captured_interfaces = vec![];
        if let Some(captured) = self.captured.as_ref() {
            for captured in captured {
                captured_interfaces.push(captured.output_interface_typ());
            }
        }

        let output_interface_type = self.ret_ty.as_ref().input_interface_typ();

        let mut interface_struct = LinkedHashMap::new();
        interface_struct.insert("input".to_string(), (None, input_interface_types.into_iter().collect()));
        interface_struct.insert("captured".to_string(), (None, captured_interfaces.into_iter().collect()));
        interface_struct.insert("output".to_string(), (None, output_interface_type));

        InterfaceTyp::Struct(interface_struct)
    }

    /// Returns true if the module is valid, i.e. contains any channels in either input or output
    /// interface
    pub(crate) fn is_valid_module(&self) -> bool {
        self.input_interface_typ().contains_channel() || self.output_interface_typ().contains_channel()
    }
}

/// Module
#[derive(Debug, Clone)]
pub(crate) enum ModuleGraphType<'tcx> {
    /// Interface
    Interface(InterfaceTyp),

    /// Module
    Module(ModuleSig<'tcx>),

    /// Composed Module Parameter
    ///
    /// TODO: Maybe integrate this into `ModuleFunction`?
    ComposedModule(ComposedModuleTy<'tcx>),

    /// Misc
    Misc(Ty<'tcx>),
}

/// Composed Module Parameter
#[derive(Debug, Clone)]
pub(crate) enum ComposedModuleTy<'tcx> {
    /// Tuple of Module Parameters
    Tuple(Vec<ModuleGraphType<'tcx>>),

    /// Array of Module Parameters
    Array(Box<ModuleGraphType<'tcx>>, usize),
}

impl<'tcx> ModuleGraphType<'tcx> {
    /// Incoming interface type, regarding this parameter is a function parameter or captured parameter of a closure).
    pub(crate) fn input_interface_typ(&self) -> InterfaceTyp {
        match self {
            ModuleGraphType::Interface(i) => i.clone(),
            ModuleGraphType::Module(m) => m.output_interface_typ(),
            ModuleGraphType::Misc(_) => InterfaceTyp::Unit,
            ModuleGraphType::ComposedModule(composed_module_param) => match composed_module_param {
                ComposedModuleTy::Tuple(params) => {
                    let mut inner = LinkedHashMap::new();
                    for (i, param) in params.iter().enumerate() {
                        inner.insert(i.to_string(), (None, param.input_interface_typ()));
                    }
                    InterfaceTyp::Struct(inner)
                }
                ComposedModuleTy::Array(param, len) => {
                    let inner_typ = param.input_interface_typ();
                    InterfaceTyp::Array(Box::new(inner_typ), *len)
                }
            },
        }
    }

    /// Outgoing interface type, regarding this parameter is a function parameter or captured parameter of a closure).
    pub(crate) fn output_interface_typ(&self) -> InterfaceTyp {
        match self {
            ModuleGraphType::Interface(_) => InterfaceTyp::Unit,
            ModuleGraphType::Module(m) => m.input_interface_typ(),
            ModuleGraphType::Misc(_) => InterfaceTyp::Unit,
            ModuleGraphType::ComposedModule(composed_module_param) => match composed_module_param {
                ComposedModuleTy::Tuple(params) => {
                    let mut inner = LinkedHashMap::new();
                    for (i, param) in params.iter().enumerate() {
                        inner.insert(i.to_string(), (None, param.output_interface_typ()));
                    }
                    InterfaceTyp::Struct(inner)
                }
                ComposedModuleTy::Array(param, len) => {
                    let inner_typ = param.output_interface_typ();
                    InterfaceTyp::Array(Box::new(inner_typ), *len)
                }
            },
        }
    }

    fn from_ty(tcx: TyCtxt<'tcx>, meta: &Meta, generic_map: &GenericMap<'tcx>, ty: Ty<'tcx>) -> Self {
        log::debug!("ModuleParam::from_ty: {:#?}", normalize_alias_ty(tcx, ty).kind());
        let ty = normalize_alias_ty(tcx, ty);

        match ty.kind() {
            rustc_type_ir::TyKind::Param(_) => match generic_map.get(ty).unwrap() {
                GenericBound::Function { input, output } => {
                    let params = match input.kind() {
                        rustc_type_ir::TyKind::Tuple(inner) => {
                            inner.iter().map(|ty| Self::from_ty(tcx, meta, generic_map, ty)).collect::<Vec<_>>()
                        }
                        _ => panic!("{input:?} -> {output:?}"),
                    };
                    Self::Module(ModuleSig {
                        name: "blackbox".to_string(),
                        params,
                        captured: None,
                        ret_ty: Self::from_ty(tcx, meta, generic_map, *output).into(),
                        generic_map: generic_map.clone(),
                    })
                }
                GenericBound::Const(_) => todo!(),
            },
            rustc_type_ir::TyKind::FnDef(id, substs) => {
                let instance = Instance::resolve(tcx, ParamEnv::empty(), *id, substs).unwrap().unwrap();
                ModuleSig::from_instance(tcx, meta, instance, generic_map.clone().into())
                    .map(Self::Module)
                    .unwrap_or_else(|| Self::Misc(ty))
            }
            rustc_type_ir::TyKind::Closure(def_id, substs) => {
                let instance = Instance::resolve(tcx, ParamEnv::empty(), *def_id, substs).unwrap().unwrap();
                ModuleSig::from_instance(tcx, meta, instance, generic_map.clone().into())
                    .map(|mut sig| {
                        // TODO: documentation
                        sig.captured = None;
                        sig
                    })
                    .map(Self::Module)
                    .unwrap_or_else(|| Self::Misc(ty))
            }
            rustc_type_ir::TyKind::Alias(kind, alias) => match kind {
                rustc_type_ir::AliasKind::Opaque => match tcx.try_expand_impl_trait_type(alias.def_id, alias.args) {
                    Ok(expanded_ty) => ModuleGraphType::from_ty(tcx, meta, generic_map, expanded_ty),
                    Err(_) => todo!(),
                },
                _ => panic!(),
            },
            rustc_type_ir::TyKind::FnPtr(sig) => match sig.no_bound_vars() {
                Some(sig) => {
                    let sig = ModuleSig::from_fn_sig(sig, tcx, meta, "fn_ptr".to_string(), generic_map.clone())
                        .expect("Expect synthisizable function");

                    log::debug!("fn_ptr: {:#?}", sig);

                    Self::Module(sig)
                }
                None => todo!(),
            },
            rustc_type_ir::TyKind::Tuple(inner) => {
                if let Ok(interface) = InterfaceTyp::from_ty(ty, meta.interface_did(), tcx) {
                    return Self::Interface(interface);
                }

                let params = inner.iter().map(|ty| Self::from_ty(tcx, meta, generic_map, ty)).collect::<Vec<_>>();

                if params.iter().all(|param| matches!(param, Self::Misc(_))) {
                    return Self::Misc(ty);
                }

                Self::ComposedModule(ComposedModuleTy::Tuple(params))
            }
            rustc_type_ir::TyKind::Array(elem_ty, len) => {
                if let Ok(interface) = InterfaceTyp::from_ty(ty, meta.interface_did(), tcx) {
                    // If the inner type is an interface, return as it is.
                    return Self::Interface(interface);
                }

                if PortDecls::from_ty(ty, tcx).is_some() {
                    // If the inner type is a port, return as it is.
                    return Self::Misc(ty);
                }

                let elem_ty = Self::from_ty(tcx, meta, generic_map, *elem_ty);

                assert!(
                    matches!(elem_ty, ModuleGraphType::Module(_) | ModuleGraphType::ComposedModule(_)),
                    "elem_ty: {elem_ty:#?}"
                );

                let len = len.eval_target_usize(tcx, ParamEnv::empty()) as usize;

                Self::ComposedModule(ComposedModuleTy::Array(Box::new(elem_ty), len))
            }
            maybe_interface_ty => {
                log::debug!("maybe_interface_ty: {:?}", maybe_interface_ty);
                log::debug!("to inter: {:?}", InterfaceTyp::from_ty(ty, meta.interface_did(), tcx));

                InterfaceTyp::from_ty(ty, meta.interface_did(), tcx)
                    .map(Self::Interface)
                    .unwrap_or_else(|_| Self::Misc(ty))
            }
        }
    }
}
