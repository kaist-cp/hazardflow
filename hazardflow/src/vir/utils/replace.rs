//! Replaces.

use std::collections::HashMap;

use super::*;

fn replaced(replaces: &HashMap<String, String>, key: &String) -> String {
    replaces.get(key).unwrap_or(key).clone()
}

pub(crate) trait Replace {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self;
}

impl<T: Replace> Replace for Vec<T> {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self {
        self.iter().map(|e| e.replace(replaces)).collect()
    }
}

impl<T: Replace + Clone> Replace for Option<T> {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self {
        self.as_ref().map(|e| e.replace(replaces))
    }
}

impl Replace for Module {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self {
        Module {
            name: self.name.clone(),
            port_decls: self.port_decls.replace(replaces),
            module_items: self.module_items.replace(replaces),
        }
    }
}

impl Replace for PortDeclaration {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self {
        match self {
            PortDeclaration::Input(width, ident) => PortDeclaration::input(*width, replaced(replaces, ident)),
            PortDeclaration::Output(width, ident) => PortDeclaration::output(*width, replaced(replaces, ident)),
        }
    }
}

impl Replace for ModuleItem {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self {
        match self {
            ModuleItem::Declarations(decls) => {
                ModuleItem::Declarations(decls.iter().map(|decl| decl.replace(replaces)).collect())
            }
            ModuleItem::ContinuousAssigns(conts) => {
                ModuleItem::ContinuousAssigns(conts.iter().map(|cont| cont.replace(replaces)).collect())
            }
            ModuleItem::ModuleInstantiation(module_inst) => {
                ModuleItem::ModuleInstantiation(module_inst.replace(replaces))
            }
            ModuleItem::AlwaysConstruct(event, stmts) => {
                ModuleItem::AlwaysConstruct(event.clone(), stmts.replace(replaces))
            }
            ModuleItem::Commented(comment_before, comment_after, items) => {
                ModuleItem::Commented(comment_before.clone(), comment_after.clone(), items.replace(replaces))
            }
        }
    }
}

impl Replace for Declaration {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self {
        match self {
            Declaration::Net(shape, ident) => Declaration::net(shape.clone(), replaced(replaces, ident)),
            Declaration::Reg(shape, ident, init) => Declaration::Reg(
                shape.clone(),
                replaced(replaces, ident),
                init.clone().map(|expr| expr.replace(replaces)),
            ),
            Declaration::Integer(ident) => Declaration::integer(replaced(replaces, ident)),
        }
    }
}

impl Replace for Expression {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self {
        match self {
            Expression::Primary(prim) => Expression::Primary(prim.replace(replaces)),
            Expression::Unary(op, prim) => Expression::Unary(*op, prim.replace(replaces)),
            Expression::Binary(lhs, op, rhs) => Expression::binary(*op, lhs.replace(replaces), rhs.replace(replaces)),
            Expression::Conditional(cond, then_expr, else_expr) => Expression::conditional(
                cond.replace(replaces),
                then_expr.replace(replaces),
                else_expr.replace(replaces),
            ),
        }
    }
}

impl Replace for Range {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self {
        match self {
            Range::Index(index) => Range::new_index(index.replace(replaces)),
            Range::Range(base, offset) => Range::new_range(base.replace(replaces), offset.replace(replaces)),
        }
    }
}

impl Replace for Primary {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self {
        match self {
            Primary::Number(_) => self.clone(),
            Primary::HierarchicalIdentifier(ident, range) => Primary::HierarchicalIdentifier(
                replaced(replaces, ident),
                range.clone().map(|range| range.replace(replaces)),
            ),
            Primary::Concatenation(concat) => Primary::Concatenation(concat.replace(replaces)),
            Primary::MultipleConcatenation(count, concat) => {
                Primary::MultipleConcatenation(*count, concat.replace(replaces))
            }
            Primary::MintypmaxExpression(expr) => Primary::MintypmaxExpression(Box::new(expr.replace(replaces))),
        }
    }
}

impl Replace for Concatenation {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self {
        Concatenation { exprs: self.exprs.iter().map(|expr| expr.replace(replaces)).collect() }
    }
}

impl Replace for ContinuousAssign {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self {
        ContinuousAssign(self.0.replace(replaces), self.1.replace(replaces))
    }
}

impl Replace for ModuleInstantiation {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self {
        ModuleInstantiation {
            port_connections: self
                .port_connections
                .iter()
                .map(|(port_name, expr)| (port_name.clone(), expr.replace(replaces)))
                .collect(),
            ..self.clone()
        }
    }
}

impl Replace for Statement {
    fn replace(&self, replaces: &HashMap<String, String>) -> Self {
        match self {
            Statement::BlockingAssignment(lvalue, expr, span) => {
                Statement::BlockingAssignment(lvalue.replace(replaces), expr.replace(replaces), *span)
            }
            Statement::Conditional(cond_expr_pairs, else_stmt, span) => Statement::Conditional(
                cond_expr_pairs.iter().map(|(expr, stmts)| (expr.replace(replaces), stmts.replace(replaces))).collect(),
                else_stmt.replace(replaces),
                *span,
            ),
            Statement::Loop(ident, count, stmt, span) => {
                Statement::Loop(replaced(replaces, ident), count.replace(replaces), stmt.replace(replaces), *span)
            }
            Statement::NonblockingAssignment(lvalue, expr, span) => {
                Statement::NonblockingAssignment(lvalue.replace(replaces), expr.replace(replaces), *span)
            }
            Statement::Case(case_expr, case_items, default, span) => Statement::Case(
                case_expr.replace(replaces),
                case_items.iter().map(|(expr, stmts)| (expr.replace(replaces), stmts.replace(replaces))).collect(),
                default.replace(replaces),
                *span,
            ),
            Statement::Display(fstring, args, span) => {
                Statement::Display(fstring.clone(), args.replace(replaces), *span)
            }
            Statement::Fatal => Statement::Fatal,
        }
    }
}
