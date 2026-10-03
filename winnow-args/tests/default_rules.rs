//! `default_fn` and `default_if`: defaults computed, or that depend on another flag.
#![cfg(feature = "derive")]

use winnow_args::{Args, Error, ValueEnum, with_env};

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
enum Format {
    Text,
    Json,
}

fn cores() -> usize {
    8
}

fn all() -> Vec<String> {
    vec!["src".into(), "tests".into()]
}

#[derive(Args, Debug, PartialEq)]
struct Cli {
    #[arg(long)]
    json: bool,
    #[arg(long)]
    format: Option<Format>,
    /// Given: `--json` was; valued: `--format` is `json`.
    #[arg(
        long,
        default_if("--json", "never"),
        default_if("--format", "json", "never"),
        default = "auto"
    )]
    color: String,
    #[arg(long, default_if("--format", "json", "true"))]
    quiet: bool,
    #[arg(short, long, env = "JOBS", default_fn = cores, default_note = "one per core")]
    jobs: usize,
    #[arg(long, default_fn = all)]
    dir: Vec<String>,
    #[arg(long, negate, default_fn = on)]
    cache: bool,
}

fn on() -> bool {
    true
}

fn parse(line: &[&str]) -> Result<Cli, Error> {
    with_env(&[], || Cli::try_parse_from(line))
}

#[test]
fn a_default_follows_another_flag() {
    assert_eq!(parse(&[]).unwrap().color, "auto");
    assert_eq!(parse(&["--json"]).unwrap().color, "never");
    assert_eq!(parse(&["--format", "json"]).unwrap().color, "never");
    assert_eq!(parse(&["--format", "text"]).unwrap().color, "auto");
    // The command line still wins.
    assert_eq!(
        parse(&["--json", "--color=always"]).unwrap().color,
        "always"
    );
}

#[test]
fn a_switch_can_follow_a_value() {
    assert!(parse(&["--format=json"]).unwrap().quiet);
    assert!(!parse(&["--format=text"]).unwrap().quiet);
    assert!(!parse(&[]).unwrap().quiet);
}

#[test]
fn a_function_gives_the_default_after_the_command_line_and_the_environment() {
    let cli = parse(&[]).unwrap();
    assert_eq!((cli.jobs, cli.cache), (8, true));
    assert_eq!(cli.dir, ["src", "tests"]);
    let cli = parse(&["-j2", "--dir", "x", "--no-cache"]).unwrap();
    assert_eq!((cli.jobs, cli.cache), (2, false));
    assert_eq!(cli.dir, ["x"]);
    let from_env = with_env(&[("JOBS", "3")], || Cli::try_parse_from::<_, &str>([]));
    assert_eq!(from_env.unwrap().jobs, 3);
}

#[test]
fn help_shows_the_note_and_does_not_require_the_flag() {
    let help = winnow_args::help::render(Cli::HELP, &["cli"], false);
    assert!(help.contains("[default: one per core]"), "{help}");
    assert!(help.contains("Usage: cli [OPTIONS]"), "{help}");
}
