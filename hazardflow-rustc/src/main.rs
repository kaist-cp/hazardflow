#![feature(rustc_private)]
extern crate clap;
extern crate hazardflow;
extern crate lazy_static;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_interface;
extern crate rustc_session;

mod options;
use std::io::Write;
use std::panic::PanicInfo;
use std::process::Command;
use std::{env, panic};

use clap::*;
use options::{Args, HazardflowArgs};
use rustc_driver::{RunCompiler, DEFAULT_LOCALE_RESOURCES};
use rustc_errors::emitter::EmitterWriter;
use rustc_interface::interface::try_print_query_stack;
use rustc_session::config::ErrorOutputType;
use rustc_session::EarlyDiagCtxt;

const BUG_REPORT_URL: &str = "https://github.com/kaist-cp/hazardflow";

lazy_static::lazy_static! {
    static ref ICE_HOOK: Box<dyn Fn(&panic::PanicInfo<'_>) + Sync + Send + 'static> = {
        let hook = panic::take_hook();
        panic::set_hook(Box::new(report_panic));
        hook
    };
}

fn report_panic(info: &PanicInfo) {
    (*ICE_HOOK)(info);

    // Separate the output with an empty line
    eprintln!();
    let fallback_bundle = rustc_errors::fallback_fluent_bundle(DEFAULT_LOCALE_RESOURCES.to_vec(), false);

    let emitter = Box::new(EmitterWriter::stderr(rustc_errors::ColorConfig::Auto, fallback_bundle));
    let diag_ctxt = rustc_errors::DiagCtxt::with_emitter(emitter);

    let mut diagnostic = diag_ctxt.struct_note("Hazardflow has paniced!");
    diagnostic.note(format!("Please report this bug over here: {}", BUG_REPORT_URL));

    diagnostic.emit();

    // If backtraces are enabled, also print the query stack
    let backtrace = env::var_os("RUST_BACKTRACE").map_or(false, |x| &x != "0");

    if backtrace {
        try_print_query_stack(&diag_ctxt, None, None);
    }
}

struct DefaultCallbacks;
impl rustc_driver::Callbacks for DefaultCallbacks {}

fn main() {
    let handler = EarlyDiagCtxt::new(ErrorOutputType::default());
    rustc_driver::init_rustc_env_logger(&handler);

    // Set the default log level to info
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    // env_logger::init();

    // env_logger::init();
    env_logger::Builder::new()
        .parse_default_env()
        .format(|buf, record| {
            let mut origin_style = buf.style();
            origin_style.set_color(env_logger::fmt::Color::Cyan).set_bold(true);
            let origin = format!("<{}:{}>", record.file().unwrap_or("unknown"), record.line().unwrap_or(0));
            let origin = origin_style.value(origin);

            let level = record.level();
            let level_style = buf.default_level_style(level);
            let meta = format!("[{}:{}]", level, chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"));
            let meta = level_style.value(meta);

            writeln!(buf, "{} {} {}", meta, origin, record.args())
        })
        .init();

    lazy_static::initialize(&ICE_HOOK);

    setup_plugin();
}

fn setup_plugin() {
    let mut args = env::args().collect::<Vec<_>>();

    let is_wrapper = args.get(1).map(|s| s.contains("rustc")).unwrap_or(false);

    if is_wrapper {
        args.remove(1);
    }

    let hazardflow: HazardflowArgs = if is_wrapper {
        // serde_json::from_str(&std::env::var("HAZARDFLOW_ARGS").unwrap()).unwrap()
        todo!()
    } else {
        let all_args = Args::parse_from(&args);
        args = all_args.rust_flags;
        all_args.hazardflow
    };

    let sysroot = sysroot_path();
    args.push(format!("--sysroot={}", sysroot));

    let normal_rustc = args.iter().any(|arg| arg.starts_with("--print"));
    let primary_package = std::env::var("CARGO_PRIMARY_PACKAGE").is_ok();

    // Did the user ask to compile this crate? Either they explicitly invoked `hazardflow-rustc` or this is a primary package.
    let user_asked_for = !is_wrapper || primary_package;

    if normal_rustc || !user_asked_for {
        return RunCompiler::new(&args, &mut DefaultCallbacks {}).run().unwrap();
    } else {
        // TODO: Parse `Cargo.toml` from the given directory and fill in the arguments automatically.
        let extern_path_ext = if cfg!(target_os = "macos") {
            "dylib"
        } else if cfg!(target_os = "linux") {
            "so"
        } else {
            todo!("Unsupported target OS")
        };

        args.extend(vec![
            "--crate-name=hazardflow".to_string(),
            "--edition=2021".to_string(),
            "hazardflow-designs/src/lib.rs".to_string(),
            "--crate-type=lib".to_string(),
            "--extern".to_string(),
            format!("hazardflow_macro=./target/debug/libhazardflow_macro.{extern_path_ext}"),
        ]);

        let opts = hazardflow.into_opts();
        let mut callbacks = hazardflow::compiler::Compiler::new(opts);

        RunCompiler::new(&args, &mut callbacks).run().unwrap();
    }
}

fn sysroot_path() -> String {
    let toolchain: toml::Value = toml::from_str(include_str!("../../rust-toolchain")).unwrap();
    let channel = toolchain["toolchain"]["channel"].as_str().unwrap();

    let output =
        Command::new("rustup").arg("run").arg(channel).arg("rustc").arg("--print").arg("sysroot").output().unwrap();

    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}
