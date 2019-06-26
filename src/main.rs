use cargo::ops::{self, CompileFilter, FilterRule, LibRule};
use cargo::core::Workspace;
use cargo::util::errors::*;
use cargo::core::compiler::Compilation;
use cargo::util::command_prelude::*;
use cargo::core::shell::Verbosity;
use cargo::core::shell::Shell;
use std::env;

#[macro_use]
extern crate clap;

fn main() {
    let cmd_args: Vec<String> = env::args().collect();
    println!("Starting to measure tests with args: \n {:?}", cmd_args);

    let mut config = match Config::default() {
        Ok(cfg) => cfg,
        Err(e) => {
            let mut shell = Shell::new();
            cargo::exit_with_error(e.into(), &mut shell)
        }
    };

    let args = App::new("cargo-measuretests")
        .author("Oliver Braunsdorf, <oliver.braunsdorf@gmx.de>")
        .about("Tool to roughly measure the execution time of a crate's test suite")
        .version(concat!("version: ", crate_version!()))
        .bin_name("cargo")
        .subcommand(clap::SubCommand::with_name("measuretests")
                        .about("Tool to roughly measure the execution time of a crate's test suite")
                        .version(concat!("version: ", crate_version!())))
        .get_matches();


    cargo_measuretests::exec(&mut config, &args);
}