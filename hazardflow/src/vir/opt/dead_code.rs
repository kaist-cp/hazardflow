use std::collections::HashSet;

use crate::vir::*;

/// Returns ident of lvalue.
fn get_lvalue_ident(lvalue: &Expression) -> String {
    if let Expression::Primary(Primary::HierarchicalIdentifier(ident, _)) = lvalue {
        ident.clone()
    } else {
        panic!("lvalue should be hierarchical identifier");
    }
}

/// Returns range expr of lvalue.
fn get_range(expr: &Expression) -> Option<Range> {
    if let Expression::Primary(Primary::HierarchicalIdentifier(_, Some(range))) = expr {
        Some(range.clone())
    } else {
        None
    }
}

/// TODO: Implement general walk trait?
trait OptimizeDeadcodeWalk {
    /// Get used variables.
    fn walk(&self, used: &mut HashSet<Expression>);
}

impl OptimizeDeadcodeWalk for Vec<ModuleItem> {
    fn walk(&self, used: &mut HashSet<Expression>) {
        for module_item in self {
            module_item.walk(used);
        }
    }
}

impl OptimizeDeadcodeWalk for ModuleItem {
    fn walk(&self, used: &mut HashSet<Expression>) {
        match self {
            ModuleItem::Declarations(decls) => {
                decls.iter().for_each(|decl| {
                    if let Declaration::Reg(_, _, Some(init)) = decl {
                        init.walk(used);
                    }
                });
            }
            ModuleItem::ContinuousAssigns(conts) => conts.walk(used),
            ModuleItem::ModuleInstantiation(module_inst) => module_inst.walk(used),
            ModuleItem::AlwaysConstruct(_, stmts) => stmts.walk(used),
            ModuleItem::Commented(_, _, items) => items.walk(used),
        }
    }
}

impl OptimizeDeadcodeWalk for Vec<ContinuousAssign> {
    fn walk(&self, used: &mut HashSet<Expression>) {
        for cont in self {
            let ContinuousAssign(_, expr) = cont;
            expr.walk(used);
        }
    }
}

impl OptimizeDeadcodeWalk for ModuleInstantiation {
    fn walk(&self, used: &mut HashSet<Expression>) {
        for (_, expr) in &self.port_connections {
            expr.walk(used);
        }
    }
}

impl OptimizeDeadcodeWalk for Vec<Statement> {
    fn walk(&self, used: &mut HashSet<Expression>) {
        for stmt in self {
            stmt.walk(used);
        }
    }
}

impl OptimizeDeadcodeWalk for Statement {
    fn walk(&self, used: &mut HashSet<Expression>) {
        match self {
            Statement::BlockingAssignment(lhs, expr, _) => {
                if let Some(range) = get_range(lhs) {
                    range.walk(used);
                }
                expr.walk(used);
            }
            Statement::Conditional(cond_expr_pairs, else_stmt, _) if else_stmt.is_empty() => {
                for (cond, stmt) in cond_expr_pairs {
                    cond.walk(used);
                    stmt.walk(used);
                }
            }
            Statement::Conditional(cond_expr_pairs, else_stmt, _) => {
                for (cond, stmt) in cond_expr_pairs {
                    cond.walk(used);
                    stmt.walk(used);
                }
                else_stmt.walk(used)
            }
            Statement::Loop(ident, count, stmt, _) => {
                used.insert(Expression::ident(ident.clone()));
                count.walk(used);
                stmt.walk(used);
            }
            Statement::NonblockingAssignment(lhs, expr, _) => {
                if let Some(range) = get_range(lhs) {
                    range.walk(used);
                }
                expr.walk(used)
            }
            Statement::Case(case_expr, case_items, default, _) => {
                case_expr.walk(used);
                for (cond, stmts) in case_items {
                    cond.walk(used);
                    stmts.walk(used);
                }
                default.walk(used);
            }
            Statement::Display(_, args, _) => {
                for arg in args {
                    arg.walk(used)
                }
            }
            Statement::Fatal => {}
        }
    }
}

impl OptimizeDeadcodeWalk for Expression {
    fn walk(&self, used: &mut HashSet<Expression>) {
        match self {
            Expression::Primary(prim) => prim.walk(used),
            Expression::Unary(_, prim) => prim.walk(used),
            Expression::Binary(lhs, _, rhs) => {
                lhs.walk(used);
                rhs.walk(used);
            }
            Expression::Conditional(cond, then_expr, else_expr) => {
                cond.walk(used);
                then_expr.walk(used);
                else_expr.walk(used);
            }
        }
    }
}

impl OptimizeDeadcodeWalk for Range {
    fn walk(&self, used: &mut HashSet<Expression>) {
        match self {
            Range::Index(index) => index.walk(used),
            Range::Range(base, offset) => {
                base.walk(used);
                offset.walk(used);
            }
        }
    }
}

impl OptimizeDeadcodeWalk for Primary {
    fn walk(&self, used: &mut HashSet<Expression>) {
        match self {
            Primary::Number(_) => {}
            Primary::HierarchicalIdentifier(ident, Some(range)) => {
                used.insert(Expression::ident(ident.clone()));
                range.walk(used);
            }
            Primary::HierarchicalIdentifier(ident, None) => {
                used.insert(Expression::ident(ident.clone()));
            }
            Primary::Concatenation(concat) => concat.walk(used),
            Primary::MultipleConcatenation(_, concat) => concat.walk(used),
            Primary::MintypmaxExpression(expr) => expr.walk(used),
        }
    }
}

impl OptimizeDeadcodeWalk for Concatenation {
    fn walk(&self, used: &mut HashSet<Expression>) {
        for expr in &self.exprs {
            expr.walk(used);
        }
    }
}

trait OptimizeDeadcode {
    /// Optimizes by using dead code elimination.
    fn optimize(&self, used: &HashSet<Expression>) -> Self;
}

impl OptimizeDeadcode for Vec<ModuleItem> {
    fn optimize(&self, used: &HashSet<Expression>) -> Self {
        self.iter()
            .filter_map(|module_item| match module_item {
                ModuleItem::Declarations(decls) => {
                    let decls = decls
                        .iter()
                        .filter_map(|decl| match decl {
                            Declaration::Net(shape, ident) => {
                                if used.get(&Expression::ident(ident.clone())).is_some() {
                                    Some(Declaration::Net(shape.clone(), ident.clone()))
                                } else {
                                    None
                                }
                            }
                            Declaration::Reg(shape, ident, init) => {
                                if used.get(&Expression::ident(ident.clone())).is_some() {
                                    Some(Declaration::Reg(shape.clone(), ident.clone(), init.clone()))
                                } else {
                                    None
                                }
                            }
                            Declaration::Integer(ident) => {
                                if used.get(&Expression::ident(ident.clone())).is_some() {
                                    Some(Declaration::Integer(ident.clone()))
                                } else {
                                    None
                                }
                            }
                        })
                        .collect::<Vec<_>>();

                    if decls.is_empty() {
                        None
                    } else {
                        Some(ModuleItem::Declarations(decls))
                    }
                }
                ModuleItem::ContinuousAssigns(conts) => {
                    let conts = conts.optimize(used);
                    if conts.is_empty() {
                        None
                    } else {
                        Some(ModuleItem::ContinuousAssigns(conts))
                    }
                }
                ModuleItem::ModuleInstantiation(module_inst) => {
                    Some(ModuleItem::ModuleInstantiation(module_inst.clone()))
                }
                ModuleItem::AlwaysConstruct(event, stmts) => {
                    Some(ModuleItem::AlwaysConstruct(event.clone(), stmts.optimize(used)))
                }
                ModuleItem::Commented(comment_before, comment_after, items) => {
                    let items = items.optimize(used);
                    if items.is_empty() {
                        None
                    } else {
                        Some(ModuleItem::Commented(comment_before.clone(), comment_after.clone(), items))
                    }
                }
            })
            .collect()
    }
}

impl OptimizeDeadcode for Vec<ContinuousAssign> {
    fn optimize(&self, used: &HashSet<Expression>) -> Self {
        self.iter()
            .filter_map(|cont| {
                let ContinuousAssign(lvalue, expr) = cont;
                if used.get(&Expression::ident(get_lvalue_ident(lvalue))).is_some() {
                    Some(ContinuousAssign(lvalue.clone(), expr.clone()))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl OptimizeDeadcode for Vec<Statement> {
    fn optimize(&self, used: &HashSet<Expression>) -> Self {
        // XXX: Think about span behavior
        self.iter()
            .filter_map(|stmt| match stmt {
                Statement::BlockingAssignment(lvalue, expr, span) => {
                    if used.get(&Expression::ident(get_lvalue_ident(lvalue))).is_some() {
                        Some(Statement::BlockingAssignment(lvalue.clone(), expr.clone(), *span))
                    } else {
                        None
                    }
                }
                Statement::Conditional(cond_expr_pairs, else_stmt, span) if else_stmt.is_empty() => {
                    let cond_expr_pairs = cond_expr_pairs
                        .iter()
                        .map(|(cond, expr)| (cond.clone(), expr.optimize(used)))
                        .collect::<Vec<_>>();
                    Some(Statement::Conditional(cond_expr_pairs, Vec::new(), *span))
                }
                Statement::Conditional(cond_expr_pairs, else_stmt, span) => {
                    let cond_expr_pairs = cond_expr_pairs
                        .iter()
                        .map(|(cond, expr)| (cond.clone(), expr.optimize(used)))
                        .collect::<Vec<_>>();

                    Some(Statement::Conditional(cond_expr_pairs, else_stmt.optimize(used), *span))
                }
                Statement::Loop(ident, count, stmt, span) => {
                    Some(Statement::Loop(ident.clone(), count.clone(), stmt.optimize(used), *span))
                }
                Statement::NonblockingAssignment(lvalue, expr, span) => {
                    if used.get(&Expression::ident(get_lvalue_ident(lvalue))).is_some() {
                        Some(Statement::NonblockingAssignment(lvalue.clone(), expr.clone(), *span))
                    } else {
                        None
                    }
                }
                Statement::Case(case_expr, case_items, default, span) => Some(Statement::Case(
                    case_expr.clone(),
                    case_items.iter().map(|(cond, stmts)| (cond.clone(), stmts.optimize(used))).collect(),
                    default.optimize(used),
                    *span,
                )),
                Statement::Display(fstring, args, span) => {
                    Some(Statement::Display(fstring.clone(), args.clone(), *span))
                }
                Statement::Fatal => Some(Statement::Fatal),
            })
            .collect()
    }
}

/// Optimizes module by using dead code elimination.
pub fn dead_code_opt(module: Module) -> Module {
    let module_items = module.module_items;
    let port_decls = module.port_decls;

    let mut relaxation = true;
    let mut module_items = module_items;

    while relaxation {
        let mut used = HashSet::new();

        for port_decl in port_decls.clone() {
            let ident = match port_decl {
                PortDeclaration::Input(_, ident) => Expression::ident(ident),
                PortDeclaration::Output(_, ident) => Expression::ident(ident),
            };
            used.insert(ident);
        }
        module_items.walk(&mut used);

        let new_module_items = module_items.optimize(&used);
        relaxation = module_items != new_module_items;
        module_items = new_module_items;
    }

    Module { name: module.name, port_decls, module_items }
}
