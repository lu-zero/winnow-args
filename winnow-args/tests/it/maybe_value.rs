//! `Option<Option<T>>`: a flag whose value may be left out. Absent is `None`,
//! bare is `Some(None)`, with a value `Some(Some(v))`.

use winnow_args::{Args, ErrorKind, ValueEnum};

#[derive(ValueEnum, Debug, PartialEq, Clone, Copy)]
enum When {
    Always,
    Never,
}

#[derive(Args, Debug, PartialEq)]
struct Cli {
    #[arg(long)]
    color: Option<Option<When>>,
    #[arg(short, long)]
    jobs: Option<Option<u32>>,
    #[arg(long, require_equals)]
    log: Option<Option<String>>,
    #[arg(short, long)]
    verbose: bool,
    #[arg(positional)]
    rest: Vec<String>,
}

fn parse(line: &[&str]) -> Cli {
    Cli::parse_from(line).unwrap()
}

#[test]
fn absent_bare_and_with_a_value() {
    assert_eq!(parse(&[]).color, None);
    assert_eq!(parse(&["--color"]).color, Some(None));
    assert_eq!(parse(&["--color", "never"]).color, Some(Some(When::Never)));
    assert_eq!(parse(&["--color=always"]).color, Some(Some(When::Always)));
    assert_eq!(parse(&["-j"]).jobs, Some(None));
    assert_eq!(parse(&["-j4"]).jobs, Some(Some(4)));
    assert_eq!(parse(&["-j", "4"]).jobs, Some(Some(4)));
}

#[test]
fn bare_before_a_flag_or_at_the_end() {
    let cli = parse(&["--color", "-v"]);
    assert_eq!((cli.color, cli.verbose), (Some(None), true));
    let cli = parse(&["-v", "--jobs"]);
    assert_eq!(cli.jobs, Some(None));
}

#[test]
fn require_equals_takes_no_detached_word() {
    let cli = parse(&["--log", "file"]);
    assert_eq!(
        (cli.log, &cli.rest[..]),
        (Some(None), &["file".to_owned()][..])
    );
    assert_eq!(parse(&["--log=x"]).log, Some(Some("x".into())));
    assert_eq!(parse(&["--log="]).log, Some(Some(String::new())));
}

#[test]
fn a_bad_value_is_an_error() {
    let e = Cli::parse_from(["--color=sometimes"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidChoice);
}

#[derive(Args, Debug, PartialEq)]
struct Defaulted {
    #[arg(long, env = "WA_TEST_MAYBE_VALUE", default = "2")]
    level: Option<Option<u8>>,
}

#[test]
fn env_and_default_hold_a_value() {
    assert_eq!(Defaulted::parse_from([""; 0]).unwrap().level, Some(Some(2)));
    assert_eq!(
        Defaulted::parse_from(["--level"]).unwrap().level,
        Some(None)
    );
}

#[cfg(feature = "help-text")]
#[test]
fn help_marks_the_value_optional() {
    let help = winnow_args::help::render(Cli::HELP, &["cli"], false);
    assert!(help.contains("--color [<COLOR>]"), "{help}");
    assert!(help.contains("--log[=<LOG>]"), "{help}");
}

#[derive(Args, Debug, PartialEq)]
struct Made {
    #[arg(long)]
    base: Option<u8>,
    /// Made `T`, as an `Option<T>` field's is.
    #[arg(long, default_from(base, |base: &Option<u8>| base.unwrap_or(1) * 2))]
    level: Option<Option<u8>>,
}

#[test]
fn default_from_makes_the_value() {
    assert_eq!(
        Made::parse_from(["--base", "3"]).unwrap().level,
        Some(Some(6))
    );
    assert_eq!(Made::parse_from(["--level"]).unwrap().level, Some(None));
}
