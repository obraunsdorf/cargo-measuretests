use cargo::util::command_prelude::*;
use cargo::core::shell::Shell;
use std::env;

#[macro_use]
extern crate clap;

const COMMAND_NAME: &str = "measuretests";


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

    println!("config: {:?}", config);

    let matches = App::new(format!("cargo-{}", COMMAND_NAME))
        .author("Oliver Braunsdorf, <oliver.braunsdorf@gmx.de>")
        .about("Tool to roughly measure the execution time of a crate's test suite")
        .version(concat!("version: ", crate_version!()))
        .bin_name("cargo")
        .subcommand(clap::SubCommand::with_name(COMMAND_NAME)
            .about("Tool to roughly measure the execution time of a crate's test suite")
            .version(concat!("version: ", crate_version!()))
            .arg(
                Arg::with_name("TESTNAME")
                    .help("If specified, only run tests containing this string in their names"),
            )
            .arg(
                Arg::with_name("args")
                    .help("Arguments for the test binary")
                    .multiple(true)
                    .last(true),
            )
            .arg(
                opt(
                    "quiet",
                    "Display one character per test instead of one line",
                )
                    .short("q"),
            )
            .arg(
                opt(
                    "runs",
                    "Specifies the number of times how often tests will be run",
                )
                    .default_value("32")
                    .short("r"),
            )
            .arg(
                opt(
                    "warmup",
                    "Specifies time in seconds to warmup tests",
                )
                    .default_value("3")
                    .short("w"),
            )
            .arg_targets_all(
                "Test only this package's library unit tests",
                "Test only the specified binary",
                "Test all binaries",
                "Test only the specified example",
                "Test all examples",
                "Test only the specified test target",
                "Test all tests",
                "Test only the specified bench target",
                "Test all benches",
                "Test all targets",
            )
            .arg(opt("doc", "Test only this library's documentation"))
            .arg(opt("no-run", "Compile, but don't run tests"))
            .arg(opt("no-fail-fast", "Run all tests regardless of failure"))
            .arg_package_spec(
                "Package to run tests for",
                "Test all packages in the workspace",
                "Exclude packages from the test",
            )
            .arg_jobs()
            .arg_release("Build artifacts in release mode, with optimizations")
            .arg_features()
            .arg_target_triple("Build for the target triple")
            .arg_target_dir()
            .arg_manifest_path()
            .arg_message_format()
            .after_help(
                "\
The test filtering argument TESTNAME and all the arguments following the
two dashes (`--`) are passed to the test binaries and thus to libtest
(rustc's built in unit-test and micro-benchmarking framework). If you're
passing arguments to both Cargo and the binary, the ones after `--` go to the
binary, the ones before go to Cargo. For details about libtest's arguments see
the output of `cargo test -- --help`. As an example, this will run all
tests with `foo` in their name on 3 threads in parallel:
    cargo test foo -- --test-threads 3
If the `--package` argument is given, then SPEC is a package ID specification
which indicates which package should be tested. If it is not given, then the
current package is tested. For more information on SPEC and its format, see the
`cargo help pkgid` command.
All packages in the workspace are tested if the `--all` flag is supplied. The
`--all` flag is automatically assumed for a virtual manifest.
Note that `--exclude` has to be specified in conjunction with the `--all` flag.
The `--jobs` argument affects the building of the test executable but does
not affect how many jobs are used when running the tests. The default value
for the `--jobs` argument is the number of CPUs. If you want to control the
number of simultaneous running test cases, pass the `--test-threads` option
to the test binaries:
    cargo test -- --test-threads=1
Compilation can be configured via the `test` profile in the manifest.
By default the rust test harness hides output from test execution to
keep results readable. Test output can be recovered (e.g., for debugging)
by passing `--nocapture` to the test binaries:
    cargo test -- --nocapture
To get the list of all options available for the test binaries use this:
    cargo test -- --help
",
            )
        )
        .get_matches();

    let args = match matches.subcommand_matches(COMMAND_NAME) {
        Some(x) => x,
        None => &matches
    };

    cargo_measuretests::exec(&mut config, &args);
}