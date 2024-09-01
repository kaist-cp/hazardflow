//! Integrates multiple verilog files into a top module.

use std::collections::{HashMap, HashSet};

use crate::compiler::prelude::Shape;
use crate::vir::utils::*;
use crate::vir::*;

/// Integrates multiple verilog files into one.
pub fn integrate(vir_modules: HashMap<String, Module>, top: String) -> Module {
    let vir_modules = vir_modules
        .into_iter()
        .map(|(name, module)| {
            if name == top {
                return (name, module);
            }

            let replaces = extract_decls(&module)
                .into_iter()
                .filter(|ident| ident != "clk" && ident != "rst")
                .map(|ident| (ident.clone(), format!("{}_{}", name, ident)))
                .collect::<HashMap<_, _>>();

            (name.clone(), module.replace(&replaces))
        })
        .collect::<HashMap<_, _>>();

    let top_vir_module = vir_modules.get(&top).unwrap();
    integrate_inner(top_vir_module, &vir_modules)
}

fn integrate_inner(module: &Module, vir_modules: &HashMap<String, Module>) -> Module {
    Module {
        name: module.name.clone(),
        port_decls: module.port_decls.clone(),
        module_items: module.module_items.iter().map(|item| integrate_inner_module_item(item, vir_modules)).collect(),
    }
}

fn integrate_inner_module_item(module_item: &ModuleItem, vir_modules: &HashMap<String, Module>) -> ModuleItem {
    match module_item {
        ModuleItem::ModuleInstantiation(module_inst) => {
            if let Some(vir_module) = vir_modules.get(&module_inst.module_name) {
                let decls = vir_module
                    .port_decls
                    .iter()
                    .filter_map(|port_decl| match port_decl {
                        PortDeclaration::Input(width, ident) => {
                            if ident == "clk" || ident == "rst" {
                                None
                            } else {
                                Some(Declaration::net(Shape::new([*width], false), ident.clone()))
                            }
                        }
                        PortDeclaration::Output(width, ident) => {
                            Some(Declaration::net(Shape::new([*width], false), ident.clone()))
                        }
                    })
                    .collect::<Vec<_>>();

                let conts = {
                    let inputs = vir_module
                        .port_decls
                        .iter()
                        .filter_map(|port_decl| {
                            if let PortDeclaration::Input(_, ident) = port_decl {
                                Some(ident.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<HashSet<_>>();

                    module_inst
                        .port_connections
                        .iter()
                        .filter(|(port_name, _)| port_name != "clk" && port_name != "rst")
                        .map(|(port_name, expr)| {
                            let port_name = format!("{}_{}", module_inst.module_name, port_name);

                            if inputs.contains(&port_name) {
                                ContinuousAssign(Expression::ident(port_name), expr.clone())
                            } else {
                                ContinuousAssign(expr.clone(), Expression::ident(port_name))
                            }
                        })
                        .collect::<Vec<_>>()
                };

                let vir_module = integrate_inner(vir_module, vir_modules);

                ModuleItem::Commented(
                    format!("Start of {}", module_inst.module_name),
                    Some(format!("End of {}", module_inst.module_name)),
                    [
                        vec![ModuleItem::Declarations(decls)],
                        vec![ModuleItem::ContinuousAssigns(conts)],
                        vir_module.module_items.clone(),
                    ]
                    .concat(),
                )
            } else {
                ModuleItem::ModuleInstantiation(module_inst.clone())
            }
        }
        ModuleItem::Commented(comment_before, comment_after, items) => ModuleItem::Commented(
            comment_before.clone(),
            comment_after.clone(),
            items.iter().map(|item| integrate_inner_module_item(item, vir_modules)).collect(),
        ),
        _ => module_item.clone(),
    }
}
