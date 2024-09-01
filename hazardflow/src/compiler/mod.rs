//! Compiler

use std::rc::Rc;

use rustc_interface::Queries;

pub mod build_submodule_graph;
pub mod codegen;
pub mod error;
pub mod module;
pub mod package;
pub mod prelude;
pub mod pure;
pub mod virgen;

use build_submodule_graph::*;
use codegen::*;
use error::*;
use module::*;
use package::*;
pub use prelude::*;
use pure::*;
use virgen::*;

use crate::utils::{copy_thir_before_steal, thir_body};

/// Hazardflow Compiler Options
#[derive(Debug, Clone)]
pub struct Options {
    /// Output Directory
    pub build_dir: std::path::PathBuf,

    /// Compiles system task such as `$fdisplay` or `assert` in generated Verilog
    pub system_task: bool,

    /// Performs wire-cache optimiation
    pub wire_cache: bool,

    /// Performs deadcode elimination
    pub deadcode: bool,

    /// Performs always-block inlining
    pub inline_always: bool,

    /// Integrates into a top module
    pub integrate: bool,

    /// Integrates into a top module
    pub detect_comb_loop: bool,

    /// Compiler Targets
    pub target: CompileTarget,

    /// Merge all modules into a single file
    pub merge: bool,
}

/// Compile Target Specifier
#[derive(Debug, Clone)]
pub enum CompileTarget {
    /// Compile all synthesizable modules
    All,

    /// Compile modules that matches the given patterns
    FilterBy(Vec<String>),
}

impl CompileTarget {
    /// Checks if the given path data should be allowed.
    pub fn should_compile(&self, path_str: &str) -> bool {
        match self {
            CompileTarget::All => true,
            CompileTarget::FilterBy(patterns) => patterns.iter().any(|pattern| path_str.contains(pattern)),
        }
    }
}

/// Hazardflow Compiler
#[derive(Debug)]
pub struct Compiler {
    /// Compiler Options
    options: Options,
}

impl Compiler {
    /// Build new compiler callback.
    pub fn new(options: Options) -> Self {
        Self { options }
    }
}

impl rustc_driver::Callbacks for Compiler {
    fn config(&mut self, config: &mut rustc_interface::Config) {
        assert!(config.override_queries.is_none());
        config.override_queries = Some(|_session, providers| {
            providers.mir_built = |tcx, def| {
                let thir = thir_body(tcx, def).borrow().clone();
                copy_thir_before_steal(def, thir.clone());
                (rustc_interface::DEFAULT_QUERY_PROVIDERS.mir_built)(tcx, def) as _
            };
        });
    }

    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        queries.global_ctxt().unwrap().enter(|tcx| {
            let package = match Package::new(tcx, Rc::new(self.options.clone())) {
                Ok(p) => p,
                Err(e) => panic!("{:#?}", e),
            };
            match package.build() {
                Ok(()) => {}
                Err(e) => log::info!("{:#?}", e),
            }
        });
        rustc_driver::Compilation::Continue
    }

    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        _queries: &'tcx Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        rustc_driver::Compilation::Stop
    }
}
