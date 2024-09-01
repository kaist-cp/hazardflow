//! Module.

mod module_ffi;
mod module_fsm;
mod module_inst;
mod module_seq;
mod module_split;
mod signature;

use std::fmt;
use std::rc::Rc;

pub(crate) use module_ffi::*;
pub(crate) use module_fsm::*;
pub(crate) use module_inst::*;
pub(crate) use module_seq::*;
pub(crate) use module_split::*;
use rustc_middle::ty::{Ty, TyCtxt};
pub(crate) use signature::*;

use super::*;

/// Primitive modules.
pub trait PrimitiveModule: fmt::Debug {
    /// Returns module name.
    fn get_module_name(&self) -> String;

    /// Returns input interface.
    fn input_interface_typ(&self) -> InterfaceTyp;

    /// Returns output interface.
    fn output_interface_typ(&self) -> InterfaceTyp;
}

/// Module
#[derive(Debug, Clone)]
pub(crate) enum ModuleInner<'tcx> {
    /// Fsm
    Fsm(Fsm<'tcx>),

    /// Module Instantiation
    ModuleInst(ModuleInst<'tcx>),

    /// FFI
    Ffi(Ffi<'tcx>),

    /// Module Split
    ModuleSplit(ModuleSplit<'tcx>),

    /// Module Sequencer
    ModuleSeq(ModuleSeq<'tcx>),
}

#[derive(Debug, Clone)]
pub(crate) struct Module<'tcx> {
    /// Module Inner
    pub(crate) inner: Rc<ModuleInner<'tcx>>,
}

impl<'tcx> Module<'tcx> {
    pub(crate) fn module_inst(self) -> Option<ModuleInst<'tcx>> {
        match &*self.inner {
            ModuleInner::ModuleInst(m) => Some(m.clone()),
            ModuleInner::Fsm(_) => None,
            ModuleInner::Ffi(_) => None,
            ModuleInner::ModuleSplit(_) => None,
            ModuleInner::ModuleSeq(_) => todo!(),
        }
    }

    /// Returns module name.
    pub(crate) fn get_module_name(&self) -> String {
        match &*self.inner {
            ModuleInner::Fsm(module) => module.get_module_name(),
            ModuleInner::ModuleInst(module) => module.get_module_name(),
            ModuleInner::Ffi(module) => module.get_module_name(),
            ModuleInner::ModuleSplit(module) => module.get_module_name(),
            ModuleInner::ModuleSeq(module) => module.get_module_name(),
        }
    }

    /// Returns input interface type
    pub(crate) fn input_interface_typ(&self) -> InterfaceTyp {
        match &*self.inner {
            ModuleInner::Fsm(fsm) => fsm.input_interface_typ(),
            ModuleInner::ModuleInst(module_inst) => module_inst.input_interface_typ(),
            ModuleInner::Ffi(ffi) => ffi.input_interface_typ(),
            ModuleInner::ModuleSplit(module) => module.input_interface_typ(),
            ModuleInner::ModuleSeq(module) => module.input_interface_typ(),
        }
    }

    /// Returns output interface type
    pub(crate) fn output_interface_typ(&self) -> InterfaceTyp {
        match &*self.inner {
            ModuleInner::Fsm(fsm) => fsm.output_interface_typ(),
            ModuleInner::ModuleInst(module_inst) => module_inst.output_interface_typ(),
            ModuleInner::Ffi(ffi) => ffi.output_interface_typ(),
            ModuleInner::ModuleSplit(module) => module.output_interface_typ(),
            ModuleInner::ModuleSeq(module) => module.output_interface_typ(),
        }
    }

    /// Returns module signature
    pub(crate) fn sig(&self) -> ModuleSig<'tcx> {
        match &*self.inner {
            ModuleInner::Fsm(_) => todo!(),
            ModuleInner::ModuleInst(module_inst) => module_inst.sig.clone(),
            ModuleInner::Ffi(_) => todo!(),
            ModuleInner::ModuleSplit(_) => todo!(),
            ModuleInner::ModuleSeq(_) => todo!(),
        }
    }
}

impl<'tcx> From<Fsm<'tcx>> for Module<'tcx> {
    fn from(module: Fsm<'tcx>) -> Module<'tcx> {
        Module { inner: Rc::new(ModuleInner::Fsm(module)) }
    }
}

impl<'tcx> From<ModuleInst<'tcx>> for Module<'tcx> {
    fn from(module: ModuleInst<'tcx>) -> Module<'tcx> {
        Module { inner: Rc::new(ModuleInner::ModuleInst(module)) }
    }
}

impl<'tcx> From<Ffi<'tcx>> for Module<'tcx> {
    fn from(module: Ffi<'tcx>) -> Module<'tcx> {
        Module { inner: Rc::new(ModuleInner::Ffi(module)) }
    }
}

impl<'tcx> From<ModuleSplit<'tcx>> for Module<'tcx> {
    fn from(module: ModuleSplit<'tcx>) -> Module<'tcx> {
        Module { inner: Rc::new(ModuleInner::ModuleSplit(module)) }
    }
}

impl<'tcx> From<ModuleSeq<'tcx>> for Module<'tcx> {
    fn from(module: ModuleSeq<'tcx>) -> Module<'tcx> {
        Module { inner: Rc::new(ModuleInner::ModuleSeq(module)) }
    }
}
