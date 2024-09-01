//! Package management for the Virgen build system.

use std::collections::HashMap;
use std::fs;
use std::io::Write;

use hir::def_id::DefId;
use rustc_hir::{self as hir, ItemId};
use rustc_middle::ty::TyCtxt;

use super::*;
use crate::*;

/// Traits that are reserved for the compiler
#[derive(Debug, Clone)]
pub enum LangTrait {
    /// `Interface` trait. (`hazardflow-designs/std/interface.rs`)
    Interface(DefId),

    /// `Default` trait.
    Default(DefId),

    /// `From` trait.
    From(DefId),

    /// `Into` trait.
    Into(DefId),
}

impl LangTrait {
    #[allow(unused)]
    fn def_id(&self) -> DefId {
        match self {
            LangTrait::Interface(id) | LangTrait::Default(id) | LangTrait::From(id) | LangTrait::Into(id) => *id,
        }
    }
}

/// The meta global information that is needed for compiling any modules in the crate
#[derive(Debug, Clone)]
pub(crate) struct Meta {
    lang_traits: Vec<LangTrait>,
}

impl Meta {
    /// Returns the `DefId` of the `Interface` trait
    pub(crate) fn interface_did(&self) -> DefId {
        self.lang_traits
            .iter()
            .find_map(|lang_trait| if let LangTrait::Interface(def_id) = lang_trait { Some(*def_id) } else { None })
            .expect("Interface trait must exist")
    }

    #[allow(unused)]
    pub(crate) fn find_lang_trait(&self, def_id: DefId) -> Option<LangTrait> {
        self.lang_traits.iter().find(|lang_trait| lang_trait.def_id() == def_id).cloned()
    }
}

/// The package manager for the Virgen build system
pub(crate) struct Package<'tcx> {
    /// The TyCtxt of the crate, which is needed to interact with the Rust compiler
    tcx: TyCtxt<'tcx>,

    /// The meta global information that is needed for compiling any modules in the crate
    meta: Rc<Meta>,

    /// The options for the compiler
    options: Rc<Options>,
}

impl<'tcx> Package<'tcx> {
    /// Creates a new `Package` instance.
    pub(crate) fn new(tcx: TyCtxt<'tcx>, options: Rc<Options>) -> VirgenResult<Self> {
        let lang_traits = ["Interface", "Default", "From", "Into"]
            .into_iter()
            .map(|name| {
                let def_id = find_trait_by_name(tcx, name).unwrap_or_else(|| panic!("{name} trait not found"));
                match name {
                    "Interface" => LangTrait::Interface(def_id),
                    "Default" => LangTrait::Default(def_id),
                    "From" => LangTrait::From(def_id),
                    "Into" => LangTrait::Into(def_id),
                    _ => unreachable!(),
                }
            })
            .collect();

        let meta = Meta { lang_traits }.into();

        Ok(Self { tcx, meta, options })
    }

    /// Returns whether the hir item is synthesizable or not.
    ///
    /// It checks (1) it has `#[synthesize]` attribute and (2) its path contains `--target` argument or not.
    fn is_synthesizable(&self, id: ItemId) -> bool {
        let hir_id = id.hir_id();

        // Returns `false` if it does not have `#[synthesize]` attribute.
        if get_hazardflow_attribute(self.tcx, hir_id) != Some(HazardFlowAttr::Synthesize) {
            return false;
        }

        let def_id = id.owner_id.def_id.to_def_id();
        let def_path = self.tcx.def_path(def_id);

        // Returns whether its path contains `--target` argument or not.
        def_path.data.iter().any(|path_data| match path_data.data.name() {
            rustc_hir::definitions::DefPathDataName::Named(sym) => self.options.target.should_compile(sym.as_str()),
            rustc_hir::definitions::DefPathDataName::Anon { .. } => false,
        })
    }

    /// Returns `Virgen` instances of all top-level modules.
    ///
    /// It iterates hir items and collects it if (1) it is a function, (2) it has `#[synthesize]` attribute, and (3) its path contains `--target` argument.
    fn collect_top_level_synthesizables(&self) -> Vec<Virgen<'tcx>> {
        let hir = self.tcx.hir();

        hir.items()
            .filter_map(|id: ItemId| {
                let item = hir.item(id);

                if matches!(item.kind, rustc_hir::ItemKind::Fn(..)) && self.is_synthesizable(id) {
                    Some(Virgen::top(self.tcx, self.meta.clone(), self.options.clone(), id.owner_id.def_id))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Builds the package.
    ///
    /// It runs in a 3 stage process:
    ///
    /// 1. Find all top-level synthesizable modules in the crate
    /// 2. Preprocess all modules(and submodules) in the crate, while collecting all submodules.
    ///    After this stage, all the modules in the crate are found and they should be ready to be compiled.
    /// 3. Compile all modules in the crate
    pub(crate) fn build(&self) -> VirgenResult<()> {
        let top_modules = self.collect_top_level_synthesizables();

        for top_module in top_modules {
            self.build_top_module(top_module)?;
        }

        Ok(())
    }

    fn build_top_module(&self, top_module: Virgen<'tcx>) -> Result<(), VirgenError> {
        let (top_name, top_module_name, mut vir_modules) = self.virgen_modules(top_module)?;

        if self.options.integrate {
            let top = vir::integrate(vir_modules, top_name.clone());
            vir_modules = HashMap::new();
            vir_modules.insert(top_name.clone(), top);
        }

        let dirpath = self.options.build_dir.join(top_module_name);
        // Creates a directory for module.
        if !dirpath.exists() {
            fs::create_dir(&dirpath).map_err(|err| VirgenError::Fs { err })?;
        } else if dirpath.is_file() {
            fs::remove_file(&dirpath).map_err(|err| VirgenError::Fs { err })?;
            fs::create_dir(&dirpath).map_err(|err| VirgenError::Fs { err })?;
        }

        let mut merged_file = if self.options.merge {
            let mut file =
                fs::File::create(dirpath.join(format!("{}.v", top_name))).map_err(|err| VirgenError::Fs { err })?;
            writeln!(file, "`timescale 1ns / 1ps\n\n").map_err(|err| VirgenError::Fs { err })?;

            Some(file)
        } else {
            None
        };

        for (name, vir_module) in vir_modules {
            let vir_module = self.optimize(vir_module);

            self.analyze(&vir_module)?;

            if let Some(merged_file) = &mut merged_file {
                self.dump_verilog(merged_file, vir_module)?;
            } else {
                let mut file =
                    fs::File::create(dirpath.join(format!("{}.v", name))).map_err(|err| VirgenError::Fs { err })?;
                writeln!(file, "`timescale 1ns / 1ps\n\n").map_err(|err| VirgenError::Fs { err })?;
                self.dump_verilog(&mut file, vir_module)?;
            }
        }

        Ok(())
    }

    fn virgen_modules(
        &self,
        top_module: Virgen<'tcx>,
    ) -> Result<(String, String, HashMap<String, vir::Module>), VirgenError> {
        let top_name = top_module.name();
        let top_module_name = top_module.top_module_name();
        let mut modules = vec![top_module];
        let mut vir_modules = HashMap::new();

        while let Some(mut module) = modules.pop() {
            let submodules = module.preprocess()?;
            for submodule in submodules {
                // TODO: check if there is circular submodule instantiation later
                if let Some(m) = submodule.module_inst() {
                    modules.push(Virgen::submodule(self.tcx, self.meta.clone(), self.options.clone(), m))
                }
            }

            log::info!("Start virgen {}", module.name());
            match module.virgen() {
                Ok(vir_module) => {
                    log::info!("Synthesized {}/{}.v", self.options.build_dir.to_string_lossy(), module.name());
                    vir_modules.insert(module.name(), vir_module);
                }
                Err(e) => {
                    log::error!("Failed to synthesize {}\n{}", module.name(), e);
                }
            };
        }

        Ok((top_name, top_module_name, vir_modules))
    }

    // Dumps Verilog code.
    fn dump_verilog(&self, file: &mut std::fs::File, vir_module: vir::Module) -> Result<(), VirgenError> {
        writeln!(file, "{}", vir_module.to_string()).map_err(|err| VirgenError::Fs { err })?;

        Ok(())
    }

    fn optimize(&self, vir_module: vir::Module) -> vir::Module {
        let mut opts: Vec<fn(vir::Module) -> vir::Module> = vec![];

        if self.options.inline_always {
            opts.push(vir::opt::inline_always)
        };

        if self.options.wire_cache {
            opts.push(vir::opt::wire_cache_opt)
        };

        if self.options.deadcode {
            opts.push(vir::opt::dead_code_opt)
        };

        opts.into_iter().fold(vir_module, |module, opt| opt(module))
    }

    #[allow(clippy::type_complexity)]
    fn analyze(&self, vir_module: &vir::Module) -> Result<(), VirgenError> {
        let mut analysis: Vec<(&str, fn(&vir::Module) -> Result<(), VirgenError>)> = vec![];

        if self.options.detect_comb_loop {
            assert!(self.options.integrate);

            analysis.push(("detect_comb_loop", vir::analysis::detect_comb_loop))
        }

        for (name, a) in analysis {
            // check time for each analysis
            let start = std::time::Instant::now();

            let analysis_result = a(vir_module);

            log::error!("{name} took: {:?}", start.elapsed());

            analysis_result?;
        }

        Ok(())
    }
}
