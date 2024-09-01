//! FFI

use super::*;

/// Fsm
///
/// It contains the interface endpoint from which the fsm starts, the interface endpoint to which it goes,
#[derive(Debug, Clone)]
pub(crate) struct Ffi<'tcx> {
    /// Module Signature
    pub(crate) sig: ModuleSig<'tcx>,
    /// Module name.
    pub(crate) module_name: String,
    /// Instance name.
    pub(crate) inst_name: String,
    /// Module parameters.
    pub(crate) params: Vec<(String, usize)>,
}

impl<'tcx> PrimitiveModule for Ffi<'tcx> {
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
