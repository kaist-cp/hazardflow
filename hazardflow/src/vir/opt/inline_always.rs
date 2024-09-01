use std::collections::HashSet;

use crate::vir::*;

trait OptimizeInlineAlways {
    /// Optimizes by inlining always block.
    fn optimize(&self, removed: &mut HashSet<String>) -> Self;
}

fn extract_lhs_ident_from_indexing_expr(expr: &Expression) -> Option<String> {
    if let Expression::Primary(Primary::HierarchicalIdentifier(ident, Some(_))) = expr {
        Some(ident.clone())
    } else {
        None
    }
}

fn extract_lhs_ident_from_expr(expr: &Expression) -> Option<String> {
    if let Expression::Primary(Primary::HierarchicalIdentifier(ident, _)) = expr {
        Some(ident.clone())
    } else {
        None
    }
}

fn extract_lhs_idents_from_stmts(stmts: &[Statement]) -> Vec<String> {
    stmts
        .iter()
        .map(|stmt| match stmt {
            Statement::BlockingAssignment(lhs, ..) => {
                if let Some(ident) = extract_lhs_ident_from_expr(lhs) {
                    vec![ident]
                } else {
                    vec![]
                }
            }
            Statement::Conditional(cond_expr_pairs, else_stmt, _) => [
                cond_expr_pairs
                    .iter()
                    .map(|(_, stmts)| extract_lhs_idents_from_stmts(stmts))
                    .collect::<Vec<_>>()
                    .concat(),
                extract_lhs_idents_from_stmts(else_stmt),
            ]
            .concat(),
            Statement::Loop(_, _, stmts, _) => extract_lhs_idents_from_stmts(stmts),
            Statement::NonblockingAssignment(lhs, ..) => {
                if let Some(ident) = extract_lhs_ident_from_expr(lhs) {
                    vec![ident]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        })
        .collect::<Vec<_>>()
        .concat()
}

fn extract_rhs_from_stmts(stmts: &[Statement]) -> Vec<Expression> {
    stmts
        .iter()
        .filter_map(|stmt| match stmt {
            Statement::BlockingAssignment(_, rhs, _) => Some(rhs.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
}

impl OptimizeInlineAlways for Vec<ModuleItem> {
    fn optimize(&self, removed: &mut HashSet<String>) -> Self {
        let temp = self
            .iter()
            .map(|module_item| match module_item {
                ModuleItem::Declarations(decls) => vec![ModuleItem::Declarations(decls.clone())],
                ModuleItem::ContinuousAssigns(conts) => {
                    vec![ModuleItem::ContinuousAssigns(conts.clone())]
                }
                ModuleItem::ModuleInstantiation(module_inst) => {
                    vec![ModuleItem::ModuleInstantiation(module_inst.clone())]
                }
                ModuleItem::AlwaysConstruct(event, stmts) => {
                    if event == "always @*" {
                        let mut conts = Vec::new();

                        let preserved: HashSet<String> = stmts
                            .iter()
                            .map(|stmt| match stmt {
                                Statement::BlockingAssignment(lhs, ..) => {
                                    if let Some(ident) = extract_lhs_ident_from_indexing_expr(lhs) {
                                        vec![ident]
                                    } else {
                                        vec![]
                                    }
                                }
                                Statement::Loop(_, count, stmts, _) => [
                                    if let Some(ident) = extract_lhs_ident_from_expr(count) {
                                        vec![ident]
                                    } else {
                                        vec![]
                                    },
                                    extract_lhs_idents_from_stmts(stmts),
                                ]
                                .concat(),
                                Statement::NonblockingAssignment(lhs, ..) => {
                                    if let Some(ident) = extract_lhs_ident_from_expr(lhs) {
                                        vec![ident]
                                    } else {
                                        vec![]
                                    }
                                }
                                _ => vec![],
                            })
                            .collect::<Vec<_>>()
                            .concat()
                            .into_iter()
                            .collect::<HashSet<_>>();

                        let stmts = stmts
                            .iter()
                            .filter_map(|stmt| match stmt {
                                Statement::BlockingAssignment(lhs, rhs, _) => {
                                    if let Some(ident) = extract_lhs_ident_from_expr(lhs) {
                                        if preserved.get(&ident).is_some() {
                                            Some(stmt.clone())
                                        } else {
                                            removed.insert(ident);
                                            conts.push(ContinuousAssign(lhs.clone(), rhs.clone()));
                                            None
                                        }
                                    } else {
                                        Some(stmt.clone())
                                    }
                                }
                                Statement::Conditional(cond_expr_pairs, else_stmt, _) => {
                                    let idents = extract_lhs_idents_from_stmts(else_stmt);
                                    let init_exprs = extract_rhs_from_stmts(else_stmt);

                                    idents.iter().for_each(|ident| {
                                        removed.insert(ident.clone());
                                    });

                                    let exprs = cond_expr_pairs.iter().rev().fold(init_exprs, |acc, (cond, stmts)| {
                                        let then_exprs = extract_rhs_from_stmts(stmts);
                                        acc.into_iter()
                                            .zip(then_exprs)
                                            .map(|(else_expr, then_expr)| {
                                                Expression::conditional(cond.clone(), then_expr, else_expr)
                                            })
                                            .collect()
                                    });

                                    idents.into_iter().zip(exprs).for_each(|(lhs, rhs)| {
                                        conts.push(ContinuousAssign(Expression::ident(lhs), rhs))
                                    });

                                    None
                                }
                                Statement::NonblockingAssignment(lhs, rhs, _) => {
                                    if let Some(ident) = extract_lhs_ident_from_expr(lhs) {
                                        if preserved.get(&ident).is_some() {
                                            Some(stmt.clone())
                                        } else {
                                            conts.push(ContinuousAssign(lhs.clone(), rhs.clone()));
                                            None
                                        }
                                    } else {
                                        Some(stmt.clone())
                                    }
                                }
                                _ => Some(stmt.clone()),
                            })
                            .collect::<Vec<_>>();

                        vec![ModuleItem::ContinuousAssigns(conts), ModuleItem::AlwaysConstruct(event.clone(), stmts)]
                    } else {
                        vec![ModuleItem::AlwaysConstruct(event.clone(), stmts.clone())]
                    }
                }
                ModuleItem::Commented(comment_before, comment_after, items) => {
                    let items = items.optimize(removed);

                    vec![ModuleItem::Commented(comment_before.clone(), comment_after.clone(), items)]
                }
            })
            .collect::<Vec<_>>()
            .concat();

        // Remove unnecessary regs.
        temp.iter()
            .map(|module_item| match module_item {
                ModuleItem::Declarations(decls) => {
                    let decls = decls
                        .iter()
                        .map(|decl| {
                            if let Declaration::Reg(shape, ident, init) = decl {
                                if removed.get(ident).is_some() {
                                    assert!(init.is_none());
                                    Declaration::Net(shape.clone(), ident.clone())
                                } else {
                                    decl.clone()
                                }
                            } else {
                                decl.clone()
                            }
                        })
                        .collect();

                    ModuleItem::Declarations(decls)
                }
                _ => module_item.clone(),
            })
            .collect()
    }
}

/// Optimizes module by using wire cache.
///
/// Wires in port declarations will not removed.
pub fn inline_always(module: Module) -> Module {
    let module_items = module.module_items;
    let port_decls = module.port_decls;

    let module_items = module_items.optimize(&mut HashSet::new());
    Module { name: module.name, port_decls, module_items }
}
