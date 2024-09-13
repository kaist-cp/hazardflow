//! Detect Combinational Loop in the module.
//!
//! This analysis is only applicable to the module that has been flattened, which can be done by `flatten` pass.

use std::collections::{HashMap, HashSet};

use itertools::{iproduct, Itertools};

use crate::compiler::error::VirgenError;
use crate::vir::*;

/// Detect combinational loop in the module.
pub fn detect_comb_loop(module: &Module) -> Result<(), VirgenError> {
    let mut decls = module.port_decls.iter().map(|p| p.name()).collect_vec();

    for item in module.module_items.iter() {
        decls.append(&mut item.get_decls());
    }

    let decl_to_id = decls.iter().enumerate().map(|(i, d)| (d.clone(), Id(i))).collect::<HashMap<_, _>>();

    let decl_to_id_reversed = decls.iter().enumerate().map(|(i, d)| (Id(i), d.clone())).collect::<HashMap<_, _>>();

    let mut d = DetectCombLoop { decl_to_id, decl_to_id_reversed, dep_graph: HashMap::new(), cond_ctx: HashSet::new() };

    d.construct_dep_graph(module)?;

    d.check_loop()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Id(usize);

#[derive(Debug)]
struct DetectCombLoop {
    decl_to_id: HashMap<String, Id>,
    decl_to_id_reversed: HashMap<Id, String>,
    dep_graph: HashMap<Id, HashSet<Id>>,

    cond_ctx: HashSet<Id>,
}

impl DetectCombLoop {
    // Check if there is a loop in the dependency graph.
    #[allow(clippy::collapsible_if)]
    fn check_loop(&self) -> Result<(), VirgenError> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        for &node in self.dep_graph.keys() {
            if !visited.contains(&node) {
                if self.dfs(node, &mut visited, &mut stack) {
                    // Found loop.
                    return Err(VirgenError::AnalysisError {
                        msg: format!(
                            "Combinational loop detected. Stack {}",
                            stack.into_iter().map(|id| self.get_decl_by_id(id)).collect::<Vec<_>>().join(" -> ")
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    fn dfs(&self, node: Id, visited: &mut HashSet<Id>, stack: &mut Vec<Id>) -> bool {
        if visited.contains(&node) {
            return false;
        }

        visited.insert(node);
        stack.push(node);

        if let Some(neighbors) = self.dep_graph.get(&node) {
            for &neighbor in neighbors {
                if stack.contains(&neighbor) {
                    return true;
                }
                if self.dfs(neighbor, visited, stack) {
                    return true;
                }
            }
        }

        stack.pop();
        false
    }
}

impl DetectCombLoop {
    fn construct_dep_graph(&mut self, module: &Module) -> Result<(), VirgenError> {
        for item in module.module_items.iter() {
            self.constuct_graph_module_item(item)?;
        }

        Ok(())
    }

    fn constuct_graph_module_item(&mut self, item: &ModuleItem) -> Result<(), VirgenError> {
        match item {
            ModuleItem::Declarations(_) => {}
            ModuleItem::ContinuousAssigns(conts) => {
                for cont in conts {
                    let ContinuousAssign(lhs, rhs) = cont;

                    self.add_assignment_edge(lhs, rhs)?;
                }
            }
            ModuleItem::ModuleInstantiation(_) => {}
            ModuleItem::AlwaysConstruct(name, stmts) => {
                if name == "always @*" {
                    for stmt in stmts.iter() {
                        self.construct_graph_stmt(stmt)?;
                    }
                }
            }
            ModuleItem::Commented(_, _, items) => {
                for item in items.iter() {
                    self.constuct_graph_module_item(item)?
                }
            }
        }

        Ok(())
    }

    fn construct_graph_stmt(&mut self, stmt: &Statement) -> Result<(), VirgenError> {
        match stmt {
            Statement::NonblockingAssignment(lhs, rhs, _) | Statement::BlockingAssignment(lhs, rhs, _) => {
                self.add_assignment_edge(lhs, rhs)
            }
            Statement::Conditional(then_branches, else_branch, _) => {
                assert!(self.cond_ctx.is_empty());
                for (cond, stmts) in then_branches.iter() {
                    let cond_nodes = cond.get_nodes(&self.decl_to_id);
                    self.add_cond_nodes(cond_nodes);

                    for stmt in stmts.iter() {
                        self.construct_graph_stmt(stmt)?;
                    }
                }

                for stmt in else_branch.iter() {
                    self.construct_graph_stmt(stmt)?;
                }

                self.clear_cond_nodes();

                Ok(())
            }
            Statement::Loop(_, _, stmts, _) => {
                // XXX: We are not handling the loop condition because there is no unbounded loop in synthesizable verilog.
                for stmt in stmts.iter() {
                    self.construct_graph_stmt(stmt)?
                }

                Ok(())
            }
            Statement::Case(expr, cases, default, _) => {
                assert!(self.cond_ctx.is_empty());
                self.add_cond_nodes(expr.get_nodes(&self.decl_to_id));

                for (_, stmts) in cases.iter() {
                    for stmt in stmts.iter() {
                        self.construct_graph_stmt(stmt)?;
                    }
                }

                for stmt in default.iter() {
                    self.construct_graph_stmt(stmt)?;
                }

                self.clear_cond_nodes();

                Ok(())
            }
            Statement::Display(..) => {
                panic!("Disable --display option to run comb loop analysis")
            }
            Statement::Fatal => Ok(()),
        }
    }

    fn add_assignment_edge(&mut self, lhs: &Expression, rhs: &Expression) -> Result<(), VirgenError> {
        let lhs = lhs.get_nodes(&self.decl_to_id);
        assert_eq!(lhs.len(), 1);

        let rhs = rhs.get_nodes(&self.decl_to_id);

        for (l, r) in iproduct!(lhs.iter(), rhs) {
            if *l == r {
                return Err(VirgenError::AnalysisError {
                    msg: format!("Self loop detected: {:?}", self.get_decl_by_id(r)),
                });
            }

            self.dep_graph.entry(*l).or_default().insert(r);
        }

        for (l, cond) in iproduct!(lhs, self.cond_ctx.iter()) {
            if l == *cond {
                return Err(VirgenError::AnalysisError {
                    msg: format!("Self loop detected: {:?}", self.get_decl_by_id(l)),
                });
            }

            self.dep_graph.entry(l).or_default().insert(*cond);
        }

        Ok(())
    }

    fn add_cond_nodes(&mut self, nodes: Vec<Id>) {
        self.cond_ctx.extend(nodes);
    }

    fn clear_cond_nodes(&mut self) {
        self.cond_ctx.clear();
    }

    fn get_decl_by_id(&self, id: Id) -> &str {
        self.decl_to_id_reversed.get(&id).unwrap()
    }
}

impl ModuleItem {
    fn get_decls(&self) -> Vec<String> {
        match self {
            ModuleItem::Declarations(decls) => decls.iter().map(|d| d.name()).collect(),
            ModuleItem::Commented(_, _, items) => items.iter().flat_map(|item| item.get_decls()).collect(),
            ModuleItem::ContinuousAssigns(_) | ModuleItem::ModuleInstantiation(_) | ModuleItem::AlwaysConstruct(..) => {
                vec![]
            }
        }
    }
}

impl Expression {
    fn get_nodes(&self, decl_to_id: &HashMap<String, Id>) -> Vec<Id> {
        match self {
            Expression::Primary(primary) | Expression::Unary(_, primary) => primary.get_nodes(decl_to_id),
            Expression::Binary(lhs, _, rhs) => [lhs.get_nodes(decl_to_id), rhs.get_nodes(decl_to_id)].concat(),
            Expression::Conditional(cond, then, els) => {
                [cond.get_nodes(decl_to_id), then.get_nodes(decl_to_id), els.get_nodes(decl_to_id)].concat()
            }
        }
    }
}

impl Primary {
    fn get_nodes(&self, decl_to_id: &HashMap<String, Id>) -> Vec<Id> {
        match self {
            Primary::Number(_) => vec![],
            Primary::HierarchicalIdentifier(name, _) => {
                vec![*decl_to_id.get(name).unwrap()]
            }
            Primary::Concatenation(concat) | Primary::MultipleConcatenation(_, concat) => concat.get_nodes(decl_to_id),
            Primary::MintypmaxExpression(expr) => expr.get_nodes(decl_to_id),
        }
    }
}

impl Concatenation {
    fn get_nodes(&self, decl_to_id: &HashMap<String, Id>) -> Vec<Id> {
        self.exprs.iter().flat_map(|p| p.get_nodes(decl_to_id)).collect()
    }
}
