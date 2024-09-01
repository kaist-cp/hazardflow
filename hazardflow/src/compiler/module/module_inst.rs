//! Module Instantiation.

use rustc_middle::ty::Instance;
use rustc_span::DUMMY_SP;

use super::*;
use crate::utils::*;

/// Module Arguement
#[derive(Debug, Clone)]
pub(crate) enum ModuleGraphValue<'tcx> {
    /// Interface
    Interface(InterfaceValue),

    /// Module
    Module(ModuleValue),

    /// Constant Function Arguement
    ///
    /// This can be either:
    /// - Pure function
    /// - Constant `Expr`
    ConstantFunctionArgs(PureValue<'tcx>),

    /// TODO: documentation
    Unit,
}

/// Interface
#[derive(Debug, Clone)]
pub(crate) enum InterfaceValue {
    /// External Interface
    ExternalInterface(EndpointPath),

    /// Interface that is created as a result of a call
    CallResultInterface(Interface),
}

impl InterfaceValue {
    pub(crate) fn external_interface(path: EndpointPath) -> Self {
        InterfaceValue::ExternalInterface(path)
    }

    pub(crate) fn call_result_interface(interface: Interface) -> Self {
        InterfaceValue::CallResultInterface(interface)
    }
}

impl<'tcx> From<InterfaceValue> for ModuleGraphValue<'tcx> {
    fn from(value: InterfaceValue) -> Self {
        ModuleGraphValue::Interface(value)
    }
}

/// Module
#[derive(Debug, Clone)]
pub(crate) enum ModuleValue {
    /// External Module
    ///
    /// We do not use `ModuleSig` on purpose to hide the details of the module given as the arguement.
    /// The user of this module arguement should use this as a black box module with given I/O.
    External(EndpointPath),

    /// Module that is created as a result of a call
    CallResult { submodule_index: usize, output_interface: Interface, path: EndpointPath },

    /// Closure Module
    Closure { submodule_index: usize, output_interface: Interface },

    /// Function Module
    Function { submodule_index: usize, output_interface: Interface },

    /// Composite Module Arg
    Composite(CompositeModuleArg),
}

impl ModuleValue {
    pub(crate) fn external_module(path: EndpointPath) -> Self {
        ModuleValue::External(path)
    }

    pub(crate) fn call_result_module(submodule_index: usize, output_interface: Interface, path: EndpointPath) -> Self {
        ModuleValue::CallResult { submodule_index, output_interface, path }
    }

    pub(crate) fn closure_module(submodule_index: usize, output_interface: Interface) -> Self {
        ModuleValue::Closure { submodule_index, output_interface }
    }

    pub(crate) fn function_module(submodule_index: usize, output_interface: Interface) -> Self {
        ModuleValue::Function { submodule_index, output_interface }
    }

    pub(crate) fn composite_module(composite_module: CompositeModuleArg) -> Self {
        ModuleValue::Composite(composite_module)
    }
}

impl<'tcx> From<ModuleValue> for ModuleGraphValue<'tcx> {
    fn from(value: ModuleValue) -> Self {
        Self::Module(value)
    }
}

/// Composition of Module Arguements
#[derive(Debug, Clone)]
pub(crate) enum CompositeModuleArg {
    /// Tuple of Module Arguements
    Tuple(Vec<ModuleValue>),

    /// Array of Module Arguments
    /// Note: Difference from `Tuple`: Array is accessed by index, while Tuple is accessed by field.
    Array(Vec<ModuleValue>, usize),
}

impl<'tcx> ModuleGraphValue<'tcx> {
    // Get the function arg
    pub(crate) fn function_arg(&self) -> Option<PureValue<'tcx>> {
        match self {
            ModuleGraphValue::ConstantFunctionArgs(arg) => Some(arg.clone()),
            ModuleGraphValue::Unit => Some(PureValue::Expr(Expr::unit(DUMMY_SP))),
            _ => None,
        }
    }

    // Get the interface arg
    pub(crate) fn interface_arg(&self) -> Option<&InterfaceValue> {
        match self {
            ModuleGraphValue::Interface(arg) => Some(arg),
            _ => None,
        }
    }

    // Get the module arg
    pub(crate) fn module_arg(&self) -> Option<&ModuleValue> {
        match self {
            ModuleGraphValue::Module(arg) => Some(arg),
            _ => None,
        }
    }

    pub(crate) fn external_path(&self) -> Option<EndpointPath> {
        match self {
            ModuleGraphValue::Module(ModuleValue::External(path))
            | ModuleGraphValue::Interface(InterfaceValue::ExternalInterface(path)) => Some(path.clone()),
            _ => None,
        }
    }
}

/// Module Instantiation
#[derive(Debug, Clone)]
pub(crate) struct ModuleInst<'tcx> {
    /// Monomorphized rust function instance
    pub(crate) instance: Instance<'tcx>,
    /// Module Signature
    pub(crate) sig: ModuleSig<'tcx>,
    /// Arguements
    pub(crate) args: Vec<ModuleGraphValue<'tcx>>,
    /// Prefix
    pub(crate) prefix: Vec<String>,
    /// Instance name.
    pub(crate) inst_name: String,
    /// Module parameters.
    pub(crate) params: Vec<(String, usize)>,
    /// Upvars
    pub(crate) upvars: Option<Vec<(Id, ModuleGraphValue<'tcx>)>>,
}

impl<'tcx> PrimitiveModule for ModuleInst<'tcx> {
    fn get_module_name(&self) -> String {
        self.sig.name.clone()
    }

    fn input_interface_typ(&self) -> InterfaceTyp {
        self.sig.input_interface_typ()
    }

    fn output_interface_typ(&self) -> InterfaceTyp {
        self.sig.output_interface_typ()
    }
}
