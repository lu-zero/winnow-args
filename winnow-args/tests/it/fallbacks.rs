//! `env` and `default`: the command line wins, then the environment, then the default.

use winnow_args::{Args, Error, ErrorKind, ValueEnum, with_env};

#[derive(ValueEnum, Debug, PartialEq, Clone, Copy)]
enum Cache {
    Auto,
    Off,
}

#[derive(Args, Debug, PartialEq)]
struct Cli {
    #[arg(long, env = "WA_MONOREPO")]
    monorepo: bool,
    #[arg(short, long, count, env = "WA_VERBOSE")]
    verbose: u8,
    #[arg(long, env = "WA_JOBS", default = "4")]
    jobs: Option<u32>,
    #[arg(long, env = "WA_ENV", delimiter = ',')]
    env: Vec<String>,
    #[arg(long, default = "auto", choices("auto", "always", "never"))]
    color: Option<String>,
    #[arg(long, env = "WA_CACHE")]
    cache: Option<Cache>,
    #[arg(positional, default = ".")]
    dir: Option<String>,
}

fn parse(line: &[&str], env: &[(&str, &str)]) -> Result<Cli, Error> {
    let words = crate::words(line);
    with_env(env, || Cli::parse_from(&words))
}

fn ok(line: &[&str], env: &[(&str, &str)]) -> Cli {
    parse(line, env).unwrap_or_else(|e| panic!("{line:?} {env:?}: {e}"))
}

#[test]
fn defaults_fill_what_nothing_else_did() {
    let cli = ok(&[], &[]);
    assert_eq!(
        (
            cli.monorepo,
            cli.verbose,
            cli.jobs,
            cli.color.as_deref(),
            cli.dir.as_deref()
        ),
        (false, 0, Some(4), Some("auto"), Some("."))
    );
    assert!(cli.env.is_empty() && cli.cache.is_none());
    assert_eq!(ok(&["x"], &[]).dir.as_deref(), Some("x"));
}

#[test]
fn the_environment_fills_before_the_default() {
    assert_eq!(ok(&[], &[("WA_JOBS", "8")]).jobs, Some(8));
    assert_eq!(ok(&[], &[("WA_ENV", "a,b")]).env, ["a", "b"]);
    assert_eq!(ok(&[], &[("WA_CACHE", "off")]).cache, Some(Cache::Off));
    assert_eq!(ok(&[], &[("WA_VERBOSE", "2")]).verbose, 2);
    // An unparseable count is ignored rather than an error, as in usage.
    assert_eq!(ok(&[], &[("WA_VERBOSE", "lots")]).verbose, 0);
}

#[test]
fn the_command_line_wins() {
    let env = [("WA_JOBS", "8"), ("WA_ENV", "a,b"), ("WA_VERBOSE", "2")];
    let cli = ok(&["--jobs", "2", "--env", "c", "-v"], &env);
    assert_eq!(
        (cli.jobs, cli.env, cli.verbose),
        (Some(2), vec!["c".to_string()], 1)
    );
}

#[test]
fn a_switch_from_the_environment_is_true_unless_it_says_otherwise() {
    for off in ["", "0", "false", "no", "off"] {
        assert!(!ok(&[], &[("WA_MONOREPO", off)]).monorepo, "{off:?}");
    }
    for on in ["1", "true", "yes", "anything"] {
        assert!(ok(&[], &[("WA_MONOREPO", on)]).monorepo, "{on:?}");
    }
}

#[test]
fn a_bad_environment_value_names_its_variable() {
    let e = parse(&[], &[("WA_JOBS", "many")]).unwrap_err();
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::InvalidValue, Some("$WA_JOBS"))
    );
    let e = parse(&[], &[("WA_CACHE", "on")]).unwrap_err();
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::InvalidChoice, Some("$WA_CACHE"))
    );
}

#[derive(Args, Debug)]
struct Required {
    #[arg(long, env = "WA_TOKEN")]
    token: String,
}

#[test]
fn the_environment_satisfies_a_required_flag() {
    let e = with_env(&[], || Required::parse_from(&[])).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::MissingRequired);
    let parsed = with_env(&[("WA_TOKEN", "t")], || Required::parse_from(&[])).unwrap();
    assert_eq!(parsed.token, "t");
}
