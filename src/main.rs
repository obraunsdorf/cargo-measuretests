use cargo::ops::{self, CompileFilter, FilterRule, LibRule};
use cargo::core::Workspace;
use cargo::util::errors::*;
use cargo::core::compiler::Compilation;
use cargo::util::command_prelude::*;
use cargo::core::shell::Verbosity;
use cargo::core::shell::Shell;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Starting to measure tests with args: \n {:?}", args);

    let mut config = match Config::default() {
        Ok(cfg) => cfg,
        Err(e) => {
            let mut shell = Shell::new();
            cargo::exit_with_error(e.into(), &mut shell)
        }
    };

    let args = clap::App::new("cargo").get_matches();

    cargo_measuretests::exec(&mut config, &args);
}