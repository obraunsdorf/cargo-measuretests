use std::env;
use cargo::core::shell::Shell;
use cargo::util::Config;

#[test]
pub fn simpletest() {
    let restore_dir = env::current_dir().unwrap();
    let mut test_dir = restore_dir.clone();
    test_dir.push("tests/simpletest");
    match env::set_current_dir(&test_dir) {
        Err(e) => {
            eprintln!("could not set current dir to {:?}, because of the following error: {:?}", test_dir, e);
            assert!(false);
        }

        _ => {}
    }

    let mut config = match Config::default() {
        Ok(cfg) => cfg,
        Err(e) => {
            let mut shell = Shell::new();
            cargo::exit_with_error(e.into(), &mut shell)
        }
    };

    let args = clap::App::new("cargo").get_matches();

    cargo_measuretests::exec(&mut config, &args);

    env::set_current_dir(restore_dir).unwrap();
}

/*pub fn check_percentage(project_name: &str, minimum_coverage: f64, has_lines: bool) {
    let mut config = Config::default();
    config.verbose = true;
    config.test_timeout = Duration::from_secs(60);
    let restore_dir = env::current_dir().unwrap();
    let test_dir = get_test_path(project_name);
    env::set_current_dir(&test_dir).unwrap();
    config.manifest = test_dir;
    config.manifest.push("Cargo.toml");

    let (res, _) = launch_tarpaulin(&config).unwrap();

    env::set_current_dir(restore_dir).unwrap();
    assert!(res.coverage_percentage() >= minimum_coverage);
    if has_lines {
        assert!(res.total_coverable() > 0);
    }
}*/