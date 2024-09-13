use std::collections::{HashMap, HashSet};

use crate::vir::*;

/// Cache that stores wire assignments.
/// For example, for the assignment `assign a = b`, (a, b) is added to cache.
#[derive(Debug, Default)]
struct WireCache {
    inner: HashMap<Expression, Expression>,
}

impl WireCache {
    /// Preprocess wire cache from given module items and port idents.
    fn preprocess(&mut self, module_items: &[ModuleItem], port_idents: &HashSet<Expression>) {
        let signedness = module_items
            .iter()
            .flat_map(|module_item| {
                if let ModuleItem::Declarations(decls) = module_item {
                    decls
                        .iter()
                        .filter(|decl| !matches!(decl, Declaration::Integer(_)))
                        .map(|decl| (decl.ident().to_string(), decl.shape().is_signed))
                        .collect()
                } else {
                    vec![]
                }
            })
            .collect::<HashMap<String, bool>>();

        for module_item in module_items {
            match module_item {
                ModuleItem::ContinuousAssigns(conts) => {
                    for cont in conts {
                        let ContinuousAssign(lvalue, expr) = cont;
                        if lvalue.is_identifier()
                            && expr.is_identifier()
                            && !port_idents.contains(lvalue)
                            && signedness.get(&lvalue.into_ident().unwrap())
                                == signedness.get(&expr.into_ident().unwrap())
                        {
                            self.merge(lvalue, expr);
                        }
                    }
                }
                ModuleItem::Commented(_, _, module_items) => self.preprocess(module_items, port_idents),
                _ => continue,
            }
        }
    }

    /// Returns the wire name that corresponds to the input. If the cache does not contain the
    /// name, return the input.
    fn get(&mut self, k: &Expression) -> Expression {
        let par = self.inner.get(k);

        match par {
            None => {
                self.inner.insert(k.clone(), k.clone());
                k.clone()
            }
            Some(par) => {
                let par = par.clone();
                if &par == k {
                    k.clone()
                } else {
                    let par = self.get(&par);
                    self.inner.insert(k.clone(), par.clone());
                    par
                }
            }
        }
    }

    /// Merges the variable name `k1` into `k2`.
    fn merge(&mut self, k1: &Expression, k2: &Expression) {
        let par1 = self.get(k1);
        let par2 = self.get(k2);

        if par1 != par2 {
            self.inner.insert(par1, par2);
        }
    }
}

trait OptimizeWireCache {
    /// Optimizes by using wire cache.
    fn optimize(&self, wire_cache: &mut WireCache) -> Self;
}

impl OptimizeWireCache for Vec<ModuleItem> {
    fn optimize(&self, wire_cache: &mut WireCache) -> Self {
        self.iter()
            .filter_map(|module_item| match module_item {
                ModuleItem::Declarations(decls) => {
                    let decls = decls
                        .iter()
                        .filter_map(|decl| match decl {
                            Declaration::Net(shape, ident) => {
                                let expr = Expression::ident(ident.clone());
                                if wire_cache.get(&expr) == expr {
                                    Some(Declaration::Net(shape.clone(), ident.clone()))
                                } else {
                                    None
                                }
                            }
                            Declaration::Reg(shape, ident, Some(init)) => {
                                Some(Declaration::Reg(shape.clone(), ident.clone(), Some(init.optimize(wire_cache))))
                            }
                            Declaration::Reg(shape, ident, None) => {
                                Some(Declaration::Reg(shape.clone(), ident.clone(), None))
                            }
                            Declaration::Integer(ident) => Some(Declaration::Integer(ident.clone())),
                        })
                        .collect::<Vec<_>>();

                    if decls.is_empty() {
                        None
                    } else {
                        Some(ModuleItem::Declarations(decls))
                    }
                }
                ModuleItem::ContinuousAssigns(conts) => {
                    let conts = conts.optimize(wire_cache);
                    if conts.is_empty() {
                        None
                    } else {
                        Some(ModuleItem::ContinuousAssigns(conts))
                    }
                }
                ModuleItem::ModuleInstantiation(module_inst) => {
                    Some(ModuleItem::ModuleInstantiation(module_inst.optimize(wire_cache)))
                }
                ModuleItem::AlwaysConstruct(event, stmts) => {
                    Some(ModuleItem::AlwaysConstruct(event.clone(), stmts.optimize(wire_cache)))
                }
                ModuleItem::Commented(comment_before, comment_after, items) => {
                    let items = items.optimize(wire_cache);
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

impl OptimizeWireCache for Vec<ContinuousAssign> {
    fn optimize(&self, wire_cache: &mut WireCache) -> Self {
        self.iter()
            .filter_map(|cont| {
                let ContinuousAssign(lvalue, expr) = cont;
                if wire_cache.get(lvalue) == lvalue.clone() {
                    Some(ContinuousAssign(lvalue.clone(), expr.optimize(wire_cache)))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl OptimizeWireCache for ModuleInstantiation {
    fn optimize(&self, wire_cache: &mut WireCache) -> Self {
        let Self { module_name, inst_name, params, port_connections } = self;

        Self {
            module_name: module_name.clone(),
            inst_name: inst_name.clone(),
            params: params.clone(),
            port_connections: port_connections
                .iter()
                .map(|(port_name, expr)| (port_name.clone(), expr.optimize(wire_cache)))
                .collect(),
        }
    }
}

impl OptimizeWireCache for Vec<Statement> {
    fn optimize(&self, wire_cache: &mut WireCache) -> Self {
        self.iter().map(|stmt| stmt.optimize(wire_cache)).collect()
    }
}

impl OptimizeWireCache for Statement {
    fn optimize(&self, wire_cache: &mut WireCache) -> Self {
        match self {
            Statement::BlockingAssignment(lvalue, expr, span) => {
                Statement::BlockingAssignment(lvalue.optimize(wire_cache), expr.optimize(wire_cache), *span)
            }
            Statement::Conditional(cond_expr_pairs, else_stmt, span) if else_stmt.is_empty() => {
                let cond_expr_pairs = cond_expr_pairs
                    .iter()
                    .map(|(cond, expr)| (cond.optimize(wire_cache), expr.optimize(wire_cache)))
                    .collect();
                Statement::Conditional(cond_expr_pairs, Vec::new(), *span)
            }
            Statement::Conditional(cond_expr_pairs, else_stmt, span) => {
                let cond_expr_pairs = cond_expr_pairs
                    .iter()
                    .map(|(cond, expr)| (cond.optimize(wire_cache), expr.optimize(wire_cache)))
                    .collect();
                Statement::Conditional(cond_expr_pairs, else_stmt.optimize(wire_cache), *span)
            }
            Statement::Loop(ident, count, stmt, span) => {
                Statement::Loop(ident.clone(), count.optimize(wire_cache), stmt.optimize(wire_cache), *span)
            }
            Statement::NonblockingAssignment(lvalue, expr, span) => {
                Statement::NonblockingAssignment(lvalue.optimize(wire_cache), expr.optimize(wire_cache), *span)
            }
            Statement::Case(case_expr, case_items, default, span) => Statement::Case(
                case_expr.optimize(wire_cache),
                case_items
                    .iter()
                    .map(|(cond, stmts)| (cond.optimize(wire_cache), stmts.optimize(wire_cache)))
                    .collect(),
                default.optimize(wire_cache),
                *span,
            ),
            Statement::Display(fstring, args, span) => {
                Statement::Display(fstring.clone(), args.iter().map(|arg| arg.optimize(wire_cache)).collect(), *span)
            }
            Statement::Fatal => Statement::Fatal,
        }
    }
}

impl OptimizeWireCache for Expression {
    fn optimize(&self, wire_cache: &mut WireCache) -> Self {
        match self {
            Expression::Primary(prim) => Expression::Primary(prim.optimize(wire_cache)),
            Expression::Unary(op, prim) => Expression::Unary(*op, prim.optimize(wire_cache)),
            Expression::Binary(lhs, op, rhs) => {
                Expression::Binary(Box::new(lhs.optimize(wire_cache)), *op, Box::new(rhs.optimize(wire_cache)))
            }
            Expression::Conditional(cond, then_expr, else_expr) => Expression::Conditional(
                Box::new(cond.optimize(wire_cache)),
                Box::new(then_expr.optimize(wire_cache)),
                Box::new(else_expr.optimize(wire_cache)),
            ),
        }
    }
}

impl OptimizeWireCache for Range {
    fn optimize(&self, wire_cache: &mut WireCache) -> Self {
        match self {
            Range::Index(index) => Range::Index(Box::new(index.optimize(wire_cache))),
            Range::Range(base, offset) => {
                Range::Range(Box::new(base.optimize(wire_cache)), Box::new(offset.optimize(wire_cache)))
            }
        }
    }
}

impl OptimizeWireCache for Primary {
    fn optimize(&self, wire_cache: &mut WireCache) -> Self {
        match self {
            Primary::Number(num) => Primary::Number(num.clone()),
            Primary::HierarchicalIdentifier(ident, Some(range)) => Primary::HierarchicalIdentifier(
                wire_cache.get(&Expression::ident(ident.clone())).to_string(),
                Some(range.optimize(wire_cache)),
            ),
            Primary::HierarchicalIdentifier(ident, None) => {
                Primary::HierarchicalIdentifier(wire_cache.get(&Expression::ident(ident.clone())).to_string(), None)
            }
            Primary::Concatenation(concat) => Primary::Concatenation(concat.optimize(wire_cache)),
            Primary::MultipleConcatenation(count, concat) => {
                Primary::MultipleConcatenation(*count, concat.optimize(wire_cache))
            }
            Primary::MintypmaxExpression(expr) => Primary::MintypmaxExpression(Box::new(expr.optimize(wire_cache))),
        }
    }
}

impl OptimizeWireCache for Concatenation {
    fn optimize(&self, wire_cache: &mut WireCache) -> Self {
        Self { exprs: self.exprs.iter().map(|expr| expr.optimize(wire_cache)).collect() }
    }
}

/// Optimizes module by using wire cache.
///
/// Wires in port declarations will not removed.
pub fn wire_cache_opt(module: Module) -> Module {
    let module_items = module.module_items;
    let port_decls = module.port_decls;

    let port_idents = port_decls
        .iter()
        .map(|port_decl| match port_decl {
            PortDeclaration::Input(_, ident) => Expression::ident(ident.clone()),
            PortDeclaration::Output(_, ident) => Expression::ident(ident.clone()),
        })
        .collect::<HashSet<Expression>>();

    let mut wire_cache = WireCache::default();
    wire_cache.preprocess(&module_items, &port_idents);

    let module_items = module_items.optimize(&mut wire_cache);
    Module { name: module.name, port_decls, module_items }
}
