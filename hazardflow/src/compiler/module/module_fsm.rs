//! Fsm

use super::*;

/// Fsm
///
/// It contains the interface endpoint from which the fsm starts, the interface endpoint to which it goes,
#[derive(Debug, Clone)]
pub(crate) struct Fsm<'tcx> {
    /// Module Signature
    pub(crate) sig: ModuleSig<'tcx>,
    /// Module name.
    pub(crate) module_name: String,
    /// Init value
    pub(crate) init_value: ExprId,
    /// Fsm logic
    pub(crate) fsm_logic: FunctionBuilder<'tcx>,
}

impl<'tcx> PrimitiveModule for Fsm<'tcx> {
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
