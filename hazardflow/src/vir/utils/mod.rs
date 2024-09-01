//! Utility functions for Verilog IR.

mod replace;

pub(crate) use replace::*;

use super::*;

/// Extracts declarations in module.
pub(crate) fn extract_decls(module: &Module) -> Vec<String> {
    let port_decls = module
        .port_decls
        .iter()
        .map(|decl| match decl {
            PortDeclaration::Input(_, name) | PortDeclaration::Output(_, name) => name.clone(),
        })
        .collect::<Vec<_>>();

    let decls = module
        .module_items
        .iter()
        .map(|item| match item {
            ModuleItem::Declarations(decls) => decls.iter().map(|decl| decl.ident().to_string()).collect::<Vec<_>>(),
            ModuleItem::Commented(_, _, items) => {
                items.iter().map(extract_decls_module_item).collect::<Vec<_>>().concat()
            }
            _ => vec![],
        })
        .collect::<Vec<_>>()
        .concat();

    [port_decls, decls].concat()
}

/// Extract declarations in module item.
fn extract_decls_module_item(module_item: &ModuleItem) -> Vec<String> {
    match module_item {
        ModuleItem::Declarations(decls) => decls.iter().map(|decl| decl.ident().to_string()).collect::<Vec<_>>(),
        ModuleItem::Commented(_, _, items) => items.iter().map(extract_decls_module_item).collect::<Vec<_>>().concat(),
        _ => vec![],
    }
}
