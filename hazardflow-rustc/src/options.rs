//! Options

use clap::Parser;
use env;
use hazardflow::*;

/// Hazardflow Compiler Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct HazardflowArgs {
    /// Compiles debug information such as `display!` or `assert!` in generated Verilog
    #[arg(short, long, default_value = "false")]
    pub(crate) system_task: bool,

    /// Performs wire-cache optimiation
    #[clap(long = "wire-cache")]
    pub(crate) wire_cache: bool,

    /// Performs deadcode elimination
    #[clap(long = "deadcode")]
    pub(crate) deadcode: bool,

    /// Performs always-block inlining
    #[clap(long = "inline-always")]
    pub(crate) inline_always: bool,

    /// Integrates into a top module
    #[clap(long = "integrate")]
    pub(crate) integrate: bool,

    /// Integrates into a top module
    #[clap(long = "detect-comb-loop")]
    pub(crate) detect_comb_loop: bool,

    /// Compiler Targets
    #[clap(long = "target", num_args = 0..)]
    pub(crate) target: Vec<String>,

    /// Merge all modules into a single file
    #[clap(long = "merge")]
    pub(crate) merge: bool,
}

impl HazardflowArgs {
    // TODO: Allow users to specify build directory
    pub fn into_opts(self) -> Options {
        let working_dir = env::current_dir().expect("Unable to gen current directory");
        let mut build_dir = working_dir;
        build_dir.push("build");
        std::fs::create_dir_all(&build_dir).expect("build dir creation failed");

        Options {
            build_dir,
            system_task: self.system_task,
            wire_cache: self.wire_cache,
            deadcode: self.deadcode,
            inline_always: self.inline_always,
            integrate: self.integrate,
            detect_comb_loop: self.detect_comb_loop,
            target: if self.target.is_empty() { CompileTarget::All } else { CompileTarget::FilterBy(self.target) },
            merge: self.merge,
        }
    }
}

#[derive(Parser)]
pub struct Args {
    #[clap(flatten)]
    pub hazardflow: HazardflowArgs,
    #[clap(last = true)]
    pub rust_flags: Vec<String>,
}
