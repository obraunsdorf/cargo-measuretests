use cargo::ops::{self, CompileFilter, FilterRule, LibRule};
use cargo::core::Workspace;
use cargo::util::errors::*;
use cargo::core::compiler::Compilation;
use cargo::util::command_prelude::*;
use cargo::core::shell::Verbosity;
use cargo::core::shell::Shell;

pub fn exec(config: &mut Config, args: &ArgMatches<'_>) -> CliResult {
    let ws = args.workspace(config)?;

    let mut compile_opts = args.compile_options(config, CompileMode::Test, Some(&ws))?;

    // `TESTNAME` is actually an argument of the test binary, but it's
    // important, so we explicitly mention it and reconfigure.
    let test_name: Option<&str> = args.value_of("TESTNAME");
    let test_args = args.value_of("TESTNAME").into_iter();
    let test_args = test_args.chain(args.values_of("args").unwrap_or_default());
    let test_args = test_args.collect::<Vec<_>>();

    let no_run = args.is_present("no-run");
    let doc = args.is_present("doc");
    if doc {
        if let CompileFilter::Only { .. } = compile_opts.filter {
            return Err(CliError::new(
                failure::format_err!("Can't mix --doc with other target selecting options"),
                101,
            ));
        }
        if no_run {
            return Err(CliError::new(
                failure::format_err!("Can't skip running doc tests with --no-run"),
                101,
            ));
        }
        compile_opts.build_config.mode = CompileMode::Doctest;
        compile_opts.filter = ops::CompileFilter::new(
            LibRule::True,
            FilterRule::none(),
            FilterRule::none(),
            FilterRule::none(),
            FilterRule::none(),
        );
    } else if test_name.is_some() {
        if let CompileFilter::Default { .. } = compile_opts.filter {
            compile_opts.filter = ops::CompileFilter::new(
                LibRule::Default, // compile the library, so the unit tests can be run filtered
                FilterRule::All,    // compile the binaries, so the unit tests in binaries can be run filtered
                FilterRule::All,    // compile the tests, so the integration tests can be run filtered
                FilterRule::none(), // specify --examples to unit test binaries filtered
                FilterRule::none(), // specify --benches to unit test benchmarks filtered
            ); // also, specify --doc to run doc tests filtered
        }
    }

    let ops = TestOptions {
        no_run,
        no_fail_fast: args.is_present("no-fail-fast"),
        compile_opts,
    };

    let err = run_tests(&ws, &ops, &test_args)?;
    match err {
        None => Ok(()),
        Some(err) => Err(match err.exit.as_ref().and_then(|e| e.code()) {
            Some(i) => CliError::new(failure::format_err!("{}", err.hint(&ws)), i),
            None => CliError::new(err.into(), 101),
        }),
    }
}

pub struct TestOptions<'a> {
    pub compile_opts: ops::CompileOptions<'a>,
    pub no_run: bool,
    pub no_fail_fast: bool,
}

pub fn run_tests(
    ws: &Workspace<'_>,
    options: &TestOptions<'_>,
    test_args: &[&str],
) -> CargoResult<Option<CargoTestError>> {
    let compilation = compile_tests(ws, options)?;

    if options.no_run {
        return Ok(None);
    }
    let (test, mut errors) = run_unit_tests(options, test_args, &compilation)?;

    // If we have an error and want to fail fast, then return.
    if !errors.is_empty() && !options.no_fail_fast {
        return Ok(Some(CargoTestError::new(test, errors)));
    }

    if errors.is_empty() {
        Ok(None)
    } else {
        Ok(Some(CargoTestError::new(test, errors)))
    }
}

/*pub fn run_benches(
    ws: &Workspace<'_>,
    options: &TestOptions<'_>,
    args: &[&str],
) -> CargoResult<Option<CargoTestError>> {
    let compilation = compile_tests(ws, options)?;

    if options.no_run {
        return Ok(None);
    }

    let mut args = args.to_vec();
    args.push("--bench");

    let (test, errors) = run_unit_tests(options, &args, &compilation)?;

    match errors.len() {
        0 => Ok(None),
        _ => Ok(Some(CargoTestError::new(test, errors))),
    }
}*/

fn compile_tests<'a>(
    ws: &Workspace<'a>,
    options: &TestOptions<'a>,
) -> CargoResult<Compilation<'a>> {
    let mut compilation = ops::compile(ws, &options.compile_opts)?;
    compilation
        .tests
        .sort_by(|a, b| (a.0.package_id(), &a.1, &a.2).cmp(&(b.0.package_id(), &b.1, &b.2)));
    Ok(compilation)
}

/// Runs the unit and integration tests of a package.
fn run_unit_tests(
    options: &TestOptions<'_>,
    test_args: &[&str],
    compilation: &Compilation<'_>,
) -> CargoResult<(Test, Vec<ProcessError>)> {
    let config = options.compile_opts.config;
    let cwd = options.compile_opts.config.cwd();

    let mut errors = Vec::new();

    for &(ref pkg, ref target, ref exe) in &compilation.tests {
        let kind = target.kind();
        let test = target.name().to_string();
        let exe_display = exe.strip_prefix(cwd).unwrap_or(exe).display();
        let mut cmd = compilation.target_process(exe, pkg)?;
        cmd.args(test_args);
        if target.harness() && config.shell().verbosity() == Verbosity::Quiet {
            cmd.arg("--quiet");
        }
        config
            .shell()
            .concise(|shell| shell.status("Running", &exe_display))?;
        config
            .shell()
            .verbose(|shell| shell.status("Running", &cmd))?;

        println!("starting time measure");
        let now = std::time::Instant::now();
        let result = cmd.exec();
        println!("time: {}", now.elapsed().as_micros());


        match result {
            Err(e) => {
                let e = e.downcast::<ProcessError>()?;
                errors.push((kind.clone(), test.clone(), pkg.name().to_string(), e));
                if !options.no_fail_fast {
                    break;
                }
            }
            Ok(()) => {}
        }
    }

    if errors.len() == 1 {
        let (kind, name, pkg_name, e) = errors.pop().unwrap();
        Ok((
            Test::UnitTest {
                kind,
                name,
                pkg_name,
            },
            vec![e],
        ))
    } else {
        Ok((
            Test::Multiple,
            errors.into_iter().map(|(_, _, _, e)| e).collect(),
        ))
    }
}

