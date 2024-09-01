//! Module Split

use super::*;

/// Module Splitter
#[derive(Debug, Clone)]
pub(crate) struct ModuleSplit<'tcx> {
    /// Module Signature
    pub(crate) sig: ModuleSig<'tcx>,
    /// Module name.
    pub(crate) module_name: String,
}

impl<'tcx> PrimitiveModule for ModuleSplit<'tcx> {
    fn get_module_name(&self) -> String {
        self.module_name.clone()
    }

    fn input_interface_typ(&self) -> InterfaceTyp {
        self.sig.input_interface_typ()
    }

    fn output_interface_typ(&self) -> InterfaceTyp {
        self.sig.output_interface_typ()
    }
}
